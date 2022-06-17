use rocket::serde::json::{Json};
use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use std::fmt;
use sqlx::{Pool, Postgres, FromRow, Type};

use crate::routes::search::SearchQuery;
use crate::embedder::{Embedder, EmbeddingModelType};

#[derive(Debug, Clone, Copy)]
pub enum SearchError {
    SearchError,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SearchError::SearchError => write!(f, "Search Error"),
        }
    }
}

#[derive(Type, FromRow, Debug, Serialize, Deserialize)]
pub struct EmbeddingSearchResult {
    pub embedding_id: i32,
    pub similarity: f64,
    pub id_document_text: i32,
}

pub async fn search_context(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, search: Json<SearchQuery>) -> Result<Vec<EmbeddingSearchResult>, SearchError> {
    let search_embedding_model = EmbeddingModelType::DistilBERT;
    let search_embedding = embedder.embed_snippet(&search_embedding_model, &search.search_query);

    // create search query
    let search_query_str = r#"
        SELECT DISTINCT "id_document_text", "id" as "embedding_id", $1 <=> "value" as similarity
        FROM "embedding"
        WHERE "id_model" = 1
        ORDER BY similarity ASC
        LIMIT $2
    "#;

    // run search query
    let search_query = sqlx::query_as(&*search_query_str)
        .bind(search_embedding)
        // .bind(search_embedding_model.to_string())
        .bind(search.limit);

    return match search_query.fetch_all(&**conn).await {
        Ok(query_result) => {
            Ok(query_result)
        },
        Err(query_error) => {
            println!("{}", query_error);
            Err(SearchError::SearchError)
        }
    }
}
