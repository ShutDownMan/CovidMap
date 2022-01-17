use crate::database::{Database, Document};
use crate::transformer::Embedder;
use crate::utils::PgVec;

use std::sync::Arc;
use tokio::sync::Mutex;

use futures::future::join_all;

pub struct Indexer {
	database: Arc<Mutex<Database>>,
	embedder: Arc<Mutex<Embedder>>,
}

impl Indexer {
	pub fn new(database: &Arc<Mutex<Database>>, embedder: &Arc<Mutex<Embedder>>) -> Indexer {
		Indexer {
			database: database.clone(),
			embedder: embedder.clone(),
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

				let db = Arc::clone(&self.database);
				let embedder = Arc::clone(&self.embedder);

				t_papers.push(tokio::spawn(async move {

					// get hold of the embedder
					let embedder_lock = embedder.lock().await;

					// get embeddings for abstract and body
					let abstract_embedding =
						Some(embedder_lock.embed_sentence(&abstract_text.clone().unwrap()));

					// drop embedder as it can be used by the other threads
					drop(embedder_lock);

					// create document struct
					let document = Document {
						paper_id,
						title,
						abstract_text,
						body_text,
						abstract_embedding
					};
					// get a hold of the database
					let db_lock = db.lock().await;

					// insert document into database
					db_lock.insert_document(document).await.unwrap();
					drop(db_lock);
				}));
			}
		}

		join_all(t_papers).await;

		Ok(())
	}
}
