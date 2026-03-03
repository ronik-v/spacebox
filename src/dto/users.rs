use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserDto {
    pub id: i64,
    pub email: String,
    pub base_folder: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserShortDto {
    pub id: i64,
    pub email: String,
    pub base_folder: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserTokensDto {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserTokensShortDto {
    pub token: String,
    pub created_at: NaiveDateTime,
}