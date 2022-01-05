// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_mut)]
// #![allow(dead_code)]

use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::{Client, Connection, Error, NoTls, Socket};

use std::env;

use crate::utils::PgVec;

pub struct Database {
	client: Client,
}

impl Database {
	pub async fn new() -> Result<Database, Error> {
		let connection_string = format!(
			"dbname={} host={} user={} password={}",
			env::var("DB_DATABASE_NAME").unwrap(),
			env::var("DB_HOST").unwrap(),
			env::var("DB_USER").unwrap(),
			env::var("DB_PASSWORD").unwrap()
		);

		let (client, connection) = tokio_postgres::connect(&connection_string, NoTls).await?;
		// The connection object performs the actual communication with the database,
		// so spawn it off to run on its own.
		tokio::spawn(async move {
			if let Err(e) = connection.await {
				eprintln!("connection error: {}", e);
			}
		});

		Ok(Database { client: client })
	}

	async fn init_database_connection() {}

	pub async fn match_query(&mut self, ts_match: String) -> Vec<Document> {
		let limit: Option<u32> = Some(20);

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
				ORDER BY rank DESC LIMIT {1}
			"#,
			&ts_match,
			limit.unwrap_or(20),
		);

		let rows = self.client.query(&query_template, &[]).await.unwrap();

		// println!("{:#?}", rows);
		rows.iter()
			.map(|row| {
				let col_paper_id: String = row.get("paper_id");
				let col_title: String = row.get("title");
				let col_abstract: String = row.get("abstract");
				let col_body: String = row.get("body");

				Document {
					paper_id: col_paper_id,
					title: Some(col_title),
					abstract_text: Some(col_abstract),
					body_text: Some(col_body),
				}
			})
			.collect::<Vec<Document>>()
	}

	pub async fn find_similar_documents_by_embedding(
		&self,
		embedding: PgVec,
		limit: Option<u32>,
	) -> Vec<Document> {
		let query_template = r#"
			SELECT
				DISTINCT paper_id,
				$1 <=> abstract_embedding AS similarity,
				title,
				abstract,
				body
			FROM
				papers
			ORDER BY similarity ASC LIMIT $2;
		"#;

		// println!("{:#?}", query_template);

		let rows = self
			.client
			.query(
				query_template,
				&[&embedding.to_string(), &limit.unwrap_or(20).to_string()],
			)
			.await
			.unwrap();
		rows.iter()
			.filter_map(|row| {
				// let col_similarity: f64 = row.get("similarity");
				let col_paper_id: String = row.get("paper_id");
				let col_title: String = row.get("title");
				let col_abstract: String = row.get("abstract");
				let col_body: String = row.get("body");
				// println!("{:#?}", col_similarity);

				// self.get_paper_by_id(col_paper_id.unwrap().as_str())

				Some(Document {
					paper_id: col_paper_id,
					title: Some(col_title),
					abstract_text: Some(col_abstract),
					body_text: Some(col_body),
				})
			})
			.collect::<Vec<Document>>()
	}

	pub async fn _get_paper_by_id(&mut self, paper_id: &str) -> Option<Document> {
		let query_template = format!(
			r#"
			SELECT
				*
			FROM
				papers
			WHERE
				paper_id = '{0}'
			;
		"#,
			paper_id
		);

		// println!("{:#?}", paper_id);

		self.client
			.query(&query_template, &[])
			.await
			.ok()?
			.first()
			.map(|row| {
				let col_paper_id: String = row.get("paper_id");
				let col_title: Option<String> = row.get("title");
				Document {
					paper_id: col_paper_id,
					title: col_title,
					abstract_text: None,
					body_text: None,
				}
			})
	}
}

pub struct Document {
	paper_id: String,
	title: Option<String>,
	abstract_text: Option<String>,
	body_text: Option<String>,
}

impl std::fmt::Debug for Document {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let m_paper_id = self.paper_id.clone();
		let m_title = self.title.clone().unwrap_or("?TITLE?".to_owned());
		let m_abstract_text = self
			.abstract_text
			.clone()
			.unwrap_or("?ABSTRACT?".to_owned());
		let end = m_abstract_text
			.chars()
			.map(|c| c.len_utf8())
			.take(150)
			.sum();

		write!(
			f,
			"{{[{}] '{}': {}...}}",
			&m_paper_id,
			&m_title,
			&m_abstract_text[..end]
		)
	}
}

pub struct Paragraph {
	paper_id: String,
	text: String,
}

impl std::fmt::Debug for Paragraph {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let m_paper_id = self.paper_id.clone();
		let m_text = self.text.clone();
		let end = m_text.chars().map(|c| c.len_utf8()).take(150).sum();

		write!(f, "{{[{}]: {}}}", &m_paper_id, &m_text[..end])
	}
}
