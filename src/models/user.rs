use serde::Serialize;
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub base_folder: Option<String>,
    pub created_at: NaiveDateTime
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserShort {
    pub id: i64,
    pub email: String,
    pub base_folder: Option<String>
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserTokens {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub created_at: NaiveDateTime
}