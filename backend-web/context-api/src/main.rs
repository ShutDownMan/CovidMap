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

// use rocket::State;
// use rocket::http::Status;

// use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

mod embedder;
mod routes;
mod services;

use embedder::Embedder;

// #[database("contextdb")]
// pub struct DbConn(diesel::PgConnection);

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
        .mount(
            "/api",
            routes![index, get_current_date, snippet_post_handler, snippet_get_handler, search_handler],
        )
        .ignite().await?
        .launch().await?;

    Ok(())
}
