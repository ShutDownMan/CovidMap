// Taken from: https://github.com/allenai/rustberta-snli/blob/master/src/modeling.rs

use tch::{nn, Device};
use rust_bert::resources::{LocalResource, Resource};
use rust_bert::roberta::RobertaForQuestionAnswering;
use rust_tokenizers::tokenizer::RobertaTokenizer;
use rust_bert::Config;
use rust_bert::bert::BertConfig;
use std::path::PathBuf;
use std::env;


pub struct Embedder {
    tokenizer: RobertaTokenizer,
    vs: tch::nn::VarStore,
    model: rust_bert::roberta::RobertaForQuestionAnswering
}

impl Embedder {
	pub fn new() -> Embedder {

        let config_resource = Resource::Local(LocalResource {
            local_path: PathBuf::from(env::var("TORCH_RESOURCE_PATH").unwrap()),
        });

        let vocab_resource = Resource::Local(LocalResource {
            local_path: PathBuf::from(env::var("TORCH_VOCAB_PATH").unwrap()),
        });

        let merges_resource = Resource::Local(LocalResource {
            local_path: PathBuf::from(env::var("TORCH_MERGES_PATH").unwrap()),
        });

        let weights_resource = Resource::Local(LocalResource {
            local_path: PathBuf::from(env::var("TORCH_WEIGHTS_PATH").unwrap()),
        });

        let config_path = config_resource.get_local_path().unwrap();
        let vocab_path = vocab_resource.get_local_path().unwrap();
        let merges_path = merges_resource.get_local_path().unwrap();
        let weights_path = weights_resource.get_local_path().unwrap();
        
        let device = Device::cuda_if_available();
        let mut vs = nn::VarStore::new(device);
        let tokenizer: RobertaTokenizer = RobertaTokenizer::from_file(
            vocab_path.to_str().unwrap(),
            merges_path.to_str().unwrap(),
            true,
            false
        ).unwrap();

        let config = BertConfig::from_file(config_path);
        let bert_model = RobertaForQuestionAnswering::new(&vs.root(), &config);
        vs.load(weights_path).unwrap();
        
		Embedder {
            tokenizer: tokenizer,
            vs: vs,
            model: bert_model
		}
	}
    
}