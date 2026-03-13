use std::path::PathBuf;
use axum::extract::Multipart;
use sqlx::PgPool;

use crate::core::storage::manager::Storage;
use crate::core::storage::traits::StorageManager;
use crate::repositories::storage::StorageRepository;
use crate::dto::storage::{DirDto, FileDto, UserDirCreateResult};

pub struct StorageService<'a> {
    user_id: i64,
    storage_repository: StorageRepository<'a>,
    storage_manager: Box<dyn StorageManager>,
    root_path: PathBuf,
}

impl<'a> StorageService<'a> {
    pub fn new(user_id: i64, db: &'a PgPool, root_path: PathBuf) -> Self {
        Self {
            user_id,
            storage_repository: StorageRepository::new(db, user_id),
            storage_manager: Box::new(Storage::new(root_path.clone())),
            root_path,
        }
    }

    pub async fn upload_file(&self, multipart: Multipart, _path_to_upload: &str, dir_id: i64) -> Result<FileDto, String> {
        let fs_dir_path = self.storage_repository.get_directory_fs_path(dir_id).await.map_err(|e| e.to_string())?;
        self.storage_manager.create_dir(&fs_dir_path).await.map_err(|e| e.message.clone())?;
        let file_info = self.storage_manager.upload_file(multipart, &fs_dir_path).await.map_err(|e| e.message.clone())?;
        let file_name = file_info.path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let storage_key = if fs_dir_path.is_empty() {
            file_name.clone()
        } else {
            format!("{}/{}", fs_dir_path, file_name)
        };
        let file_dto = match self.storage_repository.create_file(
            &file_name,
            file_info.size,
            file_info.mime_type,
            &storage_key,
            &file_info.hash,
            dir_id,
        ).await {
            Ok(dto) => dto,
            Err(e) => {
                let _ = self.storage_manager.remove_file(&storage_key).await;
                return Err(e.to_string());
            }
        };
        Ok(file_dto)
    }

    pub async fn remove_file(&self, file_id: i64) -> Result<(), String> {
        let storage_key_opt = self.storage_repository.get_file_storage_key(file_id).await.map_err(|e| e.to_string())?;
        let storage_key = match storage_key_opt {
            Some(key) => key,
            None => return Err("File not found or access denied".to_string()),
        };
        self.storage_manager.remove_file(&storage_key).await.map_err(|e| e.message.clone())?;
        self.storage_repository.delete_file_by_storage_key(&storage_key).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn create_dir(&self, name: &str) -> Result<UserDirCreateResult, String> {
        let root_user_dir_id = self.storage_repository.get_user_root_dir_id().await.map_err(|e| e.to_string())?;
        let parent_fs_path = self.storage_repository.get_directory_fs_path(root_user_dir_id).await.map_err(|e| e.to_string())?;
        let new_fs_path = if parent_fs_path.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", parent_fs_path, name)
        };
        self.storage_manager.create_dir(&new_fs_path).await.map_err(|e| e.message.clone())?;
        let create_result = match self.storage_repository.create_directory(name, Some(root_user_dir_id)).await {
            Ok(res) => res,
            Err(e) => {
                let _ = self.storage_manager.remove_dir(&new_fs_path).await;
                return Err(e.to_string());
            }
        };
        Ok(create_result)
    }

    pub async fn remove_dir(&self, dir_id: i64) -> Result<(), String> {
        let fs_path = self.storage_repository.get_directory_fs_path(dir_id).await.map_err(|e| e.to_string())?;
        self.storage_manager.remove_dir(&fs_path).await.map_err(|e| e.message.clone())?;
        self.storage_repository.delete_directory(dir_id).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn rename_dir(&self, dir_id: i64, new_name: &str) -> Result<DirDto, String> {
        let old_fs_path = self.storage_repository.get_directory_fs_path(dir_id).await.map_err(|e| e.to_string())?;
        let parent_fs_path = if let Some(pos) = old_fs_path.rfind('/') {
            old_fs_path[0..pos].to_string()
        } else {
            String::new()
        };
        let new_fs_path = if parent_fs_path.is_empty() {
            new_name.to_string()
        } else {
            format!("{}/{}", parent_fs_path, new_name)
        };
        self.storage_manager.change_name_dir(&old_fs_path, &new_fs_path).await.map_err(|e| e.message.clone())?;
        let dir_dto = match self.storage_repository.rename_directory(dir_id, new_name).await {
            Ok(dto) => dto,
            Err(e) => {
                let _ = self.storage_manager.change_name_dir(&new_fs_path, &old_fs_path).await;
                return Err(e.to_string());
            }
        };
        Ok(dir_dto)
    }
}