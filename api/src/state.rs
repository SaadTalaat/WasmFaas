use crate::db::DBPool;
use axum::extract::FromRef;
use diesel_async::{
    pooled_connection::{
        bb8::{PooledConnection, RunError},
    },
    AsyncPgConnection,
};
#[derive(FromRef, Clone)]
pub struct AppState {
    pub pool: DBPool,
}

impl AppState {
    pub fn new(pool: DBPool) -> Self {
        Self { pool }
    }

    pub async fn get_db_conn<'a>(&self) -> Result<PooledConnection<'a, AsyncPgConnection>, RunError> {
        self.pool.get_owned().await
    }
}
