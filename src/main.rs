mod args;
mod error;
mod routes;
mod state;

use axum::{routing::get, serve, Router};
use error::Error;
use routes::{js::js_handler, proxy::proxy_handler, root::root_handler, watch::watch_handler};
use state::State;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
	let state = State::default();
	let state = Arc::new(state);
	let cool_api = Router::new()
		.route("/cool-stylesheet.js", get(js_handler))
		.route("/watch", get(watch_handler));
	let app = Router::new()
		.route("/", get(root_handler))
		.nest(format!("/{}", state.args.cool_base).as_str(), cool_api)
		.route("/*path", get(proxy_handler))
		.with_state(state.clone());
	let listener = TcpListener::bind(("0.0.0.0", state.args.listen))
		.await
		.expect("should listen");

	serve(listener, app.into_make_service())
		.await
		.expect("server should start");
}
