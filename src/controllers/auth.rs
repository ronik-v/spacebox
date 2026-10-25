use axum::{
    extract::{State, Json},
    routing::post,
    Router,
};
use axum::extract::Path;
use axum::routing::delete;
use http::HeaderMap;
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
use crate::core::token::TokenExtractManager;
use crate::repositories::users::UsersRepository;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::auth::login,
        crate::controllers::auth::send_verification_code,
        crate::controllers::auth::confirm_registration,
        crate::controllers::auth::refresh_auth,
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
        .route("/api/v1/token/refresh/{user_id}", delete(refresh_auth))
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

#[utoipa::path(
    delete,
    path = "/api/v1/token/refresh/{user_id}",
    tag = "auth",
    params(
        ("user_id" = i64, Path, description = "ID пользователя")
    ),
    responses(
        (status = 200, description = "Успешно удалены старые токены и выданы новые", body = ApiResponse<UserOut>),
        (status = 401, description = "Неавторизованный доступ", body = ApiResponse<ApiError>),
        (status = 403, description = "Доступ запрещен (ID не совпадает)", body = ApiResponse<ApiError>),
        (status = 500, description = "Ошибка сервера", body = ApiResponse<ApiError>)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[axum::debug_handler]
pub async fn refresh_auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> ApiResponse<UserOut> {
    let token_manager = TokenExtractManager::new(headers);

    let token = match token_manager.check_token_error() {
        Ok(t) => t,
        Err(_) => {
            return ApiResponse::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };

    let users_repo = UsersRepository::new(&state.db);

    let user_short = match users_repo.get_by_token(&token).await {
        Ok(u) => u,
        Err(_) => {
            return ApiResponse::Error {
                error: ApiError { message: "Invalid or expired token".to_string() },
            };
        }
    };
    // check that user id correct
    if user_short.id != user_id {
        return ApiResponse::Error {
            error: ApiError { message: "Permission denied: access token owner mismatch".to_string() },
        };
    }

    AuthService::refresh_token(user_id, state).await
}
