use postgres::{Client, NoTls, Error};

struct Document {
    paper_id: String,
    abstract_text: String,
    body_text: String
}

pub query_documents() -> Vec<Document> {

}

