use super::fallback::fallback_handler;
use axum::{
	body::to_bytes,
	extract::{Query, State},
	http::Request,
	Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

#[derive(Deserialize)]
pub struct Params {
	pub pathname: String,
}

#[derive(Serialize)]
pub struct Payload {
	pub css: String,
	pub sources: Vec<String>,
}

pub async fn fetch_handler(
	State(state): State<Arc<crate::State>>,
	params: Query<Params>,
) -> Result<Json<Payload>, crate::Error> {
	let mut new_req = Request::new("".into());

	*new_req.uri_mut() = params.pathname.parse()?;

	let res = fallback_handler(State(state.clone()), new_req).await?;
	let bytes = to_bytes(res.into_body(), usize::MAX).await?.to_vec();
	let css = String::from_utf8(bytes)?;
	let sourcemap = sourcemap::locate_sourcemap_reference(css.as_bytes())?;
	let url = match sourcemap {
		Some(sourcemap::SourceMapRef::Ref(url)) => Some(url),
		Some(sourcemap::SourceMapRef::LegacyRef(url)) => Some(url),
		_ => None,
	};
	let map = if let Some(url) = url {
		if let Ok(map) = sourcemap::decode_data_url(url.as_str()) {
			Some(map)
		} else {
			let base_url = Url::parse(format!("http://0.0.0.0:{}", state.args.port).as_str())?;
			let base_url = base_url.join(params.pathname.as_str())?;
			let url = base_url.join(url.as_str())?;
			let mut new_req = Request::new("".into());

			*new_req.uri_mut() = url.to_string().parse()?;

			let res = fallback_handler(State(state.clone()), new_req).await?;
			let bytes = to_bytes(res.into_body(), usize::MAX).await?.to_vec();
			let map = String::from_utf8(bytes)?;

			if let Ok(map) = sourcemap::decode(map.as_bytes()) {
				Some(map)
			} else {
				None
			}
		}
	} else {
		None
	};
	let sources = match map {
		Some(sourcemap::DecodedMap::Regular(map)) => map.sources().map(|s| s.to_string()).collect(),
		_ => vec![],
	};

	let json = Payload { css, sources };

	Ok(Json(json))
}
