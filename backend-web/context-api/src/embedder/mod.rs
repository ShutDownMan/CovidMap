use std::{env, fmt};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sbert::SBertRT;
use pgvector::Vector;
// use rocket::fairing::{Fairing, Info, Kind};
// use rocket::{Request, Response, Data};

pub enum EmbeddingModelType {
    DistilBERT,
}

impl fmt::Display for EmbeddingModelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmbeddingModelType::DistilBERT => write!(f, "DistilBERT"),
        }
    }
}

pub struct Embedder {
    distil_bert_model: Arc<Mutex<SBertRT>>,
}

impl Embedder {
    /// create a sentence embedder instance
    pub fn new() -> Embedder {
        let mut home: PathBuf = PathBuf::new();
        home.push(env::var("PRETRAINED_DISTILBERT_MODEL_PATH").unwrap()); 
        let distil_bert_model = Arc::new(Mutex::new(SBertRT::new(home).unwrap()));

        Embedder {
            distil_bert_model
        }
    }

    pub fn embed_snippet(&self, model_type: &EmbeddingModelType, text: &String) -> Vector {
		match model_type {
            EmbeddingModelType::DistilBERT => {
                let model_lock = self.distil_bert_model.lock().unwrap();

                Vector::from((*model_lock).encode(&[text], None).unwrap().remove(0))
            }, 
        }
    }
}

// impl Fairing for Embedder {
//     // This is a request and response fairing named "GET/POST Embedder".
//     fn info(&self) -> Info {
//         Info {
//             name: "GET/POST Embedder",
//             kind: Kind::Request | Kind::Response
//         }
//     }

//     fn on_request(&self, request: &mut Request, _: &Data) { () }

//     fn on_response(&self, request: &Request, response: &mut Response) { () }
// }