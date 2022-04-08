mod database;
//mod indexer;
mod search;
mod transformer;
mod utils;

use database::Database;
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

	// let docs = db.match_query(pg_query).await;

	// println!("{:#?}", docs);

	let mut embedder = Embedder::new(db.clone());
	let h_embedder = EmbedderHandle::new(embedder);

	// let docs = embedder_ts
	// 	.lock()
	// 	.await
	// 	.semantic_query("what are the effects of coronavirus or covid on pregnant women?")
	// 	.await;

	// loop {
	// 	use std::time::Instant;
	// 	let mut query_text = String::new();

	// 	io::stdin()
	// 		.read_line(&mut query_text)
	// 		.expect("failed to readline");

	// 	let now = Instant::now();
	// 	{
	// 		let docs = (&embedder).semantic_query(&query_text.to_string()).await;

	// 		println!("=================================");
	// 		println!("{:#?}", docs);
	// 		println!("=================================");
	// 	}
	// 	let elapsed = now.elapsed();
	// 	println!("Elapsed: {:.2?}", elapsed);
	// }

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
	let addr = ([127, 0, 0, 1], 3000).into();

	let context = context.clone();

	let service = make_service_fn(move |_| {
		let context = context.clone();

		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| {
				let context = context.clone();
				async move {
					println!("Got request");
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
) -> Result<Response<Body>, hyper::Error> {
	match (req.method(), req.uri().path()) {
		// Serve some instructions at /
		(&Method::GET, "/") => Ok(Response::new(Body::from(
			"Try POSTing data to /echo such as: `curl localhost:3000/search/context -XPOST -d 'hello world'`",
		))),

		(&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

		(&Method::POST, "/search/context") => {
			let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
			let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

			// get context lock
			let ctx = context.inner.clone();
			// spawn a thread that allows blocking
			let docs = task::spawn_blocking(move || {
				// block the current thread
				executor::block_on(async move {
					// get context lock
					let ctx_lock = ctx.lock().await;
					// context search body string
					ctx_lock.semantic_query(&body_string).await
				})
			}).await.unwrap();

			// let docs = context.with_lock(|ctx| async {
			// 	ctx.semantic_query(&body_string).await
			// }).await;

			// println!("=================================");
			// println!("{:#?}", docs);
			// println!("=================================");

			let docs_str = docs.iter()
				.map(|doc| doc.to_string())
				.collect::<Vec<String>>()
				.join("\n");

			// let docs_json = docs.iter()
			// 	.map(|doc| doc.to_json())
			// 	.collect();

			Ok(Response::new(Body::from(docs_str)))
		}

		// Return the 404 Not Found for other routes.
		_ => {
			let mut not_found = Response::default();
			*not_found.status_mut() = StatusCode::NOT_FOUND;
			Ok(not_found)
		}
	}
}

/*

"what are the effects of coronavirus or covid on pregnant women?"
"what are the coronavirus side effects and tribulations"
"what are the long term effects of corona virus disease Sars-Cov-2"
"how can the coronavirus mutations occour"
"which socioeconomical impacts does the coronavírus have on underdeveloped countries"
"what are the effective medication and safety approaches to coronavírus disease"
"political view on the corona virus pandemic"
"the aftermath of the pandemic"

// let ast = search::parse(" test OR from:place OR ( sub-query OR \"exactly\" )");

*/
