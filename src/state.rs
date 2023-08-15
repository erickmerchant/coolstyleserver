use crate::args::Args;
use clap::Parser;
use hyper::{
	client::{Client, HttpConnector},
	Body,
};

type AppClient = Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct State {
	pub args: Args,
	pub client: AppClient,
}

impl Default for State {
	fn default() -> Self {
		let args = Args::parse();
		let client = AppClient::new();

		Self { args, client }
	}
}
