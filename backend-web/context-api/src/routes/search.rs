use rocket::serde::json::{Json, json, Value};
use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use sqlx::{Pool, Postgres};

use crate::services;
use crate::embedder::Embedder;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub search_query: String,
    pub limit: Int,
}

#[post("/search/context", format="json", data="<search_data>")]
pub async fn search_handler(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, search_data: Json<SearchQuery>) -> Value {
    let search_result = services::search::search_context(conn, embedder, search_data).await;

    match search_result {
        Ok(_) => {
            search_result
        },
        Err(search_result) => {
            json!({
                "message": format!("search failed: {}", search_result.to_string()),
            })
        }
    }
}