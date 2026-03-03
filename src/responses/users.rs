use serde::{Deserialize, Serialize};
use crate::dto::users::{UserDto, UserTokensShortDto};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistrationIn {
    pub email: String,
    pub password: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserOut {
    pub user: UserDto,
    pub meta: UserTokensShortDto
}
