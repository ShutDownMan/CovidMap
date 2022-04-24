// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_mut)]
// #![allow(dead_code)]

pub mod transformer {
	// Taken from: https://github.com/allenai/rustberta-snli/blob/master/src/modeling.rs
	use sbert::SBertRT;

	use std::env;
	use std::path::PathBuf;

	use crate::database::{Database, Document, Paragraph};
	use crate::utils::PgVec;

	use std::sync::Arc;
	use tokio::sync::Mutex;
	use tokio::task;

	use futures::executor;

	pub struct Embedder {
		model: SBertRT,
		database: Arc<Database>,
	}

	impl Embedder {
		/// create a sentence embedder instance
		pub fn new(database: Arc<Database>) -> Embedder {
			let mut home: PathBuf = env::current_dir().unwrap();
			home.push(env::var("PRETRAINED_MODEL_PATH").unwrap());

			let sbert_model = SBertRT::new(home).unwrap();

			Embedder {
				model: sbert_model,
				database: database,
			}
		}

		pub async fn semantic_query(&self, query_text: &str) -> Vec<Document> {
			let query_embedding = self.embed_sentence(query_text);

			self.database
				.get_papers_by_embedding(query_embedding, Some(20))
				.await
		}

		pub fn embed_sentence(&self, text: &str) -> PgVec {
			PgVec(self.model.forward(&[text], None).unwrap().remove(0))
		}
	}

	#[derive(Clone)]
	pub struct EmbedderHandle {
		pub inner: Arc<Mutex<Embedder>>,
	}

	impl EmbedderHandle {
		pub fn new(embedder: Embedder) -> Self {
			Self {
				inner: Arc::new(Mutex::new(embedder)),
			}
		}
		pub async fn with_lock<F, T>(&self, func: F) -> T
		where
			F: FnOnce(&mut Embedder) -> T + Send + 'static,
			T: Send + 'static,
		{
			// let ctx = self.inner.clone();
			// task::spawn_blocking(move || {
			// 	let ctx = self.inner.clone();
			// 	executor::block_on(async move {
			// 		let mut lock = ctx.lock().await;
			// 		func(&mut *lock)
			// 	})
			// })
			// .await
			// .unwrap();

			let ctx = self.inner.clone();
			task::spawn_blocking(move || {
				executor::block_on(async move {
					let mut lock = ctx.lock().await;
					func(&mut *lock)
				})
			}).await.unwrap()
		}
	}
}
