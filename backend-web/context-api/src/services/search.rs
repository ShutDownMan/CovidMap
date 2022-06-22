use rocket::serde::json::{Json};
use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use std::fmt;
use sqlx::{Pool, Postgres, FromRow, Type};
use pgvector::Vector;

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
    pub paper_id: String,
    pub snippet: String,
}

pub async fn search_context(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, search: Json<SearchQuery>) -> Result<Vec<EmbeddingSearchResult>, SearchError> {
    let search_embedding_model = EmbeddingModelType::DistilBERT;
    let search_embedding = embedder.embed_snippet(&search_embedding_model, &search.search_query);

    let search_allowed_snippets = match &search.allowed_snippets {
        Some(allowed_snippets) => allowed_snippets
            .iter()
            .map(|snippet_type| String::from(snippet_type))
            .collect::<Vec<String>>(),
        None => vec![String::from("title"), String::from("abstract")]
    };
    
    let search_limit = match search.limit {
        Some(limit) => limit,
        None => 20
    };

    // create search query
    let search_query_str = r#"
        SELECT
            DISTINCT "embedding"."id_document_text",
            "embedding"."id" as "embedding_id",
            "embedding"."value" <=> $1 as "similarity",
            "model"."name" as "model_name",
            "text_type"."description" as "type_of_text",
            "document"."paper_id" as "paper_id",
            "document_text"."text" as "snippet"
        FROM "embedding"
        JOIN "document_text" ON "embedding"."id_document_text" = "document_text"."id"
        JOIN "text_type" ON "document_text"."id_text_type" = "text_type"."id"
        JOIN "model" ON "embedding"."id_model" = "model"."id"
        JOIN "document" ON "document_text"."id_document" = "document"."id"
        WHERE "model"."name" = $2 AND "text_type"."description" = ANY($3)
        ORDER BY "similarity" ASC
        LIMIT $4
    "#;

    // run search query
    let search_query = sqlx::query_as(&*search_query_str)
        .bind(search_embedding)
        .bind(search_embedding_model.to_string())
        .bind(search_allowed_snippets)
        .bind(search_limit);

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

#[derive(Type, FromRow, Debug, Serialize, Deserialize)]
pub struct EmbeddingFastSearchResult {
    pub id_document_text: i32,
    pub embedding_id: i32,
    pub embedding_vector: Vector,
    pub model_name: String,
    pub type_of_text: String,
    pub paper_id: i32,
    pub snippet: String,
}

pub async fn fast_search_context(conn: &State<Pool<Postgres>>, embedder: &State<Embedder>, search: Json<SearchQuery>) -> Result<Vec<EmbeddingFastSearchResult>, SearchError> {
    let search_embedding_model = EmbeddingModelType::DistilBERT;
    let search_embedding = embedder.embed_snippet(&search_embedding_model, &search.search_query);

    let search_allowed_snippets = match &search.allowed_snippets {
        Some(allowed_snippets) => allowed_snippets
            .iter()
            .map(|snippet_type| String::from(snippet_type))
            .collect::<Vec<String>>(),
        None => vec![String::from("title"), String::from("abstract")]
    };
    
    let search_limit = match search.limit {
        Some(limit) => limit,
        None => 20
    };

    // create search query
    let search_query_str = r#"
        SELECT
            DISTINCT "embedding"."id_document_text",
            "embedding"."id" as "embedding_id",
            "embedding"."value" <=> $1 as "similarity",
        FROM "Search_DistilBERT"
        WHERE "embedding"."id_model" = 1
        ORDER BY "similarity" ASC
        LIMIT $1
    "#;

    // run search query
    let search_query = sqlx::query_as(&*search_query_str)
        .bind(search_limit);

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
