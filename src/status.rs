use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value as JsValue;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusKind {
    Ok,
    BadRequest,
    InternalError,
    NotFound,
    Forbidden,
}

impl StatusKind {
    fn as_http(&self) -> StatusCode {
        match self {
            Self::Ok => StatusCode::OK,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
        }
    }
}

#[derive(Serialize)]
pub struct Status {
    kind: StatusKind,
    message: JsValue,
}

impl Status {
    pub fn new(kind: StatusKind, message: impl Serialize) -> Self {
        Self {
            kind,
            message: serde_json::to_value(message).unwrap(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusKind::Ok, "success".to_owned())
    }

    pub fn ok_payload(payload: impl Serialize) -> Self {
        Self::new(StatusKind::Ok, payload)
    }

    pub fn error(message: String) -> Self {
        Self::new(StatusKind::InternalError, message)
    }
}

impl IntoResponse for Status {
    fn into_response(self) -> Response {
        let status_code: StatusCode = self.kind.as_http();
        (status_code, Json(self)).into_response()
    }
}
