use super::result::{APIError, APIResult};
use crate::{
    db::models::Function, extensions::Handles, extract::JsonOrWasm, state::AppState,
    status::Status, util::wasm::extract_description,
};
use axum::extract::{Extension, State};
use serde::{Deserialize, Serialize};

#[axum::debug_handler]
pub async fn deploy(
    handles: Extension<Handles>,
    state: State<AppState>,
    JsonOrWasm(func): JsonOrWasm<DeployableFunction>,
) -> APIResult {
    let bytes = match func {
        DeployableFunction::Code(code) => {
            tracing::trace!("Deploying function");
            let bytes = handles.compiler.compile(&code).await?;
            tracing::trace!("Compiled function with size: {} bytes", bytes.len());
            bytes
        }
        DeployableFunction::Bytes(bytes) => {
            tracing::trace!("Deploying Wasm module, payload size: {}", bytes.len());
            bytes
        }
    };
    // Extract function signature
    let (name, description) = extract_description(&bytes)?;
    tracing::trace!("Function signature: {:?}", &description);
    // Store function to Storage medium, i.e. Disk, S3..etc
    let suffix = rand::random::<u32>();
    let filename = format!("{}_{}", &name, suffix);
    let path = handles
        .storage
        .store(&filename, &bytes)
        .await
        .map_err(|_| APIError::InternalError)?;
    // Create a function record in DB
    tracing::debug!("Writing function to DB");
    let mut db_conn = state
        .get_db_conn()
        .await
        .map_err(|_| APIError::InternalError)?;
    let func = Function::new(&name, &path, &path, &description)?
        .insert(&mut db_conn)
        .await
        .map_err(|_| APIError::InternalError)?;
    let response = Deployment::new(func);
    tracing::trace!("Successfuly deployed function");
    Ok(Status::ok_payload(response))
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum DeployableFunction {
    Code(String),
    Bytes(Vec<u8>),
}

impl From<bytes::Bytes> for DeployableFunction {
    fn from(payload: bytes::Bytes) -> DeployableFunction {
        DeployableFunction::Bytes(payload.to_vec())
    }
}

#[derive(Serialize)]
struct Deployment {
    id: i32,
    name: String,
    uri: String,
}

impl Deployment {
    fn new(func: Function) -> Self {
        Self {
            id: func.id,
            name: func.name,
            uri: func.user_uri,
        }
    }
}
