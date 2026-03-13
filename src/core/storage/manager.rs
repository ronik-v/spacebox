use std::path::PathBuf;
use async_trait::async_trait;
use axum::extract::Multipart;
use sha2::{Digest, Sha256};
use tokio::fs::{File, remove_file, create_dir_all, remove_dir, rename};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::core::storage::traits::StorageManager;
use crate::core::storage::types::{
    DirInfo, FileData, FileInfo, StorageError,
    OP_RENAME_DIR, OP_CREATE_DIR, OP_REMOVE_DIR, OP_FILE_CREATE, OP_MULTIPART,
    OP_CHUNK_READ, OP_FILE_OPEN, OP_REMOVE_FILE, OP_WRITE_CHUNK, ERR_CHUNK_READ,
    ERR_MULTIPART, ERR_CREATE_DIR, ERR_REMOVE_DIR, ERR_RENAME_DIR, ERR_REMOVE_FILE,
    ERR_FILE_CREATE, ERR_WRITE_CHUNK, ERR_FILE_OPEN
};

pub struct Storage {
    root_path: PathBuf
}

impl Storage {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }
}

#[async_trait]
impl StorageManager for Storage {
    async fn upload_file(&self, mut multipart: Multipart, path_to_upload: &str) -> Result<FileInfo, StorageError> {
        let mut file_name: String = String::new();
        let mut file_size: i64 = 0;
        let mut content_type: Option<String> = None;
        let mut hasher = Sha256::new();
        let mut saved_path: Option<PathBuf> = None; // ← новая переменная для финального пути

        while let Some(mut field) = multipart.next_field().await.map_err(|e| StorageError {
            operation: OP_MULTIPART.to_string(),
            message: ERR_MULTIPART.to_string(),
            path: path_to_upload.to_string(),
        })? {
            file_name = field.file_name().unwrap_or_default().to_string();
            content_type = field.content_type().map(|s| s.to_string());

            let file_full_path = self.root_path.join(path_to_upload.to_string()).join(file_name);
            let file_path_str = file_full_path.to_string_lossy().to_string();

            let mut file = File::create(file_full_path.clone()).await.map_err(|e| StorageError {
                operation: OP_FILE_CREATE.to_string(),
                message: ERR_FILE_CREATE.to_string(),
                path: file_path_str.clone(),
            })?;

            while let Some(chunk) = field.chunk().await.map_err(|e| StorageError {
                operation: OP_CHUNK_READ.to_string(),
                message: ERR_CHUNK_READ.to_string(),
                path: file_path_str.clone(),
            })? {
                file.write_all(&chunk).await.map_err(|e| StorageError {
                    operation: OP_WRITE_CHUNK.to_string(),
                    message: ERR_WRITE_CHUNK.to_string(),
                    path: file_path_str.clone(),
                })?;

                hasher.update(chunk.clone());
                file_size += chunk.len() as i64;
            }
            saved_path = Some(file_full_path);
        }

        let hash = format!("{:x}", hasher.finalize());
        let path = saved_path.ok_or_else(|| StorageError {
            operation: OP_MULTIPART.to_string(),
            message: "No file was uploaded".to_string(),
            path: path_to_upload.to_string(),
        })?;

        Ok(FileInfo {
            path,
            size: file_size,
            hash,
            mime_type: content_type,
        })
    }

    async fn read_file(&self, file_path: &str) -> Result<FileData, StorageError> {
        let file_path_address = self.root_path.join(file_path.to_string());
        let file_path_str = file_path_address.to_string_lossy().to_string();

        let file = File::open(file_path_address.clone()).await.map_err(|e| StorageError {
            operation: OP_FILE_OPEN.to_string(),
            message: ERR_FILE_OPEN.to_string(),
            path: file_path_str.clone(),
        })?;
        let stream = ReaderStream::new(file);

        Ok(FileData {
            path: file_path_address.clone(),
            data: stream,
        })
    }

    async fn remove_file(&self, file_path: &str) -> Result<(), StorageError> {
        let file_path_address = self.root_path.join(file_path.to_string());
        let file_path_str = file_path_address.to_string_lossy().to_string();

        remove_file(file_path_address).await.map_err(|e| StorageError {
            operation: OP_REMOVE_FILE.to_string(),
            message: ERR_REMOVE_FILE.to_string(),
            path: file_path_str,
        })?;

        Ok(())
    }

    async fn create_dir(&self, dir_name: &str) -> Result<DirInfo, StorageError> {
        let dir_path = self.root_path.join(dir_name.to_string());
        let dir_path_str = dir_path.to_string_lossy().to_string();

        create_dir_all(dir_path).await.map_err(|e| StorageError {
            operation: OP_CREATE_DIR.to_string(),
            message: ERR_CREATE_DIR.to_string(),
            path: dir_path_str,
        })?;

        Ok(DirInfo { path: dir_name.to_string() })
    }

    async fn remove_dir(&self, dir_name: &str) -> Result<(), StorageError> {
        let dir_path = self.root_path.join(dir_name.to_string());
        let dir_path_str = dir_path.to_string_lossy().to_string();

        remove_dir(dir_path).await.map_err(|e| StorageError {
            operation: OP_REMOVE_DIR.to_string(),
            message: ERR_REMOVE_DIR.to_string(),
            path: dir_path_str,
        })?;

        Ok(())
    }

    async fn change_name_dir(&self, dir_name: &str, new_dir_name: &str) -> Result<DirInfo, StorageError> {
        let from = self.root_path.join(dir_name.to_string());
        let to = self.root_path.join(new_dir_name.to_string());
        let from_str = from.to_string_lossy().to_string();

        rename(from, to).await.map_err(|e| StorageError {
            operation: OP_RENAME_DIR.to_string(),
            message: ERR_RENAME_DIR.to_string(),
            path: from_str,
        })?;

        Ok(DirInfo { path: new_dir_name.to_string() })
    }
}