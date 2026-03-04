use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendCodeRequest {
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConfirmRegistrationRequest {
    pub email: String,
    pub code: String,
    pub password: String,
}