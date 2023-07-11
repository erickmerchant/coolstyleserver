use axum::{
	http::header,
	response::{IntoResponse, Response},
};
use std::include_str;

const COOL_STYLESHEET_JS: &str = include_str!("../../cool-stylesheet.js");

pub async fn js() -> Response {
	(
		[(header::CONTENT_TYPE, "application/javascript".to_string())],
		COOL_STYLESHEET_JS.to_string(),
	)
		.into_response()
}
