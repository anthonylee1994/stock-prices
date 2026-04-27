use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug)]
pub(crate) enum ApiError {
    BadRequest { error: &'static str },
    Toon(toon_format::ToonError),
    YahooCrumb,
    YahooFinance(reqwest::Error),
    UpstreamStatus(StatusCode),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest { error } => (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })),
            Self::Toon(error) => {
                eprintln!("Error encoding TOON response: {error}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to encode response",
                    }),
                )
            }
            Self::YahooCrumb => {
                eprintln!("Yahoo Finance did not return a valid crumb");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: "Failed to fetch quotes",
                    }),
                )
            }
            Self::YahooFinance(error) => {
                eprintln!("Error fetching quotes: {error}");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: "Failed to fetch quotes",
                    }),
                )
            }
            Self::UpstreamStatus(status) => {
                eprintln!("Yahoo Finance returned status {status}");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: "Failed to fetch quotes",
                    }),
                )
            }
        }
        .into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}
