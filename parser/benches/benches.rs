#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use std::sync::Arc;
use tokio::sync::Mutex;

use parser::*;

use tokio::runtime::Runtime;

#[bench]
fn test_query_ast(b: &mut Bencher) {
    b.iter(|| {
        let ast = search::query::parse(
            r#" ("pregnant" OR pregnancy OR maternity) (covid OR Sars-Cov-2) (effects OR disease) "#,
        )
        .unwrap();

        // println!("{:#}", ast);

        let pg_query = search::ast_to_query(&ast);

        // println!("{:#}", pg_query);
    });
}

#[async_std::bench]
async fn test_match_query(b: &mut Bencher) {
    dotenv::dotenv().expect("Failed to read .env file");

    let ast = search::query::parse(
        r#" ("pregnant" OR pregnancy OR maternity) (covid OR Sars-Cov-2) (effects OR disease) "#,
    )
    .unwrap();

    let pg_query = search::ast_to_query(&ast);

    let mut database = database::Database::new().await.unwrap();
    let database = Arc::new(Mutex::new(database));

    let x = database.clone();
    let docs = x.lock().await.match_query(pg_query).await;

    b.iter(|| {

    });
}

#[tokio::test]
async fn test_semantic_query() {
    dotenv::dotenv().expect("Failed to read .env file");

    let mut database = database::Database::new().await.unwrap();
    let database = Arc::new(Mutex::new(database));

    let mut sentence_transformer = transformer::Embedder::new(&database.clone());
    let sentence_transformer = Arc::new(Mutex::new(sentence_transformer));

    let x = sentence_transformer.clone();
    let sem_docs = x
        .lock()
        .await
        .semantic_query("what are the effects of coronavirus or covid on pregnant women?")
        .await;

    drop(x);

    println!("{:#?}", sem_docs);
}

#[tokio::test]
async fn test_indexer() {
    dotenv::dotenv().expect("Failed to read .env file");

    let mut database = database::Database::new().await.unwrap();
    let database = Arc::new(Mutex::new(database));

    let mut sentence_transformer = transformer::Embedder::new(&database.clone());
    let sentence_transformer = Arc::new(Mutex::new(sentence_transformer));

    let mut indexer = indexer::Indexer::new(&database.clone(), &sentence_transformer.clone());

    // indexer
    // 	.insert_papers_from_csv(
    // 		"/home/jedi/git-repos/CovidMap/data/small/df_covid_preprocessed.csv",
    // 	)
    // 	.await
    // 	.unwrap();
}
