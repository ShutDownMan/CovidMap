use rocket::serde::json::{Json, json, Value};
use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use sqlx::{Pool, Postgres};

use crate::services;
use crate::embedder::Embedder;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub search_query: String,
    pub limit: Option<i32>,
    pub model: Option<String>,
    pub allowed_snippets: Option<Vec<String>>,
}

#[post("/search/context", format="json", data="<search_data>")]
pub async fn search_handler(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, search_data: Json<SearchQuery>) -> Value {
    match services::search::search_context(conn, embedder, search_data).await {
        Ok(search_result) => {
            json![search_result]
        },
        Err(search_error) => {
            json!({
                "message": format!("search failed: {}", search_error.to_string()),
            })
        }
    }
}

#[post("/search/context_fast", format="json", data="<search_data>")]
pub async fn search_handler(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, search_data: Json<SearchQuery>) -> Value {
    match services::search::fast_search_context(conn, embedder, search_data).await {
        Ok(search_result) => {
            json![search_result]
        },
        Err(search_error) => {
            json!({
                "message": format!("search failed: {}", search_error.to_string()),
            })
        }
    }
}