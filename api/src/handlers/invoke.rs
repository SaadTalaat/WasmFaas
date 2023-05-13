use super::result::APIResult;
use crate::{
    db::models::{Function, InvokeRequest},
    extensions::Handles,
    registry::BackendError,
    state::AppState,
    extract::RemoteAddress,
};
use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::Value as JsValue;

pub async fn invoke(
    Extension(handles): Extension<Handles>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    RemoteAddress(addr): RemoteAddress,
    Json(request): Json<UserInvokeRequest>,
) -> APIResult {
    let mut db_conn = state
        .get_db_conn()
        .await
        .map_err(|_| BackendError::NoReply)?;
    // Fetch function
    let func = Function::get(id, &mut db_conn).await?;
    func.validate_args(&request.args)?;
    // Record request
    InvokeRequest::new(addr, func.id, Some(&request.args))
        .insert(&mut db_conn)
        .await?;
    tracing::trace!(
        "Invoking function: {} with args: {:?}",
        &func.name,
        request.args
    );
    let registry = &handles.registry;
    tracing::trace!("Dispatching invocation request to worker registry");
    Ok(registry
        .invoke(func.name, func.user_uri, func.signature, request.args)
        .await?)
}

#[derive(Deserialize)]
pub struct UserInvokeRequest {
    args: Vec<JsValue>,
}
