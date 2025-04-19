mod args;
mod error;
mod routes;
mod state;

use axum::{routing::get, serve, Router};
use error::Error;
use routes::{
	fallback::fallback_handler, fetch::fetch_handler, public::public_handler, watch::watch_handler,
};
use state::State;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
	let state = State::default();
	let listen = state.args.port;
	let state = Arc::new(state);

	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::try_from_default_env().unwrap_or_default())
		.init();

	let cool_api = Router::new()
		.route("/watch", get(watch_handler))
		.route("/fetch", get(fetch_handler))
		.route("/{*path}", get(public_handler));
	let app = Router::new()
		.nest(format!("/{}", state.args.cool_base).as_str(), cool_api)
		.fallback(get(fallback_handler))
		.with_state(state)
		.layer(TraceLayer::new_for_http());
	let listener = TcpListener::bind(("0.0.0.0", listen))
		.await
		.expect("should listen");

	serve(listener, app.into_make_service())
		.await
		.expect("server should start");
}
