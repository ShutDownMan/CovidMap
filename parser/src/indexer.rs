use crate::database::{Database, Document};
use crate::transformer::Embedder;
use crate::utils::PgVec;

use std::sync::Arc;
use tokio::sync::Mutex;

use futures::future::join_all;

pub struct Indexer {
	database: Arc<Database>,
	embedder: Arc<Mutex<Embedder>>,
}

impl Indexer {
	pub fn new(database: Arc<Database>, embedder: Arc<Mutex<Embedder>>) -> Indexer {
		Indexer {
			database: database,
			embedder: embedder,
		}
	}

	pub async fn insert_papers_from_csv(
		self,
		csv_path: &str,
	) -> Result<(), Box<dyn std::error::Error>> {
		let csv_reader = csv::Reader::from_path(csv_path)?;

		let mut t_papers = vec![];

		for paper in csv_reader.into_records() {
			if let Ok(paper) = paper {
				let paper_id: String = paper[0].to_string();
				let title: Option<String> = Some(paper[1].to_string());
				let abstract_text: Option<String> = Some(paper[2].to_string());
				let body_text: Option<String> = Some(paper[3].to_string());

				let db = self.database.clone();
				let embedder = self.embedder.clone();

				t_papers.push(tokio::spawn(async move {
					let abstract_embedding = {
						// get embeddings for abstract and body
						Some(
							embedder
								.lock()
								.await
								.embed_sentence(&abstract_text.clone().unwrap()),
						)
					};

					// create document struct
					let document = Document {
						paper_id,
						title,
						abstract_text,
						body_text,
						abstract_embedding,
					};

					// insert document into database
					db.insert_document(document).await.unwrap();
				}));
			}
		}

		join_all(t_papers).await;

		Ok(())
	}
}
