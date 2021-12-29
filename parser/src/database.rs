use postgres::{Client, Error, NoTls};
use std::env;

pub struct Database {
	client: Client,
}

impl Database {
	pub fn new() -> Database {
		Database {
			client: Database::init_database_connection(),
		}
	}

	fn init_database_connection() -> Client {
		let connection_string = format!(
			"dbname={} host={} user={} password={}",
			env::var("DB_DATABASE_NAME").unwrap(),
			env::var("DB_HOST").unwrap(),
			env::var("DB_USER").unwrap(),
			env::var("DB_PASSWORD").unwrap()
		);

		Client::connect(&connection_string, NoTls).unwrap()
	}

	pub fn match_query_documents(&mut self, ts_match: String) -> Vec<Document> {
		let query_template = format!(
			r#"
			SELECT
				ts_rank("tsv", ({0})) AS "rank",
				paper_id,
				title,
				abstract,
				body
			FROM
				papers
			WHERE
				tsv @@ ({0})
			ORDER BY rank DESC LIMIT 20
		"#,
			&ts_match
		);

		let rows = self.client.query(&query_template, &[]).unwrap();
		rows.iter()
			.map(|row| {
				let col_paper_id: String = row.get(1);
				let col_title: Option<String> = row.get(2);
				let col_abstract_text: Option<String> = row.get(3);
				let col_body_text: Option<String> = row.get(4);

				Document {
					paper_id: col_paper_id,
					title: col_title,
					abstract_text: col_abstract_text,
					body_text: col_body_text,
				}
			})
			.collect::<Vec<Document>>()
	}
}

pub struct Document {
	paper_id: String,
	title: Option<String>,
	abstract_text: Option<String>,
	body_text: Option<String>,
}

// impl std::fmt::Debug for Database {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{{{}}}", self.client)
//     }
// }

impl std::fmt::Debug for Document {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let m_paper_id = &self.paper_id;
		let m_title = self.title.clone().unwrap_or("?TITLE?".to_owned());
		let m_abstract_text = self.abstract_text
			.clone()
			.unwrap_or("?ABSTRACT?".to_owned());
		let end = m_abstract_text.chars().map(|c| c.len_utf8()).take(50).sum();

		write!(
			f,
			"{{[{}] '{}': {}...}}",
			&m_paper_id,
			&m_title,
			&m_abstract_text[..end]
		)
	}
}