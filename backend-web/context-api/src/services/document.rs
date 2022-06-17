use rocket::serde::{Serialize, Deserialize};
use rocket::State;
use std::fmt;
use sqlx::{Pool, Postgres, FromRow, Type};

#[derive(Debug, Clone, Copy)]
pub enum FetchDocumentError {
    FetchDocumentError,
}

impl fmt::Display for FetchDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FetchDocumentError::FetchDocumentError => write!(f, "Search Error"),
        }
    }
}

#[derive(Type, FromRow, Debug, Serialize, Deserialize)]
pub struct Document {
    pub id: i32,
    pub paper_id: String,
}

pub async fn fetch_document_by_paper_id(conn: &State<Pool<Postgres>>, document_paper_id: String) -> Result<Document, FetchDocumentError> {
    // create search query
    let search_query_str = r#"
        SELECT "id", "paper_id"
        FROM "document"
        WHERE "paper_id" = $1
        LIMIT 1
    "#;

    // run search query
    let search_query = sqlx::query_as(&*search_query_str)
        .bind(document_paper_id);

    return match search_query.fetch_one(&**conn).await {
        Ok(query_result) => {
            Ok(query_result)
        },
        Err(query_error) => {
            println!("{}", query_error);
            Err(FetchDocumentError::FetchDocumentError)
        }
    }
}
