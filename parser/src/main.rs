mod database;
mod search;
mod transformer;
mod utils;

use dotenv;

pub fn main() {
	dotenv::dotenv().expect("Failed to read .env file");

	// let ast = query::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");
	// let ast = search::query::parse(r#" covid "#).unwrap();

	let ast = search::query::parse(
		r#" ("pregnant" OR pregnancy OR maternity) (covid OR Sars-Cov-2) (effects OR disease) "#,
	)
	.unwrap();

	println!("{:#}", ast);

	let pg_query = search::ast_to_query(&ast);

	println!("{:#}", pg_query);

	let mut database = database::Database::new();

	let docs = database.match_query(pg_query);

	println!("{:#?}", docs);

	let mut sentence_transformer = transformer::Embedder::new(&mut database);

	let sem_docs =
		sentence_transformer.semantic_query("what are the effects of coronavirus or covid on pregnant women?");

	println!("{:#?}", sem_docs);
}
