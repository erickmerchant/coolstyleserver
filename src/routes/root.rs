use crate::routes::proxy::proxy_handler;
use axum::{
	extract::{Path, State},
	http::Request,
	response::Response,
};
use hyper::Body;
use std::sync::Arc;

pub async fn root_handler(
	State(state): State<Arc<crate::State>>,
	req: Request<Body>,
) -> Result<Response<Body>, crate::Error> {
	proxy_handler(State(state), Path("/".to_string()), req).await
}
