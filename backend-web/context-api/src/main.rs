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

use rocket_cors::{AllowedOrigins, CorsOptions};
use sqlx::postgres::PgPoolOptions;
use std::str::FromStr;

mod embedder;
mod routes;
mod services;

use embedder::Embedder;

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

    let cors = CorsOptions::default()
    .allowed_origins(AllowedOrigins::all())
    .allowed_methods(
        ["Get", "Post", "Patch"]
            .iter()
            .map(|s| FromStr::from_str(s).unwrap())
            .collect()
    )
    .allow_credentials(true)
    .to_cors()?;

    let _rocket = rocket::build()
        .manage(pool)
        .manage(embedder)
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
        .attach(cors)
        .ignite().await?
        .launch().await?;

    Ok(())
}
