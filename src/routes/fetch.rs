// use super::fallback::fallback_handler;
use crate::args::Commands;
use axum::{
	extract::{Path, State},
	Json,
};
use lightningcss::{
	bundler::{Bundler, FileProvider},
	printer::PrinterOptions,
	stylesheet::ParserOptions,
};
use parcel_sourcemap::SourceMap;
use serde::Serialize;
use std::{path, sync::Arc};

#[derive(Serialize)]
pub struct Payload {
	pub css: String,
	pub sources: Vec<String>,
}

pub async fn fetch_handler(
	State(state): State<Arc<crate::State>>,
	Path(path): Path<String>,
) -> Result<Json<Payload>, crate::Error> {
	let fs = FileProvider::new();
	let directory = match &state.args.command {
		Commands::Proxy { directory, .. } => directory,
		Commands::Serve { directory, .. } => directory,
	}
	.as_str();
	let mut sourcemap = SourceMap::new("");
	let mut bundler = Bundler::new(&fs, Some(&mut sourcemap), ParserOptions::default());
	let stylesheet = bundler
		.bundle(&path::Path::new(directory).join(&path))
		.unwrap();
	let res = stylesheet
		.to_css(PrinterOptions {
			source_map: Some(&mut sourcemap),
			..PrinterOptions::default()
		})
		.unwrap();
	let css = res.code;
	let sources: Vec<String> = sourcemap
		.get_sources()
		.iter()
		.map(|s| s.trim_start_matches(directory).to_string())
		.collect();
	let json = Payload { css, sources };

	Ok(Json(json))
}
