pub mod models;
pub mod schema;

use crate::{state::AppState, status::Status};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use diesel_async::{pooled_connection::bb8, AsyncPgConnection};

pub type DBPool = bb8::Pool<AsyncPgConnection>;
pub type DBPoolConnection = bb8::PooledConnection<'static, AsyncPgConnection>;

pub struct DBConnection(pub DBPoolConnection);

#[async_trait]
impl FromRequestParts<AppState> for DBConnection {
    type Rejection = Status;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let pool = state.pool;
        let conn = pool
            .get_owned()
            .await
            .map_err(|e| Status::error(e.to_string()))?;
        Ok(Self(conn))
    }
}
