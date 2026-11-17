use sqlx::PgPool;

use crate::core::api::ApiError;
use crate::core::errors::users::DB_ERROR;
use crate::repositories::users::UsersRepository;

pub struct RolesService<'a> {
    repo: UsersRepository<'a>,
}

impl<'a> RolesService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { repo: UsersRepository::new(db) }
    }

    pub async fn get_user_roles(&self, user_id: i64) -> Result<Vec<String>, ApiError> {
        self.repo.get_roles_by_user_id(user_id).await.map_err(|e| {
            tracing::error!("get roles failed: {e}");
            ApiError::new(DB_ERROR)
        })
    }
}