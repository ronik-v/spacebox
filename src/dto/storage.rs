use chrono::NaiveDateTime;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DirDto {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub role: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TagDto {
    pub id: i64,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FileDto {
    pub id: i64,
    pub name: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub storage_key: String,
    pub hash: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
    pub tags: Vec<TagDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserDirCreateResult {
    pub dir: DirDto,
}