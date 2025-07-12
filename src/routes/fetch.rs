use super::fallback::fallback_handler;
use crate::args::Commands;
use axum::{
	body::to_bytes,
	extract::{Path, State},
	http::Request,
	Json,
};
use serde::Serialize;
use std::sync::Arc;
use url::Url;

#[derive(Serialize)]
pub struct Payload {
	pub css: String,
	pub sources: Vec<String>,
}

pub async fn fetch_handler(
	State(state): State<Arc<crate::State>>,
	Path(path): Path<String>,
) -> Result<Json<Payload>, crate::Error> {
	let mut new_req = Request::new("".into());

	*new_req.uri_mut() = ("/".to_owned() + path.as_str()).parse()?;

	let res = fallback_handler(State(state.clone()), new_req).await?;
	let bytes = to_bytes(res.into_body(), usize::MAX).await?.to_vec();
	let css = String::from_utf8(bytes)?;
	let sourcemap = sourcemap::locate_sourcemap_reference(css.as_bytes())?;
	let map = match sourcemap {
		Some(sourcemap::SourceMapRef::Ref(url)) | Some(sourcemap::SourceMapRef::LegacyRef(url)) => {
			if let Ok(map) = sourcemap::decode_data_url(url.as_str()) {
				Some(map)
			} else {
				let base_url = Url::parse(
					format!(
						"http://0.0.0.0:{}",
						match state.args.command {
							Commands::Proxy { port, .. } => port,
							Commands::Serve { port, .. } => port,
						}
					)
					.as_str(),
				)?;
				let base_url = base_url.join(path.as_str())?;
				let url = base_url.join(url.as_str())?;
				let mut new_req = Request::new("".into());

				*new_req.uri_mut() = url.to_string().parse()?;

				let res = fallback_handler(State(state.clone()), new_req).await?;
				let bytes = to_bytes(res.into_body(), usize::MAX).await?.to_vec();
				let map = String::from_utf8(bytes)?;

				sourcemap::decode(map.as_bytes()).ok()
			}
		}
		_ => None,
	};
	let sources = match map {
		Some(sourcemap::DecodedMap::Regular(map)) => map.sources().map(|s| s.to_string()).collect(),
		_ => Vec::new(),
	};
	let json = Payload { css, sources };

	Ok(Json(json))
}
