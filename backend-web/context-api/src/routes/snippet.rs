use rocket::serde::json::{Json, json, Value};
use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use sqlx::{Pool, Postgres};

use crate::services;
use crate::embedder::Embedder;

#[derive(Debug, Serialize, Deserialize)]
pub enum TextType {
    Title,
    Abstract,
    BodyText,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertSnippet {
    pub paper_id: String,
    pub text: String,
    pub text_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FetchSnippet {
    pub id: i32
}

#[post("/document/snippet", format="json", data="<snippet_data>")]
pub async fn snippet_post_handler(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, snippet_data: Json<InsertSnippet>) -> Value {
    let snippet_result = services::snippet::insert_snippet(conn, embedder, snippet_data).await;

    match snippet_result {
        Ok(_) => {
            json!({
                "message": "inserted snippet successfully.",
            })
        },
        Err(snippet_result) => {
            json!({
                "message": format!("couldn't insert snippet: {}", snippet_result.to_string()),
            })
        }
    }
}

#[get("/document/snippet?<id>")]
pub async fn snippet_get_handler(conn: &State<Pool<Postgres>>, id: i32) -> Value {
    let snippet_result = services::snippet::fetch_snippet(conn, FetchSnippet { id: id }).await;

    match snippet_result {
        Ok(snippet) => {
            json!(snippet)
        },
        Err(snippet_error) => {
            json!({
                "message": format!("error when fetching snippet: {}", snippet_error.to_string()),
            })
        }
    }
}

#[get("/document/document_snippet?<id>")]
pub async fn document_snippet_get_handler(conn: &State<Pool<Postgres>>, id: i32) -> Value {
    let snippet_result = services::snippet::fetch_document_snippet(conn, FetchSnippet { id: id }).await;

    match snippet_result {
        Ok(snippet) => {
            json!(snippet)
        },
        Err(snippet_error) => {
            json!({
                "message": format!("error when fetching snippet: {}", snippet_error.to_string()),
            })
        }
    }
}