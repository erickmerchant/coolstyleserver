mod args;
mod error;
mod routes;
mod state;

use axum::{routing::get, Router, Server};
use error::Error;
use routes::{js::js_handler, proxy::proxy_handler, root::root_handler, watch::watch_handler};
use state::State;
use std::{net::SocketAddr, sync::Arc};

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
	let addr = SocketAddr::from(([0, 0, 0, 0], state.args.listen));

	Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.expect("server should start");
}
