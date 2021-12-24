mod search;

pub fn main() {
	// let ast = query::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");
	let ast = search::query::parse(r#" "pregnant" OR pregnancy (covid OR Sars-Cov-2) (trials OR tests OR experiment) "#).unwrap();
	println!("{:#}", ast);

	let pg_query = search::ast_to_query(&ast);

	println!("( {:#} )", pg_query);

	// let docs = database::query_documents(pg_query);

	// println!("{:#}", docs);
}