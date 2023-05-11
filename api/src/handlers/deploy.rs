use super::result::{APIError, APIResult};
use crate::{
    db::models::Function, extensions::Handles, state::AppState, status::Status,
    util::wasm::extract_description,
};
use axum::{
    extract::{Extension, State},
    Json,
};

use serde::{Deserialize, Serialize};

pub async fn deploy(
    Extension(handles): Extension<Handles>,
    State(state): State<AppState>,
    Json(func): Json<DeployableFunction>,
) -> APIResult {
    // Compile function
    tracing::info!("Deploying function: {}", &func.name);
    let bytes = handles.compiler.compile(&func.body).await?;
    tracing::trace!("Compiled function with size: {} bytes", bytes.len());
    // Extract function signature
    let description = extract_description(&func.name, &bytes)?;
    tracing::trace!("Function signature: {:?}", &description);
    // Store function to Storage medium, i.e. Disk, S3..etc
    let suffix = rand::random::<u32>();
    let filename = format!("{}_{}", &func.name, suffix);
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
    let func = Function::new(&func.name, &path, &path, &description)?
        .insert(&mut db_conn)
        .await
        .map_err(|_| APIError::InternalError)?;
    let response = Deployment::new(func);
    tracing::trace!("Successfuly deployed function");
    Ok(Status::ok_payload(response))
}

#[derive(Deserialize)]
pub struct DeployableFunction {
    name: String,
    body: String,
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
