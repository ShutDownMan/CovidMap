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
    FetchError,
}

impl fmt::Display for SnippetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SnippetError::InsertError => write!(f, "Insertion Error"),
            SnippetError::FetchError => write!(f, "Fetch Error"),
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

#[derive(Type, FromRow, Debug, Serialize, Deserialize)]
pub struct DocumentSnippet {
    id: i32,
    id_text_type: i32,
    id_document: i32,
    title: Option<String>,
    snippet_text: Option<String>,
    abstract_text: Option<String>,
}

#[derive(Type, FromRow, Debug, Serialize, Deserialize)]
pub struct TableID {
    id: i32,
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
            Err(SnippetError::FetchError)
        }
    }
}

pub async fn fetch_document_snippet(conn: &State<Pool<Postgres>>, snippet_data: FetchSnippet) -> Result<DocumentSnippet, SnippetError> {
    // create snippet search query
    let fetch_snippet_query_str = r#"
        SELECT
            "document_text"."id",
            "document_text"."text",
            "document_text"."id_text_type",
            "document_text"."id_document"
        FROM "document_text"
        WHERE "id" = $1
        LIMIT 1
    "#;

    // run snippet search query
    let fetch_snippet_query = sqlx::query_as(&*fetch_snippet_query_str)
        .bind(snippet_data.id);

    // getting document data from snippet
    let document_snippet: Snippet = match fetch_snippet_query.fetch_one(&**conn).await {
        Ok(query_result) => {
            query_result
        },
        Err(query_error) => {
            println!("{}", query_error);
            return Err(SnippetError::FetchError)
        }
    };

    // fetch document title
    let fetch_document_title_query_str = r#"
        SELECT *
        FROM "document_text"
        WHERE "id" = $1 AND "id_text_type" = 1
        LIMIT 1
    "#;
    
    // run document title search query
    let fetch_document_title_query = sqlx::query_as(&*fetch_document_title_query_str)
        .bind(document_snippet.id_document);
    
    // getting document title from snippet
    let document_title: Snippet = match fetch_document_title_query.fetch_one(&**conn).await {
        Ok(query_result) => {
            query_result
        },
        Err(query_error) => {
            println!("{}", query_error);
            return Err(SnippetError::FetchError)
        }
    };

    // fetch document abstract
    let fetch_document_abstract_query_str = r#"
        SELECT *
        FROM "document_text"
        WHERE "id" = $1 AND "id_text_type" = 2
        LIMIT 1
    "#;

    // run document abstract search query
    let fetch_document_abstract_query = sqlx::query_as(&*fetch_document_abstract_query_str)
        .bind(document_snippet.id_document);

    // getting document abstract from snippet
    let document_abstract: Option<Snippet> = match fetch_document_abstract_query.fetch_one(&**conn).await {
        Ok(query_result) => {
            Some(query_result)
        },
        Err(query_error) => {
            println!("{}", query_error);
            None
        }
    };

    // creating document snippet and returning it
    Ok(DocumentSnippet {
        id: document_snippet.id,
        id_text_type: document_snippet.id_text_type,
        id_document: document_snippet.id_document,
        title: Some(document_title.text),
        abstract_text: match document_abstract {
            Some(document_abstract) => Some(document_abstract.text),
            None => None
        },
        snippet_text: Some(document_snippet.text),
    })
}
