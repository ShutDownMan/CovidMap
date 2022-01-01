// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(unused_mut)]
// #![allow(dead_code)]

// Taken from: https://github.com/allenai/rustberta-snli/blob/master/src/modeling.rs
use sbert::SBertRT;

use std::env;
use std::path::PathBuf;

use crate::database::{Database, Document, Paragraph};
use crate::utils::PgVec;

pub struct Embedder<'a> {
	model: SBertRT,
	database: &'a mut Database
}

impl<'a> Embedder<'a> {
	/// create a sentence embedder instance
	pub fn new(database: &'a mut Database) -> Embedder {
		let mut home: PathBuf = env::current_dir().unwrap();
		home.push(env::var("PRETRAINED_MODEL_PATH").unwrap());

		let sbert_model = SBertRT::new(home).unwrap();

		Embedder { model: sbert_model, database: database }
	}

	pub fn semantic_query(&mut self, query_text: &str) -> Vec<Paragraph> {
		let query_embedding = self.embed_sentence(query_text);

		// println!("{:#?}", query_embedding);

		self.database.find_similar_documents_by_embedding(PgVec(query_embedding), None)
	}

	fn embed_sentence(&self, text: &str) -> Vec<f32> {
		self.model.forward(&[text], None).unwrap().remove(0)
	}
}
