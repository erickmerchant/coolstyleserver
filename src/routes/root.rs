use crate::routes;
use axum::{
	extract::{Path, State},
	http::Request,
	response::Response,
};
use hyper::Body;

pub async fn root(
	State(state): State<crate::State>,
	req: Request<Body>,
) -> Result<Response<Body>, crate::Error> {
	routes::proxy(State(state), Path("/".to_string()), req).await
}
