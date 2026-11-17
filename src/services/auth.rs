use crate::core::api::ApiResponse;
use crate::core::errors::users::{DB_ERROR, FAILED_CREATE_TOKEN_ERROR, INVALID_CREDENTIALS_ERROR};
use crate::core::security::Password;
use crate::core::state::AppState;
use crate::core::token::get_auth_token;
use crate::dto::users::{UserDto, UserTokensShortDto};
use crate::repositories::users::UsersRepository;
use crate::responses::users::UserOut;


pub struct AuthService;

impl AuthService {
    pub async fn login(
        email: String,
        password: String,
        state: AppState,
    ) -> ApiResponse<UserOut> {
        let repo = UsersRepository::new(&state.db);

        match repo.get_by_email(&email).await {
            Ok(Some(user)) => {
                let token = get_auth_token();
                let password_manager = Password::new(password);
                let is_correct_pass = password_manager.is_correct(&user.password);

                if !is_correct_pass {
                    return ApiResponse::Error {
                        error: crate::core::api::ApiError {
                            message: format!("{}", INVALID_CREDENTIALS_ERROR),
                        },
                    }
                }
                let roles = match repo.get_roles_by_user_id(user.id).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("get roles failed: {e}");
                        return ApiResponse::Error {
                            error: crate::core::api::ApiError { message: DB_ERROR.to_string() },
                        };
                    }
                };

                match repo.create_token(&user.id, &token).await {
                    Ok(user_token) => {
                        let user_dto = UserDto {
                            id: user.id,
                            email: user.email,
                            roles,
                            created_at: user.created_at,
                        };

                        let token_dto = UserTokensShortDto {
                            token: user_token.token,
                            created_at: user_token.created_at,
                        };

                        ApiResponse::Success {
                            data: UserOut {
                                user: user_dto,
                                meta: token_dto,
                            },
                        }
                    }
                    Err(e) => ApiResponse::Error {
                        error: crate::core::api::ApiError {
                            message: format!("{}: {}", FAILED_CREATE_TOKEN_ERROR, e),
                        },
                    },
                }
            }
            Ok(None) => ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: INVALID_CREDENTIALS_ERROR.to_string(),
                },
            },
            Err(e) => ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: format!("{}: {}", DB_ERROR, e),
                },
            },
        }
    }
    pub async fn refresh_token(user_id: i64, state: AppState) -> ApiResponse<UserOut> {
        let repo = UsersRepository::new(&state.db);
        // clear all user tokens and create new
        match repo.get_user_by_id(user_id).await {
            Ok(Some(user)) => {
                match repo.remove_all_tokens(user_id).await {
                    Ok(_) => {
                        let token = get_auth_token();

                        let roles = match repo.get_roles_by_user_id(user.id).await {
                            Ok(r) => r,
                            Err(e) => {
                                tracing::error!("get roles failed: {e}");
                                return ApiResponse::Error {
                                    error: crate::core::api::ApiError { message: DB_ERROR.to_string() },
                                };
                            }
                        };

                        match repo.create_token(&user_id, &token).await {
                            Ok(user_token) => {
                                ApiResponse::Success {
                                    data: UserOut {
                                        user: UserDto {
                                            id: user.id,
                                            email: user.email,
                                            roles,
                                            created_at: user.created_at,
                                        },
                                        meta: UserTokensShortDto {
                                            token: user_token.token,
                                            created_at: user_token.created_at,
                                        },
                                    },
                                }
                            }
                            Err(e) => ApiResponse::Error {
                                error: crate::core::api::ApiError {
                                    message: format!("{}: {}", FAILED_CREATE_TOKEN_ERROR, e),
                                },
                            },
                        }
                    }
                    Err(e) => ApiResponse::Error {
                        error: crate::core::api::ApiError {
                            message: format!("{}: {}", DB_ERROR, e),
                        },
                    },
                }
            }
            Ok(None) => ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: INVALID_CREDENTIALS_ERROR.to_string(),
                },
            },
            Err(e) => ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: format!("{}: {}", DB_ERROR, e),
                },
            },
        }
    }
}
