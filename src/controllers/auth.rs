use axum::{
    extract::{State, Json},
    routing::post,
    Router,
};
use utoipa::OpenApi;

use crate::{
    core::{
        api::{ApiResponse, ApiError},
        state::AppState,
    },
    dto::requests::{LoginRequest, SendCodeRequest, ConfirmRegistrationRequest},
    responses::users::UserOut,
    services::{auth::AuthService, registration::RegistrationService},
    dto::users::{UserDto, UserTokensShortDto},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::auth::login,
        crate::controllers::auth::send_verification_code,
        crate::controllers::auth::confirm_registration
    ),
    components(
        schemas(
            crate::dto::requests::LoginRequest,
            crate::dto::requests::SendCodeRequest,
            crate::dto::requests::ConfirmRegistrationRequest,
            crate::core::api::ApiError,
            crate::responses::users::UserOut,
            crate::dto::users::UserDto,
            crate::dto::users::UserTokensShortDto,
        )
    ),
    tags(
        (name = "auth", description = "Авторизация и регистрация пользователей")
    )
)]
pub struct AuthApiDoc;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/login", post(login))
        .route("/api/v1/register/send-code", post(send_verification_code))
        .route("/api/v1/register/confirm", post(confirm_registration))
}

#[utoipa::path(
    post,
    path = "/api/v1/login",
    tag = "auth",
    request_body(content = LoginRequest, description = "Данные для входа", content_type = "application/json"),
    responses(
        (status = 200, description = "Успешный вход", body = ApiResponse<UserOut>),
        (status = 400, description = "Неверные данные", body = ApiResponse<String>),
        (status = 500, description = "Ошибка сервера", body = ApiResponse<String>)
    )
)]
#[axum::debug_handler]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> ApiResponse<UserOut> {
    AuthService::login(payload.email, payload.password, state).await
}

#[utoipa::path(
    post,
    path = "/api/v1/register/send-code",
    tag = "auth",
    request_body(content = SendCodeRequest, description = "Email для отправки кода", content_type = "application/json"),
    responses(
        (status = 200, description = "Код успешно отправлен", body = ApiResponse<String>),
        (status = 409, description = "Пользователь уже существует", body = ApiResponse<String>),
        (status = 500, description = "Ошибка отправки", body = ApiResponse<String>)
    )
)]
#[axum::debug_handler]
pub async fn send_verification_code(
    State(state): State<AppState>,
    Json(payload): Json<SendCodeRequest>,
) -> ApiResponse<String> {
    RegistrationService::send_code(payload.email, state).await
}

#[utoipa::path(
    post,
    path = "/api/v1/register/confirm",
    tag = "auth",
    request_body(content = ConfirmRegistrationRequest, description = "Данные для завершения регистрации", content_type = "application/json"),
    responses(
        (status = 200, description = "Регистрация завершена, выдан токен", body = ApiResponse<UserOut>),
        (status = 400, description = "Неверный код или ошибка", body = ApiResponse<String>),
        (status = 500, description = "Ошибка сервера", body = ApiResponse<String>)
    )
)]
#[axum::debug_handler]
pub async fn confirm_registration(
    State(state): State<AppState>,
    Json(payload): Json<ConfirmRegistrationRequest>,
) -> ApiResponse<UserOut> {
    RegistrationService::confirm(
        payload.email,
        payload.code,
        payload.password,
        state,
    ).await
}