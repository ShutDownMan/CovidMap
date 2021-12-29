mod database;
mod search;
mod transformer;

use tch::Device;

use dotenv;

use rust_bert::pipelines::question_answering::{QaInput, QuestionAnsweringModel};

pub fn main() {
	dotenv::dotenv().expect("Failed to read .env file");

	// let ast = query::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");
	let ast = search::query::parse(
		r#" "pregnant" OR pregnancy (covid OR Sars-Cov-2) (trials OR tests OR experiment) "#,
	)
	.unwrap();
	// let ast = search::query::parse(r#" covid "#).unwrap();
	println!("{:#}", ast);

	let pg_query = search::ast_to_query(&ast);

	println!("{:#}", pg_query);

	// let mut database = database::Database::new();

	// let docs = database.match_query_documents(pg_query);

	// let mut sentence_transformer = transformer::Embedder::new();

	let qa_model = QuestionAnsweringModel::new(Default::default()).unwrap();

	let question = String::from("What are the effects of covid on pregnant women ?");
	let context = String::from("the coronavírus debilitates and causes nausea in pregnancy");
	
	let answers = qa_model.predict(&vec![QaInput { question, context }], 1, 32);

	println!("{:#?}", answers);
	println!("{:}", Device::cuda_if_available().is_cuda());

	// let docs = sentence_transformer.semantic_query_documents(pg_query);

	// println!("{:#?}", docs);
}
