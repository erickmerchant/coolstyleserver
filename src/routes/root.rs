use crate::routes::proxy;
use axum::{
	extract::{Path, State},
	http::Request,
	response::Response,
};
use hyper::Body;

pub async fn handler(
	State(state): State<crate::State>,
	req: Request<Body>,
) -> Result<Response<Body>, crate::Error> {
	proxy::handler(State(state), Path("/".to_string()), req).await
}
