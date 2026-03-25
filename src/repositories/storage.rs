use sqlx::{Error, PgPool, Row};
use chrono::NaiveDateTime;

use crate::dto::storage::{DirDto, DirFilesDto, DirsListDto, FileDto, UserDirCreateResult};

pub struct StorageRepository<'a> {
    db: &'a PgPool,
    user_id: i64,
}

impl<'a> StorageRepository<'a> {
    pub fn new(db: &'a PgPool, user_id: i64) -> Self {
        Self { db, user_id }
    }

    pub async fn get_user_dir_id(&self, dir_id: &i64) -> Result<i64, Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM user_dirs WHERE dir_id = $1 AND user_id = $2"
        )
            .bind(*dir_id)
            .bind(self.user_id)
            .fetch_one(self.db)
            .await
    }

    // IMPORTANT: ORDER BY parent_id DESC NULLS LAST
    // is needed for the correct operation of the iterative tree construction below!
    pub async fn get_dirs_list_with_files(&self) -> Result<Vec<DirsListDto>, Error> {
        let dir_rows = sqlx::query(
            r#"SELECT
                ud.dir_id,
                d.name AS dir_name,
                d.parent_id,
                d.created_at AS dir_created,
                CASE
                    WHEN MAX(f.id) IS NOT null THEN JSONB_AGG(JSONB_BUILD_OBJECT(
                    'file_id', f.id, 'file_name', f.name, 'file_size', f.size,
                    'file_path', f.storage_key, 'file_hash', f.hash, 'created_at', f.created_at
                )) ELSE NULL END
                AS dir_files
            FROM user_dirs ud
            JOIN dirs d ON ud.dir_id = d.id
            LEFT JOIN dir_files df ON df.user_dir_id = ud.dir_id
            LEFT JOIN files f ON f.id = df.file_id
            WHERE ud.user_id = $1
            GROUP BY
                ud.dir_id,
                d.name,
                d.parent_id,
                d.created_at
            ORDER BY d.parent_id DESC NULLS LAST"#
        )
            .bind(self.user_id)
            .fetch_all(self.db)
            .await?;

        let mut dirs: Vec<DirsListDto> = Vec::new();

        for row in dir_rows {
            dirs.push(DirsListDto {
                dir_id: row.get("dir_id"),
                dir_name: row.get("dir_name"),
                parent_id: row.get("parent_id"),
                dir_files: row.get("dir_files"),
            });
        }

        Ok(dirs)
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

        Ok(UserDirCreateResult {
            dir: DirDto {
                id: dir_id,
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
            .bind(dir_id)
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

    pub async fn get_directory_fs_path(&self, dir_id: i64) -> Result<String, Error> {
        sqlx::query_scalar::<_, String>(
            r#"
            WITH RECURSIVE dir_path AS (
                SELECT d.id, d.name, d.parent_id, 0 AS level
                FROM dirs d
                JOIN user_dirs ud ON d.id = ud.dir_id
                WHERE d.id = $1
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
            .bind(dir_id)
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