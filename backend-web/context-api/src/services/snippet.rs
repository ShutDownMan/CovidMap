use rocket::serde::json::{Json};
use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use std::fmt;
use sqlx::{Pool, Postgres, Type, FromRow};

use crate::routes::snippet::{InsertSnippet, FetchSnippet};
use crate::embedder::{Embedder, EmbeddingModelType};

#[derive(Debug, Clone, Copy)]
pub enum SnippetError {
    InsertError,
}

impl fmt::Display for SnippetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SnippetError::InsertError => write!(f, "Insertion Error"),
        }
    }
}

#[derive(Type, FromRow, Debug, Serialize, Deserialize)]
pub struct Snippet {
    id: i32,
    text: String,
    id_text_type: i32,
    id_document: i32,
}

pub async fn insert_snippet(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, snippet: Json<InsertSnippet>) -> Result<(), SnippetError> {
    let snippet_embedding_model = EmbeddingModelType::DistilBERT;
    let snippet_embedding = embedder.embed_snippet(&snippet_embedding_model, &snippet.text);

    // create insert query
    let insert_snippet_query_str = r#"
        CALL insert_snippet($1, $2, $3, $4, $5);
    "#;

    // run insert query
    let insert_snippet_query = sqlx::query(&*insert_snippet_query_str)
        .bind(&snippet.text_type)
        .bind(&snippet.paper_id)
        .bind(&snippet.text)
        .bind(snippet_embedding_model.to_string())
        .bind(snippet_embedding);

    let result = insert_snippet_query.execute(&**conn);

    return match result.await {
        Ok(_) => {
            Ok(())
        },
        Err(query_error) => {
            println!("{}", query_error);
            Err(SnippetError::InsertError)
        }
    }
}

pub async fn fetch_snippet(conn: &State<Pool<Postgres>>, snippet_data: FetchSnippet) -> Result<Snippet, SnippetError> {
    // create search query
    let fetch_snippet_query_str = r#"
        SELECT "id", "text", "id_text_type", "id_document"
        FROM "document_text"
        WHERE "id" = $1
        LIMIT 1
    "#;

    // run search query
    let fetch_snippet_query = sqlx::query_as(&*fetch_snippet_query_str)
        .bind(snippet_data.id);

    return match fetch_snippet_query.fetch_one(&**conn).await {
        Ok(query_result) => {
            Ok(query_result)
        },
        Err(query_error) => {
            println!("{}", query_error);
            Err(SnippetError::InsertError)
        }
    }
}
