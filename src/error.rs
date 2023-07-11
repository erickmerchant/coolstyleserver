use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
};

pub struct Error(anyhow::Error);

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		let Error(err) = self;

		(
			StatusCode::INTERNAL_SERVER_ERROR,
			err.backtrace().to_string(),
		)
			.into_response()
	}
}

impl<E> From<E> for Error
where
	E: Into<anyhow::Error>,
{
	fn from(err: E) -> Self {
		Self(err.into())
	}
}
