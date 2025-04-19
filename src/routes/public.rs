use axum::{
	extract::Path,
	http::{header, StatusCode},
	response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

const PUBLIC: Dir = include_dir!("./public/");

pub async fn public_handler(Path(path): Path<String>) -> Response {
	if let (Some(content_type), Some(Some(body))) = (
		mime_guess::from_path(&path).first(),
		PUBLIC
			.get_file(path.trim_start_matches("/"))
			.map(|f| f.contents_utf8()),
	) {
		return ([(header::CONTENT_TYPE, content_type.to_string())], body).into_response();
	}

	StatusCode::NOT_FOUND.into_response()
}
