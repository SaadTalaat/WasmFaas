use crate::compile::{compile, CompileError};
use crate::state::Handles;
use crate::status::Status;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use wasmer::{
    imports, Function as HostFunction, FunctionEnv as HostFunctionEnv,
    FunctionEnvMut as HostFunctionEnvMut, Instance, Module, Store,
};

#[derive(Deserialize)]
pub struct DeployableFunction {
    name: String,
    body: String,
}

#[derive(Debug)]
pub enum DeploymentError {
    LoadingInstance,
    InvalidModule,
    FunctionNotFound(String),
    CorruptFunctionDesc,
    UnsupportedType(String),
    NotAFunction,
    CompileError,
    InternalError(String),
}
impl From<CompileError> for DeploymentError {
    fn from(error: CompileError) -> Self {
        match error {
            CompileError::CompileError => DeploymentError::CompileError,
            _ => DeploymentError::InternalError(format!("{}", error)),
        }
    }
}

impl IntoResponse for DeploymentError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        };
        let msg = match self {
            Self::LoadingInstance => "could not parse wasm instance".to_owned(),
            Self::InvalidModule => "corrupt wasm module".to_owned(),
            Self::CorruptFunctionDesc => "corrupt function".to_owned(),
            Self::FunctionNotFound(name) => format!("function not found: {}", name),
            Self::UnsupportedType(type_name) => format!("unsupported type: {}", type_name),
            Self::NotAFunction => "not a function".to_owned(),
            Self::CompileError => "compilation error".to_owned(),
            Self::InternalError(msg) => format!("Internal Error: {}", msg),
        };

        (status, msg).into_response()
    }
}

pub async fn deploy(
    Extension(handles): Extension<Handles>,
    Json(func): Json<DeployableFunction>,
) -> Result<Json<Status>, DeploymentError> {
    let bytes = compile(&func.body).await?;
    let description = describe_fn(&func.name, &bytes)?;
    match description {
        TypeDesc::Function(_) => {
            println!("Description: {:?}", description);
            println!("Deploying: {}", bytes.len());
            let json_desc = serde_json::to_string(&description).unwrap();
            handles
                .storage
                .store(&func.name, &bytes, json_desc)
                .unwrap();
            Ok(Json(Status::ok()))
        }
        _ => Err(DeploymentError::NotAFunction),
    }
}

fn describe_fn(name: &str, bytes: &[u8]) -> Result<TypeDesc, DeploymentError> {
    let raw_description = extract_description(name, bytes)?;
    let parsed_description = TypeDesc::decode(&mut &raw_description[..])?;
    match parsed_description {
        TypeDesc::Function(_) => Ok(parsed_description),
        _ => Err(DeploymentError::NotAFunction),
    }
}

fn extract_description(name: &str, bytes: &[u8]) -> Result<Vec<u8>, DeploymentError> {
    let mut store = Store::default();
    let module = Module::new(&store, bytes).map_err(|_| DeploymentError::InvalidModule)?;
    fn description(mut env: HostFunctionEnvMut<Vec<u8>>, x: u8) {
        env.data_mut().push(x);
    }

    let env = HostFunctionEnv::new(&mut store, Vec::new());
    let import_obj = imports! {
        "__wbindgen_placeholder__" => {
            "__wbindgen_describe" => HostFunction::new_typed_with_env(
                &mut store,
                &env,
                description)
        },
        "__wbindgen_externref_xform__" => {
            "__wbindgen_externref_table_grow" => HostFunction::new_typed(&mut store, |x: i32|x),
            "__wbindgen_externref_table_set_null" => HostFunction::new_typed(&mut store, |_: i32|())
        }
    };

    let instance = Instance::new(&mut store, &module, &import_obj)
        .map_err(|_| DeploymentError::LoadingInstance)?;
    let desc_func_name = format!("__wbindgen_describe_{}", name);
    let wasm_desc_func = instance
        .exports
        .get_function(&desc_func_name)
        .map_err(|_| DeploymentError::FunctionNotFound(String::from(name)))?;
    wasm_desc_func
        .call(&mut store, &vec![])
        .map_err(|_| DeploymentError::CorruptFunctionDesc)?;
    let result = env.as_ref(&store).clone();
    Ok(result)
}

