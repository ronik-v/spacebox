use sqlx::{PgPool, Error as SqlxError};
use chrono::{DateTime, Utc};

use crate::dto::storage::{DirDto, FileDto, TagDto, UserDirCreateResult};

pub struct StorageRepository<'a> {
    db: &'a PgPool,
    user_id: i64,
}

impl<'a> StorageRepository<'a> {
    pub fn new(db: &'a PgPool, user_id: i64) -> Self {
        Self { db, user_id }
    }

    pub async fn get_user_base_folder(&self) -> Result<String, SqlxError> {
        sqlx::query_scalar!(
            "SELECT base_folder FROM users WHERE id = $1",
            self.user_id
        )
            .fetch_one(self.db)
            .await
    }

    pub async fn create_directory(
        &self,
        name: &str,
        parent_user_dir_id: Option<i64>,
    ) -> Result<UserDirCreateResult, SqlxError> {
        let parent_dir_id: Option<i64> = if let Some(pud_id) = parent_user_dir_id {
            sqlx::query_scalar!(
                "SELECT dir_id FROM user_dirs WHERE id = $1 AND user_id = $2",
                pud_id,
                self.user_id
            )
                .fetch_optional(self.db)
                .await?
        } else {
            None
        };

        let dir_row = sqlx::query!(
            r#"
            INSERT INTO dirs (name, parent_id)
            VALUES ($1, $2)
            RETURNING id, name, parent_id, created_at
            "#,
            name,
            parent_dir_id
        )
            .fetch_one(self.db)
            .await?;

        let user_dir_row = sqlx::query!(
            r#"
            INSERT INTO user_dirs (user_id, dir_id, role)
            VALUES ($1, $2, 'OWNER')
            RETURNING id
            "#,
            self.user_id,
            dir_row.id
        )
            .fetch_one(self.db)
            .await?;

        Ok(UserDirCreateResult {
            user_dir_id: user_dir_row.id,
            dir: DirDto {
                id: dir_row.id,
                user_dir_id: user_dir_row.id,
                name: dir_row.name,
                parent_id: dir_row.parent_id,
                role: "OWNER".to_string(),
                created_at: dir_row.created_at,
            },
        })
    }

