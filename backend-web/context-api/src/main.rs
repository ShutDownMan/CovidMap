#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
// use std::convert::Infallible;

use std::env;
use dotenv;

use anyhow::Result;

use routes::date::get_current_date;
use routes::snippet::{snippet_post_handler, snippet_get_handler};
use routes::search::search_handler;
use routes::document::get_document_by_paper_id;

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use sqlx::postgres::PgPoolOptions;

mod embedder;
mod routes;
mod services;

use embedder::Embedder;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    let embedder = Embedder::new();

    let _rocket = rocket::build()
        .manage(pool)
        .manage(embedder)
        .attach(CORS)
        .mount(
            "/api",
            routes![
                index,
                get_current_date,
                snippet_post_handler,
                snippet_get_handler,
                search_handler,
                get_document_by_paper_id
            ],
        )
        .ignite().await?
        .launch().await?;

    Ok(())
}
