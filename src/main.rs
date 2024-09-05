mod args;
mod error;
mod routes;
mod state;

use axum::{routing::get, serve, Router};
use error::Error;
use routes::{js::js_handler, proxy::proxy_handler, root::root_handler, watch::watch_handler, fetch::fetch_handler};
use state::State;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
	let state = State::default();
	let listen = state.args.listen;
	let state = Arc::new(state);

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(tracing::Level::DEBUG)
		.init();

	let cool_api = Router::new()
		.route("/cool-stylesheet.js", get(js_handler))
		.route("/watch", get(watch_handler))
		.route("/fetch", get(fetch_handler));
	let app = Router::new()
		.route("/", get(root_handler))
		.nest(format!("/{}", state.args.cool_base).as_str(), cool_api)
		.route("/*path", get(proxy_handler))
		.with_state(state)
		.layer(TraceLayer::new_for_http());
	let listener = TcpListener::bind(("0.0.0.0", listen))
		.await
		.expect("should listen");

	serve(listener, app.into_make_service())
		.await
		.expect("server should start");
}
