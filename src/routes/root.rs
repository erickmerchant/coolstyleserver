use super::fallback::fallback_handler;
use axum::{body::Body, extract::State, http::Request, response::Response};
use std::sync::Arc;

pub async fn root_handler(
	State(state): State<Arc<crate::State>>,
	req: Request<Body>,
) -> Result<Response<Body>, crate::Error> {
	fallback_handler(State(state), req).await
}
