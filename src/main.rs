mod args;
mod error;
mod routes;
mod state;

use error::*;
use routes::{js::*, proxy::*, root::*, watch::*};
use state::*;

use axum::{routing::get, Router, Server};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
	let state = State::default();
	let cool_api = Router::new()
		.route("/cool-stylesheet.js", get(js))
		.route("/watch", get(watch));
	let app = Router::new()
		.route("/", get(root))
		.nest(format!("/{}", state.args.base).as_str(), cool_api)
		.route("/*path", get(proxy))
		.with_state(state.clone());
	let addr = SocketAddr::from(([0, 0, 0, 0], state.args.listen));

	Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.expect("server should start");
}
