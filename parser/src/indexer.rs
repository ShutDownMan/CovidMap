use crate::database::{Database, Document};
use crate::transformer::Embedder;
use crate::utils::PgVec;

use std::sync::Arc;
use tokio::sync::Mutex;

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

		for paper in csv_reader.into_records() {
			if let Ok(paper) = paper {
				let paper_id: String = paper[0].to_string();
				let title: Option<String> = Some(paper[1].to_string());
				let abstract_text: Option<String> = Some(paper[2].to_string());
				let body_text: Option<String> = Some(paper[3].to_string());

				let db = Arc::clone(&self.database);
				let embedder = Arc::clone(&self.embedder);
				tokio::spawn(async move {
					let embedder = embedder.lock().await;

					let abstract_embedding = Some(embedder.embed_sentence(&abstract_text.clone().unwrap()));
					let body_embedding = Some(embedder.embed_sentence(&body_text.clone().unwrap()));
	
					drop(embedder);
	
					let document = Document {
						paper_id,
						title,
						abstract_text,
						body_text,
						abstract_embedding,
						body_embedding,
					};
	
					let mut db = db.lock().await;

					db.insert_document(&document).await.unwrap();
				});
			}
		}

		Ok(())
	}
}
