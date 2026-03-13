use sqlx::{Error, PgPool, Row};
use chrono::NaiveDateTime;

use crate::dto::storage::{DirDto, FileDto, TagDto, UserDirCreateResult};

pub struct StorageRepository<'a> {
    db: &'a PgPool,
    user_id: i64,
}

impl<'a> StorageRepository<'a> {
    pub fn new(db: &'a PgPool, user_id: i64) -> Self {
        Self { db, user_id }
    }

    pub async fn create_directory(
        &self,
        name: &str,
        parent_user_dir_id: Option<i64>,
    ) -> Result<UserDirCreateResult, Error> {
        let parent_dir_id: Option<i64> = if let Some(pud_id) = parent_user_dir_id {
            sqlx::query_scalar::<_, i64>(
                "SELECT dir_id FROM user_dirs WHERE id = $1 AND user_id = $2"
            )
                .bind(pud_id)
                .bind(self.user_id)
                .fetch_optional(self.db)
                .await?
        } else {
            None
        };

        let dir_row = sqlx::query(
            r#"
            INSERT INTO dirs (name, parent_id)
            VALUES ($1, $2)
            RETURNING id, name, parent_id, created_at
            "#
        )
            .bind(name)
            .bind(parent_dir_id)
            .fetch_one(self.db)
            .await?;

        let dir_id: i64 = dir_row.get("id");
        let dir_name: String = dir_row.get("name");
        let dir_parent_id: Option<i64> = dir_row.get("parent_id");
        let dir_created_at: NaiveDateTime = dir_row.get("created_at");

        let user_dir_row = sqlx::query(
            r#"
            INSERT INTO user_dirs (user_id, dir_id, role)
            VALUES ($1, $2, 'OWNER')
            RETURNING id
            "#
        )
            .bind(self.user_id)
            .bind(dir_id)
            .fetch_one(self.db)
            .await?;

        let user_dir_id: i64 = user_dir_row.get("id");

        Ok(UserDirCreateResult {
            user_dir_id,
            dir: DirDto {
                id: dir_id,
                user_dir_id,
                name: dir_name,
                parent_id: dir_parent_id,
                role: "OWNER".to_string(),
                created_at: dir_created_at,
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
    ) -> Result<FileDto, Error> {
        let dir_id_opt: Option<i64> = sqlx::query_scalar::<_, i64>(
            "SELECT dir_id FROM user_dirs WHERE id = $1 AND user_id = $2"
        )
            .bind(user_dir_id)
            .bind(self.user_id)
            .fetch_optional(self.db)
            .await?;

        let dir_id = match dir_id_opt {
            Some(id) => id,
            None => return Err(Error::RowNotFound),
        };

        let file_row = sqlx::query(
            r#"
            INSERT INTO files (name, size, mime_type, storage_key, hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, size, mime_type, storage_key, hash, created_at
            "#
        )
            .bind(name)
            .bind(size)
            .bind(mime_type.as_deref())
            .bind(storage_key)
            .bind(hash)
            .fetch_one(self.db)
            .await?;

        let file_id: i64 = file_row.get("id");
        let file_name: String = file_row.get("name");
        let file_size: i64 = file_row.get("size");
        let file_mime_type: Option<String> = file_row.get("mime_type");
        let file_storage_key: String = file_row.get("storage_key");
        let file_hash: String = file_row.get("hash");
        let file_created_at: NaiveDateTime = file_row.get("created_at");

        sqlx::query(
            "INSERT INTO dir_files (user_dir_id, file_id) VALUES ($1, $2)"
        )
            .bind(user_dir_id)
            .bind(file_id)
            .execute(self.db)
            .await?;

        Ok(FileDto {
            id: file_id,
            name: file_name,
            size: file_size,
            mime_type: file_mime_type,
            storage_key: file_storage_key,
            hash: file_hash,
            created_at: file_created_at,
            tags: vec![],
        })
    }

    pub async fn get_file_by_storage_key(&self, storage_key: &str) -> Result<Option<FileDto>, Error> {
        let rows = sqlx::query(
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
            "#
        )
            .bind(storage_key)
            .bind(self.user_id)
            .fetch_all(self.db)
            .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let first = &rows[0];
        let file_id: i64 = first.get("id");
        let file_name: String = first.get("name");
        let file_size: i64 = first.get("size");
        let file_mime_type: Option<String> = first.get("mime_type");
        let file_storage_key: String = first.get("storage_key");
        let file_hash: String = first.get("hash");
        let file_created_at: NaiveDateTime = first.get("created_at");

        let mut tags = vec![];
        for row in rows {
            let tag_id: Option<i64> = row.get("tag_id");
            let tag_name: Option<String> = row.get("tag_name");
            let tag_created_at: Option<NaiveDateTime> = row.get("tag_created_at");

            if let (Some(id), Some(name), Some(created)) = (tag_id, tag_name, tag_created_at) {
                tags.push(TagDto { id, name, created_at: created });
            }
        }

        Ok(Some(FileDto {
            id: file_id,
            name: file_name,
            size: file_size,
            mime_type: file_mime_type,
            storage_key: file_storage_key,
            hash: file_hash,
            created_at: file_created_at,
            tags,
        }))
    }

    pub async fn delete_file_by_storage_key(&self, storage_key: &str) -> Result<(), Error> {
        let file_id_opt: Option<i64> = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM files WHERE storage_key = $1"
        )
            .bind(storage_key)
            .fetch_optional(self.db)
            .await?;

        if let Some(file_id) = file_id_opt {
            sqlx::query("DELETE FROM dir_files WHERE file_id = $1")
                .bind(file_id)
                .execute(self.db)
                .await?;

            sqlx::query("DELETE FROM files WHERE id = $1")
                .bind(file_id)
                .execute(self.db)
                .await?;
        }

        Ok(())
    }

    pub async fn rename_directory(&self, user_dir_id: i64, new_name: &str) -> Result<DirDto, Error> {
        let row = sqlx::query(
            r#"
            UPDATE dirs
            SET name = $1
            FROM user_dirs ud
            WHERE ud.id = $2
              AND ud.user_id = $3
              AND ud.dir_id = dirs.id
            RETURNING dirs.id, dirs.name, dirs.parent_id, dirs.created_at, ud.id AS user_dir_id, ud.role
            "#
        )
            .bind(new_name)
            .bind(user_dir_id)
            .bind(self.user_id)
            .fetch_one(self.db)
            .await?;

        Ok(DirDto {
            id: row.get("id"),
            user_dir_id: row.get("user_dir_id"),
            name: row.get("name"),
            parent_id: row.get("parent_id"),
            role: row.get("role"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn get_user_root_dir_id(&self) -> Result<i64, Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT ud.id
            FROM user_dirs ud
            JOIN dirs d ON ud.dir_id = d.id
            WHERE ud.user_id = $1
              AND d.parent_id IS NULL
            "#
        )
            .bind(self.user_id)
            .fetch_one(self.db)
            .await
    }

    pub async fn get_directory_fs_path(&self, user_dir_id: i64) -> Result<String, Error> {
        sqlx::query_scalar::<_, String>(
            r#"
            WITH RECURSIVE dir_path AS (
                SELECT d.id, d.name, d.parent_id, 0 AS level
                FROM dirs d
                JOIN user_dirs ud ON d.id = ud.dir_id
                WHERE ud.id = $1
                  AND ud.user_id = $2
                UNION ALL
                SELECT d.id, d.name, d.parent_id, dp.level + 1
                FROM dirs d
                JOIN dir_path dp ON d.id = dp.parent_id
            )
            SELECT COALESCE(string_agg(name, '/' ORDER BY level DESC), '') AS path
            FROM dir_path
            "#
        )
            .bind(user_dir_id)
            .bind(self.user_id)
            .fetch_one(self.db)
            .await
    }

    pub async fn get_file_storage_key(&self, file_id: i64) -> Result<Option<String>, Error> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT f.storage_key
            FROM files f
            JOIN dir_files df ON f.id = df.file_id
            JOIN user_dirs ud ON df.user_dir_id = ud.id
            WHERE f.id = $1 AND ud.user_id = $2
            "#
        )
            .bind(file_id)
            .bind(self.user_id)
            .fetch_optional(self.db)
            .await
    }

    pub async fn delete_directory(&self, user_dir_id: i64) -> Result<(), Error> {
        let dir_id: Option<i64> = sqlx::query_scalar::<_, i64>(
            "SELECT dir_id FROM user_dirs WHERE id = $1 AND user_id = $2"
        )
            .bind(user_dir_id)
            .bind(self.user_id)
            .fetch_optional(self.db)
            .await?;
        let dir_id = dir_id.ok_or(Error::RowNotFound)?;

        sqlx::query("DELETE FROM user_dirs WHERE id = $1")
            .bind(user_dir_id)
            .execute(self.db)
            .await?;

        sqlx::query("DELETE FROM dirs WHERE id = $1")
            .bind(dir_id)
            .execute(self.db)
            .await?;

        Ok(())
    }
}