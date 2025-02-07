use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use moly_protocol::protocol::ApiError;

/// Wrapper struct that implements `IntoResponse` for `ApiError`.
pub struct ApiErrorResponse(pub ApiError, pub StatusCode);

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let (error, status) = (self.0, self.1);
        (status, Json(error)).into_response()
    }
}

/// Utility function for mapping errors into a `ApiErrorResponse`.
pub fn api_error(status: StatusCode, message: &str, param: Option<&str>) -> ApiErrorResponse {
    ApiErrorResponse(
        ApiError::new(
            message,
            status.canonical_reason().unwrap_or("unknown"),
            param,
            Some(status.as_str()),
        ),
        status,
    )
}

/// Utility function for mapping errors into a `500 Internal Server Error`
/// response.
pub fn internal_error<E: std::fmt::Display>(err: E) -> ApiErrorResponse {
    log::error!("Internal server error: {}", err);
    api_error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string(), None)
}
