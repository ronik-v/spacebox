use uuid::Uuid;

use crate::core::api::ApiResponse;
use crate::core::errors::users::{DB_ERROR, FAILED_CREATE_TOKEN_ERROR, INVALID_CREDENTIALS_ERROR};
use crate::core::state::AppState;
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

        match repo.get_by_email_password(&email, &password).await {
            Ok(Some(user)) => {
                let token = Uuid::new_v4().hyphenated().to_string();

                match repo.create_token(&user.id, &token).await {
                    Ok(user_token) => {
                        let user_dto = UserDto {
                            id: user.id,
                            email: user.email,
                            base_folder: user.base_folder,
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
}
