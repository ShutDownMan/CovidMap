// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_mut)]
// #![allow(dead_code)]

use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::{Client, Connection, Error, NoTls, Socket};

use std::env;

use crate::utils::PgVec;

pub struct Database {
	pub client: Client,
}

impl std::fmt::Debug for Database {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:#?}", self.client)
	}
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
			.filter_map(|row| self.get_document_from_row(row).ok())
			.collect::<Vec<Document>>()
	}

	fn get_document_from_row(
		&self,
		row: &tokio_postgres::Row,
	) -> Result<Document, Box<dyn std::error::Error>> {
		let col_paper_id: String = row.try_get("paper_id")?;
		let col_title: String = row.try_get("title")?;
		let col_abstract: String = row.try_get("abstract")?;
		let col_body: String = row.try_get("body")?;
		let col_abstract_embedding: Vec<f32> = row.try_get("abstract_embedding")?;
		let col_body_embedding: Vec<f32> = row.try_get("body_embedding")?;

		Ok(Document {
			paper_id: col_paper_id,
			title: Some(col_title),
			abstract_text: Some(col_abstract),
			body_text: Some(col_body),
			abstract_embedding: Some(PgVec(col_abstract_embedding)),
			body_embedding: Some(PgVec(col_body_embedding)),
		})
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
			.filter_map(|row| self.get_document_from_row(row).ok())
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

		let rows = self.client.query(&query_template, &[]).await.ok()?;

		self.get_document_from_row(rows.first()?).ok()
	}

	pub async fn insert_document(
		&mut self,
		document: &Document,
	) -> Result<(), Box<dyn std::error::Error>> {
		// let query_template = format!(
		// 	r#"
		// 	INSERT INTO papers
		// 		(paper_id, title, abstract, body, abstract_embedding, body_embedding)
		// 	VALUES
		// 		('{0:}', '{1:}', '{2:}', '{3:}', {4:}, {5:})
		// 	ON CONFLICT ON CONSTRAINT papers_pkey DO UPDATE SET
		// 		title = '{1:}',
		// 		abstract = '{2:}',
		// 		body = '{3:}',
		// 		abstract_embedding = {4:},
		// 		body_embedding = {5:}
		// "#,
		// );
		let query_template = r#"
			INSERT INTO papers
				(paper_id, title, abstract, body, abstract_embedding, body_embedding)
			VALUES
				($1, $2, $3, $4, $5, $6)
			ON CONFLICT ON CONSTRAINT papers_pkey DO UPDATE SET
				title = $2,
				abstract = $3,
				body = $4,
				abstract_embedding = $5,
				body_embedding = $6
		"#;

		// println!("{}", &document.title.clone().unwrap_or(String::from("")));
		println!("{}", query_template);

		self.client
			.query(
				query_template,
				&[
					&document.paper_id.to_string(),
					&document.title.clone().unwrap_or(String::from("?TITLE?")),
					&document
						.abstract_text
						.clone()
						.unwrap_or(String::from("?ABSTRACT?")),
					&document.body_text.clone().unwrap_or(String::from("?BODY?")),
					&document.abstract_embedding.as_ref().unwrap(),
					&document.body_embedding.as_ref().unwrap(),
				],
			)
			.await?;

		println!("{:#?}", document);

		Ok(())
	}
}

pub struct Document {
	pub paper_id: String,
	pub title: Option<String>,
	pub abstract_text: Option<String>,
	pub body_text: Option<String>,
	pub abstract_embedding: Option<PgVec>,
	pub body_embedding: Option<PgVec>,
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
