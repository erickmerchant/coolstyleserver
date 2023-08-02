use axum::{
	extract::{Path, State},
	http::{header, Request, StatusCode, Uri},
	response::{IntoResponse, Response},
};
use camino::Utf8Path;
use hyper::{body::to_bytes, header::HeaderValue, Body};
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use std::{fs, sync::Arc};

fn rewrite_body(base: String, body: String) -> Result<String, crate::Error> {
	let mut output = vec![];
	let mut rewriter = HtmlRewriter::new(
		Settings {
			element_content_handlers: vec![element!("link[rel=stylesheet]", |el| {
				el.set_attribute("is", "cool-stylesheet")?;
				el.after(
					&format!(
						r#"<script type="module" src="/{}/cool-stylesheet.js"></script>"#,
						base
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

	Ok(body)
}

pub async fn proxy(
	State(state): State<Arc<crate::State>>,
	Path(path): Path<String>,
	mut req: Request<Body>,
) -> Result<Response, crate::Error> {
	let path_query = req
		.uri()
		.path_and_query()
		.map(|v| v.as_str())
		.unwrap_or(&path);
	let mut result = Ok(StatusCode::NOT_FOUND.into_response());

	if let Some(proxy) = state.args.proxy.clone() {
		let uri = format!("{}{}", proxy, path_query);

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
			let body = rewrite_body(state.args.base.clone(), body)?;

			headers.remove("content-length");
			res = Response::new(Body::from(body));
			*res.headers_mut() = headers;
		};

		result = Ok(res.into_response());
	} else {
		let uri = Utf8Path::new(state.args.watch.clone().as_str())
			.join(path_query.trim_start_matches('/'));

		if let Some(ext) = uri.extension() {
			let asset = mime_guess::from_ext(ext)
				.first()
				.map(|content_type| (content_type, fs::read(uri)));

			if let Some((content_type, Ok(body))) = asset {
				let mut body = String::from_utf8(body)?;

				if content_type.to_string().starts_with("text/html") {
					body = rewrite_body(state.args.base.clone(), body)?;
				};

				result =
					Ok(([(header::CONTENT_TYPE, content_type.to_string())], body).into_response());
			};
		};
	}

	result
}
