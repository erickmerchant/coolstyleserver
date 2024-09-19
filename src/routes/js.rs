use axum::{
	http::header,
	response::{IntoResponse, Response},
};
use std::include_str;

const COOL_STYLESHEET_JS: &str = include_str!("../../cool-stylesheet.js");

pub async fn js_handler() -> Response {
	(
		[(
			header::CONTENT_TYPE,
			"application/javascript; charset=utf-8",
		)],
		COOL_STYLESHEET_JS,
	)
		.into_response()
}
