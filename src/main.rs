mod args;
mod error;
mod routes;
mod state;

pub use args::*;
pub use error::*;
pub use state::*;

use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
	let state = State::new();
	let cool_api = Router::new()
		.route("/cool-stylesheet.js", get(routes::js))
		.route("/watch", get(routes::watch));
	let app = Router::new()
		.route("/", get(routes::root))
		.nest(format!("/{}", state.args.base).as_str(), cool_api)
		.route("/*path", get(routes::proxy))
		.with_state(state.clone());
	let addr = SocketAddr::from(([0, 0, 0, 0], state.args.listen));

	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.await
		.expect("server should start");
}
