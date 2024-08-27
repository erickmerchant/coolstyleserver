use axum::{
	body::Body,
	extract::{Path, State},
	http::Request,
	response::{IntoResponse, Response},
};
use http_body_util::{BodyExt, Empty};
use hyper::header::HeaderValue;
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use std::sync::Arc;

pub async fn proxy_handler(
	State(state): State<Arc<crate::State>>,
	Path(path): Path<String>,
	req: Request<Body>,
) -> Result<Response<Body>, crate::Error> {
	let path_query = req
		.uri()
		.path_and_query()
		.map(|v| v.as_str())
		.unwrap_or(&path);
	let url = format!("{}{}", state.args.proxy, path_query);
	let original_headers = req.headers();
	let mut req = hyper::Request::builder()
		.uri(url)
		.body(Empty::<bytes::Bytes>::new())?;

	*req.headers_mut() = original_headers.clone();

	req.headers_mut()
		.insert("accept-encoding", HeaderValue::from_str("identity")?);

	let res = state.client.request(req).await?;
	let (parts, body) = res.into_parts();
	let bytes = body.collect().await?.to_bytes();
	let mut res = Response::from_parts(parts.clone(), Body::from(bytes.clone())).into_response();

	if res
		.headers()
		.get("content-type")
		.map_or(false, |h| h.as_ref().starts_with("text/html".as_bytes()))
	{
		let mut output = Vec::new();
		let mut rewriter = HtmlRewriter::new(
			Settings {
				element_content_handlers: vec![element!("link[rel=stylesheet]", |el| {
					el.set_attribute("is", "cool-stylesheet")?;
					el.after(
						&format!(
							r#"<script type="module" src="/{}/cool-stylesheet.js"></script>"#,
							state.args.cool_base
						),
						ContentType::Html,
					);

					Ok(())
				})],
				..Default::default()
			},
			|c: &[u8]| output.extend_from_slice(c),
		);

		rewriter.write(&bytes)?;
		rewriter.end()?;

		let body = String::from_utf8(output)?;

		res.headers_mut().remove("content-length");
		*res.body_mut() = Body::from(body);
	}

	Ok(res)
}
