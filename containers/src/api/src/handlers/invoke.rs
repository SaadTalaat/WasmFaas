use crate::registry::BackendError;
use crate::state::Handles;
use crate::status::Status;
use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use wasmer::{imports, Instance, Module, Store, Value as WasmValue};

#[derive(Deserialize)]
pub struct InvokeRequest {
    args: Vec<JsonValue>,
}

#[axum_macros::debug_handler]
pub async fn invoke(
    Extension(handles): Extension<Handles>,
    Path(name): Path<String>,
    Json(request): Json<InvokeRequest>,
) -> Result<Status, BackendError> {
    tracing::trace!("Invoking function: {} with args: {:?}", name, request.args);
    let registry = &handles.registry;
    tracing::trace!("Dispatching invocation request to worker registry");
    registry.invoke(name, request.args).await
}

/// XXX: Function not used,
/// just a boilerplate function to executing wasm code
/// for rust
async fn _invoke_local(name: &str, user_args: Vec<JsonValue>, wasm_bytes: Vec<u8>) {
    let mut store = Store::default();
    let module = Module::new(&store, wasm_bytes).unwrap();
    let import_obj = imports! {};
    let instance = Instance::new(&mut store, &module, &import_obj).unwrap();
    let memory = instance.exports.get_memory("memory").unwrap();
    let func = instance.exports.get_function(name).unwrap();

    let mut address_ptr = 0;

    let mut args: Vec<WasmValue> = vec![];
    for arg in user_args.iter() {
        match arg {
            JsonValue::Bool(b) => args.push(WasmValue::I32(if *b { 1 } else { 0 })),
            JsonValue::Number(n) if arg.is_i64() => args.push(WasmValue::I64(n.as_i64().unwrap())),
            JsonValue::Number(n) if arg.is_f64() => args.push(WasmValue::F64(n.as_f64().unwrap())),
            JsonValue::String(n) => {
                memory
                    .view(&mut store)
                    .write(address_ptr, n.as_bytes())
                    .unwrap();
                args.push(WasmValue::I32(address_ptr as i32));
                args.push(WasmValue::I32(n.bytes().len() as i32));
                address_ptr += n.bytes().len() as u64;
            }
            _ => panic!("Unknown arg {:?}", arg),
        }
    }
    println!(
        "Func:{}, param_arity: {}, result_arity: {}, type: {}",
        name,
        func.param_arity(&mut store),
        func.result_arity(&mut store),
        func.ty(&mut store)
    );
    let result = func.call(&mut store, &args).unwrap();
    let val = &result[0];
    println!("Result: {:?}", val);
    format!("Result: {:?}\n", val);
    ()
}
