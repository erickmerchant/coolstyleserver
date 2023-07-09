use axum::{
	extract::{Path, State},
	http::{header, uri::Uri, Request},
	response::sse::{Event, Sse},
	response::{IntoResponse, Response},
	routing::get,
	Router,
};
use futures::stream::{self, Stream};
use hyper::{client::HttpConnector, http::HeaderValue, Body};
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use std::{convert::Infallible, include_str, net::SocketAddr, time::Duration};
use tokio_stream::StreamExt;

type Client = hyper::client::Client<HttpConnector, Body>;

const COOL_STYLESHEET_JS: &str = include_str!("../cool-stylesheet.js");

#[tokio::main]
async fn main() {
	let client = Client::new();
	let cool_api = Router::new()
		.route("/cool-stylesheet.js", get(client_js_handler))
		.route("/changes", get(sse_handler));
	let app = Router::new()
		.route("/", get(root_handler))
		.nest("/coolstyleserver", cool_api)
		.route("/*path", get(handler))
		.with_state(client);

	let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
	println!("reverse proxy listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.unwrap();
}

async fn root_handler(State(client): State<Client>, req: Request<Body>) -> Response<Body> {
	handler(State(client), Path("/".to_string()), req).await
}

async fn client_js_handler() -> Response {
	(
		[(header::CONTENT_TYPE, "application/javascript".to_string())],
		COOL_STYLESHEET_JS.to_string(),
	)
		.into_response()
}

async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
	let stream = stream::repeat_with(|| Event::default().data(r#"{"hrefs":[]}"#))
		.map(Ok)
		.throttle(Duration::from_secs(1));

	// 	let mut watcher = notify::recommended_watcher(|res| {
	// 		match res {
	// 			 Ok(event) => println!("event: {:?}", event),
	// 			 Err(e) => println!("watch error: {:?}", e),
	// 		}
	// })?;
	// watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

	Sse::new(stream).keep_alive(
		axum::response::sse::KeepAlive::new()
			.interval(Duration::from_secs(1))
			.text("keep-alive-text"),
	)
}

async fn handler(
	State(client): State<Client>,
	Path(path): Path<String>,
	mut req: Request<Body>,
) -> Response<Body> {
	let path_query = req
		.uri()
		.path_and_query()
		.map(|v| v.as_str())
		.unwrap_or(&path);
	let uri = format!("http://127.0.0.1:3000{}", path_query);

	println!("{}", uri);
	*req.uri_mut() = Uri::try_from(uri).unwrap();
	req.headers_mut().insert(
		"accept-encoding",
		HeaderValue::from_str("identity").unwrap(),
	);

	let mut res = client.request(req).await.unwrap();
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
							r#"<script type="module" src="/coolstyleserver/cool-stylesheet.js"></script>"#,
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
