use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserDto {
    pub id: i64,
    pub email: String,
    pub base_folder: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserShortDto {
    pub id: i64,
    pub email: String,
    pub base_folder: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserTokensDto {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserTokensShortDto {
    pub token: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
}