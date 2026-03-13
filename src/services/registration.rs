use rand::{Rng, thread_rng};

use crate::core::api::ApiResponse;
use crate::core::errors::users::{DB_ERROR, FAILED_CREATE_TOKEN_ERROR, FAILED_CREATE_USER_ERROR, FAILED_SEND_CODE_ERROR, FAILED_STORE_CODE_ERROR, UNCORRECTED_VERIFICATION_CODE_ERROR, USER_IS_EXISTS_ERROR, VERIFICATION_ERROR};
use crate::core::state::AppState;
use crate::core::token::get_auth_token;
use crate::dto::users::{UserDto, UserTokensShortDto};
use crate::repositories::users::UsersRepository;
use crate::responses::users::UserOut;
use crate::services::verification::RegistrationVerification;


pub struct RegistrationService;

impl RegistrationService {
    pub async fn send_code(
        email: String,
        state: AppState,
    ) -> ApiResponse<String> {
        let repo = UsersRepository::new(&state.db);

        if let Ok(Some(_)) = repo.get_by_email(&email).await {
            return ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: USER_IS_EXISTS_ERROR.to_string(),
                },
            };
        }

        let code: String = (0..6)
            .map(|_| thread_rng().gen_range(0..=9).to_string())
            .collect();

        let ver = RegistrationVerification::new(
            &state.cfg,
            email.clone(),
            code,
            state.redis.clone(),
        );

        if ver.send_code_verification().await.is_err() {
            return ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: FAILED_SEND_CODE_ERROR.to_string(),
                },
            };
        }

        if ver.set_code_verification_in_redis().await.is_err() {
            return ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: FAILED_STORE_CODE_ERROR.to_string(),
                }
            }
        }

        ApiResponse::Success {
            data: "Код верификации успешно отправлен на ваш email".to_string(),
        }
    }

    pub async fn confirm(
        email: String,
        code: String,
        password: String,
        state: AppState,
    ) -> ApiResponse<UserOut> {
        let ver = RegistrationVerification::new(
            &state.cfg,
            email.clone(),
            code,
            state.redis.clone(),
        );

        match ver.is_correct_verification_code().await {
            Ok(true) => {
                let repo = UsersRepository::new(&state.db);

                match repo.add(&email, &password).await {
                    Ok(Some(user)) => {
                        let token = get_auth_token();

                        match repo.create_token(&user.id, &token).await {
                            Ok(user_token) => {
                                let user_dto = UserDto {
                                    id: user.id,
                                    email: user.email,
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
                            message: FAILED_CREATE_USER_ERROR.to_string(),
                        },
                    },
                    Err(e) => ApiResponse::Error {
                        error: crate::core::api::ApiError {
                            message: format!("{}: {}", DB_ERROR, e),
                        },
                    },
                }
            }
            Ok(false) => ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: UNCORRECTED_VERIFICATION_CODE_ERROR.to_string(),
                },
            },
            Err(e) => ApiResponse::Error {
                error: crate::core::api::ApiError {
                    message: format!("{}: {}", VERIFICATION_ERROR, e),
                },
            },
        }
    }
}