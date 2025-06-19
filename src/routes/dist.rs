use axum::{
	extract::Path,
	http::{header, StatusCode},
	response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

const DIST: Dir = include_dir!("./dist/");

pub async fn dist_handler(Path(path): Path<String>) -> Response {
	if let (Some(content_type), Some(Some(body))) = (
		mime_guess::from_path(&path).first(),
		DIST.get_file(path.trim_start_matches("/"))
			.map(|f| f.contents_utf8()),
	) {
		return ([(header::CONTENT_TYPE, content_type.to_string())], body).into_response();
	}

	StatusCode::NOT_FOUND.into_response()
}
