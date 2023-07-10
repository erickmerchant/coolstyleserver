use axum::{
	extract::{Path, State},
	http::{header, uri::Uri, Request},
	response::sse::{Event, Sse},
	response::{IntoResponse, Response},
	routing::get,
	Router,
};
use clap::Parser;
use futures::stream::Stream;
use futures::{SinkExt, StreamExt};
use hyper::{client::HttpConnector, http::HeaderValue, Body};
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use notify::Watcher;
use pathdiff::diff_paths;
use serde_json::json;
use std::{convert::Infallible, fs::canonicalize, include_str, net::SocketAddr, path, time};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(short, long)]
	listen: u16,
	#[arg(short, long)]
	proxy: String,
	#[arg(short, long)]
	watch: String,
	#[arg(short, long)]
	base: String,
}

type Client = hyper::client::Client<HttpConnector, Body>;

const COOL_STYLESHEET_JS: &str = include_str!("../cool-stylesheet.js");

#[derive(Clone)]
struct AppState {
	args: Args,
	client: Client,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();
	let client = Client::new();
	let state = AppState {
		args: args.clone(),
		client: client.clone(),
	};
	let cool_api = Router::new()
		.route("/cool-stylesheet.js", get(client_js_handler))
		.route("/changes", get(sse_handler));
	let app = Router::new()
		.route("/", get(root_handler))
		.nest(format!("/{}", state.args.base).as_str(), cool_api)
		.route("/*path", get(handler))
		.with_state(state);
	let addr = SocketAddr::from(([0, 0, 0, 0], args.listen));
	println!("reverse proxy listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.unwrap();
}

async fn root_handler(State(state): State<AppState>, req: Request<Body>) -> Response<Body> {
	handler(State(state), Path("/".to_string()), req).await
}

async fn client_js_handler() -> Response {
	(
		[(header::CONTENT_TYPE, "application/javascript".to_string())],
		COOL_STYLESHEET_JS.to_string(),
	)
		.into_response()
}

async fn sse_handler(
	State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
	Sse::new(async_stream::try_stream! {
		let (mut watcher, mut rx) = async_watcher().unwrap();
		watcher.watch(path::Path::new(state.args.watch.as_str()), notify::RecursiveMode::Recursive).unwrap();

		while let Some(res) = rx.next().await {
			match res {
				Ok(event) => {
					println!("changed: {:?}", event);
					let hrefs = event.paths.iter().map(|p| {
						if let Some(p) = diff_paths(p, canonicalize(state.args.watch.as_str()).unwrap()) {
							Some(format!("/{}", p.to_str().unwrap()))
						} else {
							None
						}
					}).collect::<Vec<_>>();

					yield Event::default().data(json!({
						"hrefs": hrefs,
					}).to_string());
				},
				Err(e) => println!("watch error: {:?}", e),
			}
		}
	})
	.keep_alive(
		axum::response::sse::KeepAlive::new()
			.interval(time::Duration::from_secs(10))
			.text("keep-alive-text"),
	)
}

async fn handler(
	State(state): State<AppState>,
	Path(path): Path<String>,
	mut req: Request<Body>,
) -> Response<Body> {
	let path_query = req
		.uri()
		.path_and_query()
		.map(|v| v.as_str())
		.unwrap_or(&path);
	let uri = format!("{}{}", state.args.proxy, path_query);

	println!("{}", uri);
	*req.uri_mut() = Uri::try_from(uri).unwrap();
	req.headers_mut().insert(
		"accept-encoding",
		HeaderValue::from_str("identity").unwrap(),
	);

	let mut res = state.client.request(req).await.unwrap();
	let mut headers = res.headers_mut().clone();

	if let Some(header) = headers.get("content-type") {
		if header.as_ref().starts_with("text/html".as_bytes()) {
			let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
			let body = String::from_utf8(body.to_vec()).unwrap();
			let mut output = vec![];
			let mut rewriter = HtmlRewriter::new(
				Settings {
					element_content_handlers: vec![element!("link[rel=stylesheet]", |el| {
						el.set_attribute("is", "cool-stylesheet").unwrap();

						el.after(
							&format!(
								r#"<script type="module" src="/{}/cool-stylesheet.js"></script>"#,
								state.args.base
							),
							ContentType::Html,
						);

						Ok(())
					})],
					..Default::default()
				},
				|c: &[u8]| output.extend_from_slice(c),
			);

			rewriter.write(body.as_bytes()).unwrap();
			rewriter.end().unwrap();
			let body = String::from_utf8(output).unwrap();

			headers.remove("content-length");

			res = Response::new(Body::from(body));

			*res.headers_mut() = headers;
		}
	}

	res
}

fn async_watcher() -> notify::Result<(
	notify::RecommendedWatcher,
	futures::channel::mpsc::Receiver<notify::Result<notify::Event>>,
)> {
	let (mut tx, rx) = futures::channel::mpsc::channel(1);

	// Automatically select the best implementation for your platform.
	// You can also access each implementation directly e.g. INotifyWatcher.
	let watcher = notify::RecommendedWatcher::new(
		move |res| {
			futures::executor::block_on(async {
				tx.send(res).await.unwrap();
			})
		},
		notify::Config::default(),
	)?;

	Ok((watcher, rx))
}
