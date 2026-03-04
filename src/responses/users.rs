use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::users::{UserDto, UserTokensShortDto};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserRegistrationIn {
    pub email: String,
    pub password: String
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserOut {
    pub user: UserDto,
    pub meta: UserTokensShortDto
}
