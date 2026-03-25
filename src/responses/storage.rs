use serde::Serialize;
use utoipa::ToSchema;

use crate::dto::storage::DirFilesDto;

// realise folder structure like node list
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DirObjectWithFilesNode {
    pub id: i64,
    pub name: String,
    pub files: Option<Vec<DirFilesDto>>,
    #[schema(no_recursion)]
    pub sub_dir: Option<Vec<DirObjectWithFilesNode>>
}