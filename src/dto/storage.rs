use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct DirDto {
    pub id: i64,
    pub user_dir_id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TagDto {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FileDto {
    pub id: i64,
    pub name: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub storage_key: String,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<TagDto>,
}

#[derive(Debug, Clone)]
pub struct UserDirCreateResult {
    pub user_dir_id: i64,
    pub dir: DirDto,
}