#[derive(Debug, Serialize)]
#[repr(u8)]
enum TypeDesc {
    I8,                      // 0
    U8,                      // 1
    I16,                     // 2
    U16,                     // 3
    I32,                     // 4
    U32,                     // 5
    I64,                     // 6
    U64,                     // 7
    F32,                     // 8
    F64,                     // 9
    Boolean,                 // 10
    Function(Box<Function>), // 11
    _Closure,                // 12
    CachedString,            // 13
    String,                  // 14
    Ref(Box<Self>),          // 15
    RefMut(Box<Self>),       // 16
    LongRef,                 // 17
    Slice(Box<Self>),        // 18
    Vector(Box<Self>),       // 19
    Externref,               // 20
    NamedExternref,          // 21
    Enum,                    // 22
    RustStruct,              // 23
    Char,                    // 24
    Option(Box<Self>),       // 25
    Result,                  // 26
    Unit,                    // 27
    ClampedU8,               // 28
}

impl TypeDesc {
    pub fn decode(bytes: &mut &[u8]) -> Result<Self, DeploymentError> {
        let byte = bytes[0];
        let unsupported_err = |msg: &str| -> Result<Self, DeploymentError> {
            Err(DeploymentError::UnsupportedType(msg.to_owned()))
        };
        *bytes = &bytes[1..];
        let ty = match byte {
            0 => Self::I8,
            1 => Self::U8,
            2 => Self::I16,
            3 => Self::U16,
            4 => Self::I32,
            5 => Self::U32,
            6 => Self::I64,
            7 => Self::U64,
            8 => Self::F32,
            9 => Self::F64,
            10 => Self::Boolean,
            11 => Self::Function(Box::new(Function::decode(bytes)?)),
            12 => return unsupported_err("closure"),
            13 => return unsupported_err("cached_string"),
            14 => Self::String,
            15 => Self::Ref(Box::new(Self::decode(bytes)?)),
            16 => Self::RefMut(Box::new(Self::decode(bytes)?)),
            17 => return unsupported_err("long_ref"),
            18 => Self::Slice(Box::new(Self::decode(bytes)?)),
            19 => Self::Vector(Box::new(Self::decode(bytes)?)),
            20 => return unsupported_err("extern_ref"),
            21 => return unsupported_err("named_extern_ref"),
            22 => return unsupported_err("enum"),
            23 => return unsupported_err("struct"),
            24 => Self::Char,
            25 => Self::Option(Box::new(Self::decode(bytes)?)),
            26 => return unsupported_err("result"),
            27 => Self::Unit,
            28 => return unsupported_err("clamped"),
            _ => return unsupported_err("undefined"),
        };
        Ok(ty)
    }
}

#[derive(Debug, Serialize)]
struct Function {
    pub params: Vec<TypeDesc>,
    pub shim_idx: u8,
    pub ret: TypeDesc,
    pub inner_ret: Option<TypeDesc>,
}

impl Function {
    pub fn decode(bytes: &mut &[u8]) -> Result<Self, DeploymentError> {
        let shim_idx = bytes[0];
        let nparam = bytes[1];
        *bytes = &bytes[2..];
        let mut params = vec![];
        for _ in 0..nparam {
            params.push(TypeDesc::decode(bytes)?);
        }
        let ret = TypeDesc::decode(bytes)?;
        let inner_ret = Some(TypeDesc::decode(bytes)?);
        let instance = Self {
            params,
            shim_idx,
            ret,
            inner_ret,
        };
        Ok(instance)
    }
}

#[derive(Serialize)]
struct Closure {}
