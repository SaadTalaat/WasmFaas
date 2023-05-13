use crate::status::{Status, StatusKind};
use axum::{
    body::Bytes,
    extract::{ConnectInfo, FromRequest, FromRequestParts},
    http::{
        header::{CONTENT_TYPE},
        request::Parts,
        Request
    },
    response::{IntoResponse, Response},
    Json,
};
use std::net::{SocketAddr};

pub struct WasmRepr<T>(pub T);

#[axum::async_trait]
impl<S, B, T> FromRequest<S, B> for WasmRepr<T>
where
    S: Send + Sync,
    B: Send + 'static,
    Json<T>: FromRequest<S, B>,
    Bytes: FromRequest<S, B>,
    T: 'static + From<Bytes> + From<String>,
{
    type Rejection = Response;

    async fn from_request(request: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let content_type_header: Option<&str> = request
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|t| t.to_str().ok());

        if let Some(content_type) = content_type_header {
            if content_type.starts_with("application/json") {
                let Json(payload) = Json::from_request(request, state)
                    .await
                    .map_err(IntoResponse::into_response)?;
                Ok(Self(payload))
            } else if content_type.starts_with("application/wasm") {
                let payload = Bytes::from_request(request, state)
                    .await
                    .map_err(IntoResponse::into_response)?;
                Ok(Self(T::from(payload)))
            } else if content_type.starts_with("application/text") {
                let payload = Bytes::from_request(request, state)
                    .await
                    .map_err(IntoResponse::into_response)?;
                let payload = String::from_utf8(payload.to_vec()).map_err(|err| {
                    Status::new(
                        StatusKind::UnsupportedMediaType,
                        format!("malformed payload {err}"),
                    )
                    .into_response()
                })?;
                Ok(Self(T::from(payload)))
            } else {
                let msg = format!("unsupported content type: {}", content_type);
                Err(Status::new(StatusKind::UnsupportedMediaType, msg).into_response())
            }
        } else {
            let msg = format!("Content-Type must be provided");
            Err(Status::new(StatusKind::UnsupportedMediaType, msg).into_response())
        }
    }
}

pub struct RemoteAddress(pub String);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RemoteAddress
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let maybe_addr = parts
            .headers
            .get("X-Forwarded-For")
            .and_then(|v| v.to_str().ok());
        match maybe_addr {
            Some(addr) => {
                Ok(RemoteAddress(addr.to_owned()))
            }
            None => match parts.extensions.get::<ConnectInfo<SocketAddr>>() {
                Some(socket_addr) => {
                    let addr = format!("{}", socket_addr.ip());
                    Ok(RemoteAddress(addr))
                }
                None => {
                    let msg = format!("Unrecognized remote host");
                    Err(Status::new(StatusKind::BadRequest, msg).into_response())
                }
            },
        }
    }
}
