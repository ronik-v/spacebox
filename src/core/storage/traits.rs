use async_trait::async_trait;
use axum::extract::Multipart;

use crate::core::storage::types::{DirInfo, FileData, FileInfo, StorageError};

// I/O basic disk operation for storage[service] api
#[async_trait]
pub trait StorageManager: Send + Sync + 'static {
    async fn upload_file(&self, multipart: Multipart, path_to_upload: &str) -> Result<FileInfo, StorageError>;

    async fn read_file(&self, file_path: &str) -> Result<FileData, StorageError>;

    async fn remove_file(&self, file_path: &str) -> Result<(), StorageError>;

    async fn create_dir(&self, dir_name: &str) -> Result<DirInfo, StorageError>;

    async fn remove_dir(&self, dir_name: &str) -> Result<(), StorageError>;

    async fn change_name_dir(&self, dir_name: &str, new_dir_name: &str) -> Result<DirInfo, StorageError>;
}