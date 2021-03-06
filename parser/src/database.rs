// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_mut)]
// #![allow(dead_code)]

use futures::future;
use tokio_postgres::{Client, Error, NoTls};

use json::{array, object, JsonValue};
use std::env;

// use crate::utils::PgVec;
use pgvector::Vector;

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

	pub async fn match_query(&self, ts_match: String) -> Vec<Document> {
		let limit: Option<u32> = Some(20);

		let query_template = format!(
			r#"
				SELECT
					ts_rank("tsv", ({0})) AS "rank",
					paper_id,
					title,
					abstract,
					body,
					abstract_embedding,
					body_embedding
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
		// let col_abstract_embedding: Vec<f32> = row.try_get("abstract_embedding")?;

		Ok(Document {
			paper_id: col_paper_id,
			title: Some(col_title),
			abstract_text: Some(col_abstract),
			body_text: None,
			// abstract_embedding: Some(PgVec(col_abstract_embedding))
			abstract_embedding: None,
		})
	}

	pub async fn get_papers_by_embedding(
		&self,
		embedding: pgvector::Vector,
		limit: Option<u32>,
	) -> Vec<Document> {
		let query_template = format!(
			r#"
			SELECT
				DISTINCT paper_id,
				$1 <=> abstract_embedding AS similarity,
				title,
				abstract,
				body
			FROM
				papers
			ORDER BY similarity ASC LIMIT $2;
		"#
		);

		// println!("{:#?}", query_template);

		let rows = self
			.client
			.query(&query_template, &[&embedding, &limit.unwrap_or(20)])
			.await
			.unwrap();
		rows.iter()
			.filter_map(|row| self.get_document_from_row(row).ok())
			.collect::<Vec<Document>>()
	}

	pub async fn get_paper_by_id(&self, paper_id: &str) -> Option<Document> {
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

	pub async fn get_papers_by_ids(&self, paper_ids: Vec<&str>) -> Vec<Document> {
		let tasks: Vec<_> = paper_ids
			.iter()
			.map(|paper_id| self.get_paper_by_id(paper_id))
			.collect();

		future::join_all(tasks)
			.await
			.into_iter()
			.filter_map(|x| x)
			.collect()
	}

	pub async fn insert_abstract(
		&self,
		document: Document,
	) -> Result<(), Box<dyn std::error::Error>> {
		let paper_id: String = document.paper_id;
		let title: String = document.title.unwrap_or(String::from("?TITLE?"));
		let abstract_text: String = document.abstract_text.unwrap_or(String::from("?ABSTRACT?"));
		// let body_text: String = document.body_text.unwrap_or(String::from("?BODY?"));
		// let title_embedding: Vector = document.title_embedding.unwrap_or(Vector::from(vec![]));
		let abstract_embedding: Vector =
			document.abstract_embedding.unwrap_or(Vector::from(vec![]));
		// let body_embedding: Vector = document.body_embedding.unwrap_or(Vector(vec![]));

		// inserting on document table
		let query_document = format!(
			r#"
			INSERT INTO document
				(title, pmc_id)
			VALUES
				($1, $2)
			ON CONFLICT ON CONSTRAINT document_pk DO UPDATE SET
				title = $1,
				pmc_id = $2
			RETURNING (id)
			;"#
		);

		let insert_result = self
			.client
			.query(&query_document, &[&title, &paper_id])
			.await
			.unwrap();

		let id_document: i32 = insert_result[0].get("id");
		println!("ID: {}", id_document);
		let id_text_type = 1;

		// inserting on document_text table
		let query_document_text = format!(
			r#"
			INSERT INTO document_text
				(text, id_document, id_text_type)
			VALUES
				($1, $2, $3)
			ON CONFLICT ON CONSTRAINT document_text_pk DO UPDATE SET
				text = $1,
				id_document = $2,
				id_text_type = $3
			RETURNING (id)
			;"#
		);

		let insert_result = self
			.client
			.query(
				&query_document,
				&[&abstract_text, &id_document, &id_text_type],
			)
			.await
			.unwrap();

		let value = abstract_embedding;
		let id_model = 1;
		let id_document_text: i32 = insert_result[0].get("id");

		// inserting on embedding table
		let query_embedding = format!(
			r#"
			INSERT INTO embedding
				(value, id_model, id_document_text)
			VALUES
				($1, $2, $3)
			ON CONFLICT ON CONSTRAINT embedding_pk DO UPDATE SET
				value = $1,
				id_model = $2,
				id_document_text = $3
			RETURNING (id)
			;"#
		);

		self.client
			.query(&query_document, &[&value, &id_model, &id_document_text])
			.await
			.unwrap();

		// println!("{}", &document.title.clone().unwrap_or(String::from("")));
		// println!("{}", query_template);

		println!("{:#?}", paper_id);

		// println!("{:#?}", title);

		Ok(())
	}

	pub async fn get_all_embeddings(&self) -> Vec<(String, f64)> {
		let query_template = format!(
			r#"
			SELECT
				paper_id,
				abstract_embedding
			FROM
				papers
			;"#
		);

		// println!("{:#?}", paper_id);

		let rows = self.client.query(&query_template, &[]).await.ok().unwrap();

		rows.iter()
			.map(|row| (row.get("paper_id"), row.get("abstract_embedding")))
			.collect()
	}
}

pub trait Json {
	fn to_json(&self) -> json::JsonValue;
}

pub struct Document {
	pub paper_id: String,
	pub title: Option<String>,
	pub abstract_text: Option<String>,
	pub body_text: Option<Vec<String>>,
	pub abstract_embedding: Option<pgvector::Vector>,
}

impl Json for Document {
	fn to_json(&self) -> json::JsonValue {
		let m_paper_id = self.paper_id.clone();
		let m_title = self.title.clone().unwrap_or("?TITLE?".to_owned());
		let m_abstract_text = self
			.abstract_text
			.clone()
			.unwrap_or("?ABSTRACT?".to_owned());
		let m_body_text = self.body_text.clone().unwrap_or(vec![]);

		object! {
			paper_id: m_paper_id,
			title: m_title,
			abstract_text: m_abstract_text,
			// body_text: m_body_text
		}
	}
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

impl std::fmt::Display for Document {
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

impl std::fmt::Display for Paragraph {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let m_paper_id = self.paper_id.clone();
		let m_text = self.text.clone();
		let end = m_text.chars().map(|c| c.len_utf8()).take(150).sum();

		write!(f, "{{[{}]: {}}}", &m_paper_id, &m_text[..end])
	}
}
