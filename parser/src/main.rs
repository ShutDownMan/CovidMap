mod database;
mod indexer;
mod search;
mod transformer;
mod utils;

use dotenv;
use tokio;

use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
	dotenv::dotenv().expect("Failed to read .env file");

	// let ast = search::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");
	// let ast = search::query::parse(r#" covid "#).unwrap();

	let ast = search::query::parse(
		r#" ("pregnant" OR pregnancy OR maternity) (covid OR Sars-Cov-2) (effects OR disease) "#,
	)
	.unwrap();

	println!("{:#}", ast);

	let pg_query = search::ast_to_query(&ast);

	println!("{:#}", pg_query);

	let mut database = database::Database::new().await.unwrap();
	let database = Arc::new(Mutex::new(database));

	let x = database.clone();
	let docs = x.lock().await.match_query(pg_query).await;
	drop(x);

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

	// let mut indexer = indexer::Indexer::new(&database.clone(), &sentence_transformer.clone());

	// indexer.insert_papers_from_csv("/home/jedi/git-repos/CovidMap/data/full/df_covid_preprocessed.csv").await.unwrap();
}

/*
"what are the effects of coronavirus or covid on pregnant women?"
"what are the coronavirus side effects and tribulations"
"what are the long term effects of corona virus disease Sars-Cov-2"
"how can the coronavirus mutation occour"
"which socioeconomical impacts does the coronavírus have on under developed countries"
"what are the effective medication and safety approaches to coronavírus disease"

*/