    pub async fn create_file(
        &self,
        name: &str,
        size: i64,
        mime_type: Option<String>,
        storage_key: &str,
        hash: &str,
        user_dir_id: i64,
    ) -> Result<FileDto, SqlxError> {
        let dir_id_opt: Option<i64> = sqlx::query_scalar!(
            "SELECT dir_id FROM user_dirs WHERE id = $1 AND user_id = $2",
            user_dir_id,
            self.user_id
        )
            .fetch_optional(self.db)
            .await?;

        let dir_id = match dir_id_opt {
            Some(id) => id,
            None => return Err(SqlxError::RowNotFound),
        };

        let row = sqlx::query!(
            r#"
            INSERT INTO files (name, size, mime_type, storage_key, hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, size, mime_type, storage_key, hash, created_at
            "#,
            name,
            size,
            mime_type.as_deref(),
            storage_key,
            hash
        )
            .fetch_one(self.db)
            .await?;

        sqlx::query!(
            "INSERT INTO dir_files (user_dir_id, file_id) VALUES ($1, $2)",
            user_dir_id,
            row.id
        )
            .execute(self.db)
            .await?;

        Ok(FileDto {
            id: row.id,
            name: row.name,
            size: row.size,
            mime_type: row.mime_type,
            storage_key: row.storage_key,
            hash: row.hash,
            created_at: row.created_at,
            tags: vec![],
        })
    }

    pub async fn get_file_by_storage_key(&self, storage_key: &str) -> Result<Option<FileDto>, SqlxError> {
        let rows = sqlx::query!(
            r#"
            SELECT
                f.id,
                f.name,
                f.size,
                f.mime_type,
                f.storage_key,
                f.hash,
                f.created_at,
                t.id AS tag_id,
                t.name AS tag_name,
                t.created_at AS tag_created_at
            FROM files f
            LEFT JOIN file_tags ft ON f.id = ft.file_id
            LEFT JOIN tags t ON ft.tag_id = t.id
            WHERE f.storage_key = $1
              AND EXISTS (
                  SELECT 1
                  FROM dir_files df
                  JOIN user_dirs ud ON df.user_dir_id = ud.id
                  WHERE df.file_id = f.id
                    AND ud.user_id = $2
              )
            ORDER BY t.name
            "#,
            storage_key,
            self.user_id
        )
            .fetch_all(self.db)
            .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let first = &rows[0];
        let mut tags = vec![];

        for row in rows {
            if let (Some(tag_id), Some(tag_name), Some(tag_created_at)) = (row.tag_id, row.tag_name, row.tag_created_at) {
                tags.push(TagDto {
                    id: tag_id,
                    name: tag_name,
                    created_at: tag_created_at,
                });
            }
        }

        Ok(Some(FileDto {
            id: first.id,
            name: first.name,
            size: first.size,
            mime_type: first.mime_type,
            storage_key: first.storage_key,
            hash: first.hash,
            created_at: first.created_at,
            tags,
        }))
    }

    pub async fn delete_file_by_storage_key(&self, storage_key: &str) -> Result<(), SqlxError> {
        let file_id_opt: Option<i64> = sqlx::query_scalar!(
            "SELECT id FROM files WHERE storage_key = $1",
            storage_key
        )
            .fetch_optional(self.db)
            .await?;

        if let Some(file_id) = file_id_opt {
            sqlx::query!("DELETE FROM dir_files WHERE file_id = $1", file_id)
                .execute(self.db)
                .await?;

            sqlx::query!("DELETE FROM files WHERE id = $1", file_id)
                .execute(self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn rename_directory(&self, user_dir_id: i64, new_name: &str) -> Result<DirDto, SqlxError> {
        let row = sqlx::query!(
            r#"
            UPDATE dirs
            SET name = $1
            FROM user_dirs ud
            WHERE ud.id = $2
              AND ud.user_id = $3
              AND ud.dir_id = dirs.id
            RETURNING dirs.id, dirs.name, dirs.parent_id, dirs.created_at, ud.id AS user_dir_id, ud.role
            "#,
            new_name,
            user_dir_id,
            self.user_id
        )
            .fetch_one(self.db)
            .await?;

        Ok(DirDto {
            id: row.id,
            user_dir_id: row.user_dir_id,
            name: row.name,
            parent_id: row.parent_id,
            role: row.role,
            created_at: row.created_at,
        })
    }

    pub async fn get_directory_contents(&self, user_dir_id: i64) -> Result<(Vec<DirDto>, Vec<FileDto>), SqlxError> {
        let dirs = sqlx::query!(
            r#"
            SELECT
                d.id,
                ud.id AS user_dir_id,
                d.name,
                d.parent_id,
                ud.role,
                d.created_at
            FROM dirs d
            JOIN user_dirs ud ON d.id = ud.dir_id
            WHERE ud.user_id = $1
              AND d.parent_id = (
                  SELECT dir_id FROM user_dirs WHERE id = $2
              )
            "#,
            self.user_id,
            user_dir_id
        )
            .fetch_all(self.db)
            .await?
            .into_iter()
            .map(|r| DirDto {
                id: r.id,
                user_dir_id: r.user_dir_id,
                name: r.name,
                parent_id: r.parent_id,
                role: r.role,
                created_at: r.created_at,
            })
            .collect();

        let files = sqlx::query!(
            r#"
            SELECT
                f.id,
                f.name,
                f.size,
                f.mime_type,
                f.storage_key,
                f.hash,
                f.created_at,
                COALESCE(ARRAY_AGG(t.name) FILTER (WHERE t.name IS NOT NULL), ARRAY[]::text[]) AS tags
            FROM files f
            JOIN dir_files df ON f.id = df.file_id
            LEFT JOIN file_tags ft ON f.id = ft.file_id
            LEFT JOIN tags t ON ft.tag_id = t.id
            WHERE df.user_dir_id = $1
            GROUP BY f.id
            "#,
            user_dir_id
        )
            .fetch_all(self.db)
            .await?
            .into_iter()
            .map(|r| FileDto {
                id: r.id,
                name: r.name,
                size: r.size,
                mime_type: r.mime_type,
                storage_key: r.storage_key,
                hash: r.hash,
                created_at: r.created_at,
                tags: r.tags.unwrap_or_default().into_iter().map(|name| TagDto {
                    id: 0,
                    name,
                    created_at: Utc::now(),
                }).collect(),
            })
            .collect();

        Ok((dirs, files))
    }
}