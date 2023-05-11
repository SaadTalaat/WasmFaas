use super::error::Error as DBError;
use crate::db::{schema::functions, DBPoolConnection};
use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::*,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsValue;
use std::{io::Write, time::SystemTime};

#[derive(Selectable, Queryable, Identifiable)]
#[diesel(table_name = functions)]
pub struct Function {
    pub id: i32,
    pub arity: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub name: String,
    pub uri: String,
    pub user_uri: String,
    pub signature: FunctionType,
}

impl Function {
    pub fn new<'a>(
        name: &'a str,
        uri: &'a str,
        user_uri: &'a str,
        description: &[u8],
    ) -> Result<NewFunction<'a>, DBError> {
        let (arity, signature) = Self::describe(description)?;
        let instance = NewFunction {
            arity: arity as i32,
            name,
            uri,
            user_uri,
            signature,
        };
        Ok(instance)
    }

    fn describe(mut description: &[u8]) -> Result<(usize, FunctionType), DBError> {
        let parsed_description = TypeDesc::decode(&mut description)?;
        match parsed_description {
            TypeDesc::Function(func) => Ok((func.arity(), *func)),
            _ => Err(DBError::NotAFunction),
        }
    }
    pub async fn get(id: i32, conn: &mut DBPoolConnection) -> Result<Self, DBError> {
        functions::table
            .filter(functions::id.eq(id))
            .select(Function::as_select())
            .get_result(conn)
            .await
            .map_err(|e| DBError::DBError(e))
    }

    pub fn validate_args(&self, args: &Vec<JsValue>) -> Result<(), DBError> {
        let args_arity = args.len() as i32;
        // TODO: validate arg types
        if self.arity != args_arity {
            Err(DBError::MismatchedArgs)
        } else {
            Ok(())
        }
    }
}
#[derive(Insertable)]
#[diesel(table_name = functions)]
pub struct NewFunction<'a> {
    arity: i32,
    name: &'a str,
    uri: &'a str,
    user_uri: &'a str,
    signature: FunctionType,
}

impl<'a> NewFunction<'a> {
    pub async fn insert(&self, db_conn: &mut DBPoolConnection) -> Result<Function, DBError> {
        diesel::insert_into(functions::table)
            .values(self)
            .returning(Function::as_returning())
            .get_result(db_conn)
            .await
            .map_err(|e| DBError::DBError(e))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum TypeDesc {
    I8,                          // 0
    U8,                          // 1
    I16,                         // 2
    U16,                         // 3
    I32,                         // 4
    U32,                         // 5
    I64,                         // 6
    U64,                         // 7
    F32,                         // 8
    F64,                         // 9
    Boolean,                     // 10
    Function(Box<FunctionType>), // 11
    _Closure,                    // 12
    CachedString,                // 13
    String,                      // 14
    Ref(Box<Self>),              // 15
    RefMut(Box<Self>),           // 16
    LongRef,                     // 17
    Slice(Box<Self>),            // 18
    Vector(Box<Self>),           // 19
    Externref,                   // 20
    NamedExternref,              // 21
    Enum,                        // 22
    RustStruct,                  // 23
    Char,                        // 24
    Option(Box<Self>),           // 25
    Result,                      // 26
    Unit,                        // 27
    ClampedU8,                   // 28
}

impl TypeDesc {
    pub fn decode(bytes: &mut &[u8]) -> Result<Self, DBError> {
        let byte = bytes[0];
        let unsupported_err =
            |msg: &str| -> Result<Self, DBError> { Err(DBError::UnsupportedType(msg.to_owned())) };
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
            11 => Self::Function(Box::new(FunctionType::decode(bytes)?)),
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

impl ToSql<Jsonb, Pg> for FunctionType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = serde_json::to_value(self)?;
        out.write_all(&[1])?;
        serde_json::to_writer(out, &value)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl FromSql<Jsonb, Pg> for FunctionType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
        let signature = serde_json::from_value(value)?;
        Ok(signature)
    }
}

#[derive(Debug, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Jsonb)]
pub struct FunctionType {
    params: Vec<TypeDesc>,
    shim_idx: u8,
    ret: TypeDesc,
    inner_ret: Option<TypeDesc>,
}

impl FunctionType {
    pub fn decode(bytes: &mut &[u8]) -> Result<Self, DBError> {
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

    pub fn arity(&self) -> usize {
        self.params.len()
    }
}
