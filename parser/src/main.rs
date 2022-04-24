mod database;
//mod indexer;
mod search;
mod transformer;
mod utils;

use database::{Database, Json};
use json::{object, array, JsonValue};
use transformer::transformer::{Embedder, EmbedderHandle};

use dotenv;
use tokio;

use std::convert::Infallible;
use std::io;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::task;

use futures::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

#[tokio::main]
async fn main() {
	dotenv::dotenv().expect("Failed to read .env file");

	let ast = search::query::parse(
		r#" ("pregnant" OR pregnancy OR maternity) (covid OR Sars-Cov-2) (effects OR disease) "#,
	)
	.unwrap();

	let pg_query = search::ast_to_query(&ast);

	let mut database = Database::new().await.unwrap();
	let db = Arc::new(database);

	let mut embedder = Embedder::new(db.clone());
	let h_embedder = EmbedderHandle::new(embedder);

	match startup_server(h_embedder.clone()).await {
		Ok(_) => {}
		Err(e) => {
			println!("{}", e);
		}
	}
}

async fn startup_server(
	context: EmbedderHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let addr = ([127, 0, 0, 1], 6553).into();

	let context = context.clone();

	let service = make_service_fn(move |_| {
		let context = context.clone();

		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| {
				let context = context.clone();
				async move {
					println!("Got {} request to {}", req.method(), req.uri().path());
					Ok::<_, Infallible>(match router(req, context.clone()).await {
						Ok(rsp) => {
							println!("Sending success response");
							rsp
						}
						Err(e) => {
							println!("Sending error response");
							Response::new(Body::from(format!("Internal error {:?}", e)))
						}
					})
				}
			}))
		}
	});

	let server = Server::bind(&addr).serve(service);

	println!("Listening on http://{}", addr);

	server.await?;

	Ok(())
}
use futures::executor;
use tokio::runtime::Runtime; // 0.3.5


/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn router(
	req: Request<Body>,
	context: EmbedderHandle,
) -> Result<Response<Body>, hyper::http::Error> {
	if req.method() == &Method::OPTIONS {
		println!("Sending OPTIONS headers");
		return Ok(Response::builder()
			.status(StatusCode::OK)
			.header("Access-Control-Allow-Origin", "*")
			.header("Access-Control-Allow-Headers", "*")
			.header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
			.body(Body::default())?)
	}

	match (req.method(), req.uri().path()) {
		// Serve some instructions at /
		(&Method::GET, "/") => Ok(Response::new(Body::from(
			"Try POSTing data to /echo such as: `curl localhost:6553/search/context -XPOST -d 'hello world'`",
		))),

		(&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

		(&Method::POST, "/search/context") => {
			let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
			let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
			let body_json = json::parse(&body_string);
			
			let search_query = match body_json.ok() {
				Some(body_json) => {
					if body_json["search_query"].is_null() {
						println!("Sending BAD_REQUEST");
						return Response::builder()
							.status(StatusCode::BAD_REQUEST)
							.header("Access-Control-Allow-Origin", "*")
							.body(Body::from(r#"{
								"error": "Bad search request."
							}"#));
					}

					body_json["search_query"].to_string()
				}
				None => {
					println!("Sending BAD_REQUEST");
					return Response::builder()
						.status(StatusCode::BAD_REQUEST)
						.header("Access-Control-Allow-Origin", "*")
						.body(Body::from(r#"{
							"error": could not parse request."
						}"#));
				}
			};
			println!("{:#?}", search_query);

			// get context lock
			let ctx = context.inner.clone();
			// spawn a thread that allows blocking
			let docs = task::spawn_blocking(move || {
				// block the current thread
				executor::block_on(async move {
					// get context lock
					let ctx_lock = ctx.lock().await;
					// context search body string
					ctx_lock.semantic_query(&search_query).await
				})
			}).await.unwrap();

			// serialize documents to json objects
			let docs_json = object!{
				search_results: docs.iter()
					.map(|doc| doc.to_json())
					.collect::<Vec<json::JsonValue>>()
			};

			println!("Sending OK");
			let response = Response::builder()
				.status(StatusCode::OK)
				.header("Access-Control-Allow-Origin", "*")
				.body(Body::from(docs_json.to_string()))?;

			// return serialized documents as a json string
			Ok(response)
		}

		// Return the 404 Not Found for other routes.
		_ => {
			println!("Sending NOT_FOUND");
			let mut not_found = Response::default();
			*not_found.status_mut() = StatusCode::NOT_FOUND;
			Ok(not_found)
		}
	}
}

/*

""

// let ast = search::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");

*/
