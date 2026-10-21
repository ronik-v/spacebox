use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DirFilesDto {
    pub file_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub file_path: String,
    pub file_hash: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirsListDto {
    pub dir_id: i64,
    pub dir_name: String,
    pub parent_id: Option<i64>,
    pub dir_files: Option<Json<Vec<DirFilesDto>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubDirsListDto {
    pub dir_id: i64,
    pub dir_name: String,
    pub parent_id: Option<i64>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserDirsStructureWithFilesDto {
    pub dir_id: i64,
    pub dir_name: String,
    pub parent_id: Option<i64>,
    #[schema(value_type = Option<Vec<DirFilesDto>>)]
    pub dir_files: Option<Json<Vec<DirFilesDto>>>,
    #[schema(value_type = Option<Vec<SubDirsListDto>>)]
    pub sub_dirs: Option<Json<Vec<SubDirsListDto>>>,
}