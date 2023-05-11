use super::error::Error as DBError;
use crate::db::{schema::invoke_requests, DBPoolConnection};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use serde_json::Value as JsValue;
use std::time::SystemTime;

#[derive(Selectable, Queryable, Identifiable)]
#[diesel(belongs_to(Function))]
#[diesel(table_name = invoke_requests)]
pub struct InvokeRequest {
    pub id: i32,
    pub created_at: SystemTime,
    pub function_id: i32,
    pub user_addr: String,
    pub payload: Option<JsValue>,
}

#[derive(Insertable)]
#[diesel(table_name = invoke_requests)]
pub struct NewInvokeRequest {
    pub function_id: i32,
    pub user_addr: String,
    pub payload: Option<JsValue>,
}

impl InvokeRequest {
    pub fn new(
        user_addr: String,
        function_id: i32,
        payload: Option<&impl Serialize>,
    ) -> NewInvokeRequest {
        let payload = payload.map(|p| serde_json::to_value(p).unwrap());
        NewInvokeRequest {
            user_addr,
            function_id,
            payload: payload,
        }
    }
}

impl NewInvokeRequest {
    pub async fn insert(&self, db_conn: &mut DBPoolConnection) -> Result<InvokeRequest, DBError> {
        diesel::insert_into(invoke_requests::table)
            .values(self)
            .returning(InvokeRequest::as_returning())
            .get_result(db_conn)
            .await
            .map_err(|e| DBError::DBError(e))
    }
}
