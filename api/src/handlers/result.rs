use crate::{
    db::models::error::Error as DBError,
    registry::BackendError,
    status::{Status, StatusKind},
    util::{compiler::CompileError, wasm::WasmError},
};
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type APIResult = Result<Status, APIError>;

#[derive(Debug, Error)]
pub enum APIError {
    #[error("{0}")]
    WasmError(#[from] WasmError),
    #[error("{0}")]
    DBError(#[from] DBError),
    #[error("{0}")]
    RegistryError(#[from] BackendError),
    #[error("called identifier is not a function")]
    NotAFunction,
    #[error("{0}")]
    CompileError(#[from] CompileError),
    #[error("internal error")]
    InternalError,
}

impl Into<Status> for APIError {
    fn into(self) -> Status {
        let msg = format!("{}", self);
        let kind = match self {
            Self::DBError(DBError::NotFound) => StatusKind::NotFound,
            Self::InternalError => StatusKind::InternalError,
            _ => StatusKind::BadRequest,
        };
        Status::new(kind, msg)
    }
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let status: Status = self.into();
        status.into_response()
    }
}
