use rocket::serde::json::{json, Value};
use rocket::State;
use sqlx::{Pool, Postgres};

use crate::services;

#[get("/document?<id>")]
pub async fn get_document_by_paper_id(conn: &State<Pool<Postgres>>, id: String) -> Option<Value> {
    match services::document::fetch_document_by_paper_id(conn, id).await {
        Ok(search_result) => {
            Some(json![search_result])
        },
        Err(_search_error) => {
            // json!({
            //     "message": format!("search failed: {}", search_error.to_string()),
            // })
            None
        }
    }
}