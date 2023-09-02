use axum::{
	extract::{Path, State},
	http::{uri::Uri, Request},
	response::Response,
};
use hyper::{body::to_bytes, header::HeaderValue, Body};
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use std::sync::Arc;

pub async fn proxy_handler(
	State(state): State<Arc<crate::State>>,
	Path(path): Path<String>,
	mut req: Request<Body>,
) -> Result<Response<Body>, crate::Error> {
	let path_query = req
		.uri()
		.path_and_query()
		.map(|v| v.as_str())
		.unwrap_or(&path);
	let uri = format!("{}{}", state.args.proxy, path_query);

	*req.uri_mut() = Uri::try_from(uri)?;
	req.headers_mut()
		.insert("accept-encoding", HeaderValue::from_str("identity")?);

	let mut res = state.client.request(req).await?;
	let mut headers = res.headers_mut().clone();

	if headers
		.get("content-type")
		.map_or(false, |h| h.as_ref().starts_with("text/html".as_bytes()))
	{
		let body = to_bytes(res.into_body()).await?;
		let body = String::from_utf8(body.to_vec())?;
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

		rewriter.write(body.as_bytes())?;
		rewriter.end()?;

		let body = String::from_utf8(output)?;

		headers.remove("content-length");
		res = Response::new(Body::from(body));
		*res.headers_mut() = headers;
	}

	Ok(res)
}
