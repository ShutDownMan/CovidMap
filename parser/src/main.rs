mod database;
mod search;
mod transformer;
mod utils;
mod indexer;

use tokio;
use dotenv;

use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
	dotenv::dotenv().expect("Failed to read .env file");

	// let ast = query::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");
	// let ast = search::query::parse(r#" covid "#).unwrap();

	// let ast = search::query::parse(
	// 	r#" ("pregnant" OR pregnancy OR maternity) (covid OR Sars-Cov-2) (effects OR disease) "#,
	// )
	// .unwrap();

	// println!("{:#}", ast);

	// let pg_query = search::ast_to_query(&ast);

	// println!("{:#}", pg_query);

	let mut database = database::Database::new().await.unwrap();
	let database = Arc::new(Mutex::new(database));

	// let docs = database.match_query(pg_query).await;

	// println!("{:#?}", docs);

	let mut sentence_transformer = transformer::Embedder::new(&database);
	let sentence_transformer = Arc::new(Mutex::new(sentence_transformer));

	// let sem_docs =
	// 	sentence_transformer.semantic_query("what are the effects of coronavirus or covid on pregnant women?");

	// println!("{:#?}", sem_docs);

	let mut indexer = indexer::Indexer::new(&database, &sentence_transformer);

	indexer.insert_papers_from_csv("/home/jedi/git-repos/CovidMap/data/small/df_covid_preprocessed.csv").await.unwrap();


}


/*
"what are the effects of coronavirus or covid on pregnant women?"
"what are the coronavirus side effects and tribulations"
"what are the long term effects of corona virus disease Sars-Cov-2"
"how can the coronavirus mutation occour"
"which socioeconomical impacts does the coronavírus have on under developed countries"
"what are the effective medication and safety approaches to coronavírus disease"

*/