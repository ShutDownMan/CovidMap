#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

// Taken from: https://github.com/allenai/rustberta-snli/blob/master/src/modeling.rs
use sbert::SBertRT;

use std::env;
use std::path::PathBuf;

pub struct Embedder {
	model: SBertRT,
}

impl Embedder {
	pub fn new() -> Embedder {
		let mut home: PathBuf = env::current_dir().unwrap();
		home.push(env::var("PRETRAINED_MODEL_PATH").unwrap());

		// println!("{:#?}", home);

		let sbert_model = SBertRT::new(home).unwrap();

		let texts = ["effects of covid on pregnancy and pregnant women"];

		let batch_size = None;

		let output = sbert_model.forward(&texts.to_vec(), batch_size).unwrap();

		// println!("{:#?}", output);

		let sum = output[0].iter().map(|x| x.abs()).sum::<f32>();

		println!("{:#?}", sum);
		println!("{:#?}", output[0]);

		Embedder { model: sbert_model }
	}
}
