use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// HTTP errors returned by the API.
///
/// The serialized shape mirrors the `HttpException` bodies produced by the
/// previous NestJS implementation so existing clients keep working.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    BadRequest(String),
    MethodNotAllowed,
    BadGateway(String),
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn bad_gateway(message: impl Into<String>) -> Self {
        Self::BadGateway(message.into())
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::BadGateway(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

/// Nest emits `{message, error, statusCode}` when an exception carries a custom
/// message, and `{message, statusCode}` when it only carries the status text.
#[derive(Debug, Serialize)]
struct ApiErrorBody {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'static str>,
    #[serde(rename = "statusCode")]
    status_code: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = match self {
            Self::BadRequest(message) => ApiErrorBody {
                message,
                error: Some("Bad Request"),
                status_code: status.as_u16(),
            },
            Self::MethodNotAllowed => ApiErrorBody {
                message: "Method Not Allowed".to_owned(),
                error: None,
                status_code: status.as_u16(),
            },
            Self::BadGateway(message) => ApiErrorBody {
                message,
                error: Some("Bad Gateway"),
                status_code: status.as_u16(),
            },
        };

        (status, Json(body)).into_response()
    }
}

/// Shared `@All()` handler: any method we do not explicitly route is a 405.
pub async fn method_not_allowed() -> ApiError {
    ApiError::MethodNotAllowed
}
