use rocket::serde::json::{Json};
use rocket::State;
use std::fmt;
use sqlx::{Pool, Postgres};

use crate::routes::snippet::Snippet;
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

pub async fn insert_snippet(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, snippet: Json<Snippet>) -> Result<(), SnippetError> {
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
