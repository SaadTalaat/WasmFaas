use crate::{
    registry::BackendError,
    extensions::Handles,
    state::AppState,
    db::models::function::{Function},
};
use super::result::{APIResult};
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;

pub async fn invoke(
    Extension(handles): Extension<Handles>,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<InvokeRequest>,
) -> APIResult {
    let mut db_conn = state.get_db_conn().await.map_err(|_| BackendError::NoReply)?;
    let func = Function::get(&name, &mut db_conn).await?;
    func.validate_args(&request.args)?;
    tracing::trace!("Invoking function: {} with args: {:?}", name, request.args);
    let registry = &handles.registry;
    tracing::trace!("Dispatching invocation request to worker registry");
    Ok(registry.invoke(name, request.args).await?)
}

#[derive(Deserialize)]
pub struct InvokeRequest {
    args: Vec<JsonValue>,
}
