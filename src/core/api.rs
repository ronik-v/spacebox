use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success { data: T },
    Error { error: ApiError }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ApiError {
    pub message: String,
}

impl<T: serde::Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Success { data } => {
                let body = serde_json::json!({ "data": data });
                (StatusCode::OK, Json(body)).into_response()
            }
            ApiResponse::Error { error } => {
                let status = StatusCode::BAD_REQUEST;
                (status, Json(serde_json::json!({ "error": error }))).into_response()
            }
        }
    }
}