use crate::args::Args;
use clap::Parser;
use http_body_util::Empty;
use hyper_util::client::legacy::{connect::HttpConnector, Client};

type AppClient = Client<HttpConnector, Empty<bytes::Bytes>>;

#[derive(Clone)]
pub struct State {
	pub args: Args,
	pub client: AppClient,
}

impl Default for State {
	fn default() -> Self {
		let args = Args::parse();
		let client =
			Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpConnector::new());

		Self { args, client }
	}
}
