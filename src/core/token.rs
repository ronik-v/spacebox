use axum::Json;
use rand::{RngCore, rngs::OsRng};
use base64::{Engine as _, engine::general_purpose};
use http::header::AUTHORIZATION;
use http::{HeaderMap, StatusCode};
use crate::core::api::ApiError;


pub fn get_auth_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}


pub struct TokenExtractManager {
    pub headers: HeaderMap
}

impl TokenExtractManager {
    pub fn new(headers: HeaderMap) -> Self { Self { headers } }

    pub fn check_token_error(&self) -> Result<String, (StatusCode, Json<ApiError>)> {
        return match self.extract_bearer_token() {
            Some(t) => Ok(t),
            None => {
                return Err((StatusCode::UNAUTHORIZED, Json(ApiError { message: "No token".into() })));
            }
        }
    }

    fn extract_bearer_token(&self) -> Option<String> {
        if let Some(value) = self.headers.get(AUTHORIZATION) {
            if let Ok(s) = value.to_str() {
                let s = s.trim();
                if let Some(rest) = s.strip_prefix("Bearer ") {
                    return Some(rest.to_string());
                }

                if let Some(rest) = s.strip_prefix("bearer ") {
                    return Some(rest.to_string());
                }
            }
        }
        None
    }
}