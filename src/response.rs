use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use toon_format::encode_default;

use crate::error::ApiError;

pub(crate) struct ToonResponse {
    body: String,
}

impl ToonResponse {
    fn new(body: String) -> Self {
        Self { body }
    }
}

impl IntoResponse for ToonResponse {
    fn into_response(self) -> Response {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            )],
            self.body,
        )
            .into_response()
    }
}

pub(crate) fn toon_response<T: Serialize>(value: &T) -> Result<ToonResponse, ApiError> {
    encode_default(value)
        .map(ToonResponse::new)
        .map_err(ApiError::Toon)
}
