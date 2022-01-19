
mod database;
//mod indexer;
mod search;
mod transformer;
mod utils;

use dotenv;
use tokio;

use std::sync::Arc;

#[tokio::main]
async fn main() {
	dotenv::dotenv().expect("Failed to read .env file");

	let ast = search::query::parse(
		r#" ("pregnant" OR pregnancy OR maternity) (covid OR Sars-Cov-2) (effects OR disease) "#,
	)
	.unwrap();

	let pg_query = search::ast_to_query(& ast);

	let mut database = database::Database::new().await.unwrap();
	let db = Arc::new(database);

	// let docs = db.match_query(pg_query).await;

	// println!("{:#?}", docs);

	let mut embedder = transformer::Embedder::new(db.clone());

	let docs = (& embedder).semantic_query("what are the effects of coronavirus or covid on pregnant women?").await;

	println!("{:#?}", docs);
}

/*
"what are the effects of coronavirus or covid on pregnant women?"
"what are the coronavirus side effects and tribulations"
"what are the long term effects of corona virus disease Sars-Cov-2"
"how can the coronavirus mutation occour"
"which socioeconomical impacts does the coronavírus have on under developed countries"
"what are the effective medication and safety approaches to coronavírus disease"

// let ast = search::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");

*/
