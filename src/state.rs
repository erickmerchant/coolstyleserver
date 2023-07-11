use crate::args::Args;
use clap::Parser;
use hyper::{client::HttpConnector, Body};

type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct State {
	pub args: Args,
	pub client: Client,
}

impl State {
	pub fn new() -> Self {
		let args = Args::parse();
		let client = Client::new();
		Self {
			args: args.clone(),
			client: client.clone(),
		}
	}
}
