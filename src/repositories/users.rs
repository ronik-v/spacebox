use sqlx::{Error, PgPool};

use crate::models::user::{User, UserShort, UserTokens, UserWithRoles};

pub struct UsersRepository<'a> {
    db: &'a PgPool
}

impl<'a> UsersRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self { Self { db } }

    pub async fn add(&self, email: &String, password: &String) -> Result<Option<User>, Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users(email, password) VALUES($1, $2) RETURNING *"
        )
            .bind(email)
            .bind(password)
            .fetch_optional(self.db)
            .await
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(self.db)
            .await
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(self.db)
            .await
    }

    pub async fn get_by_token(&self, token: &str) -> Result<Option<UserWithRoles>, Error> {
        sqlx::query_as::<_, UserWithRoles>(
            r#"
            SELECT
                u.id,
                u.email,
                COALESCE(
                    array_agg(r.name) FILTER (WHERE r.name IS NOT NULL),
                    '{}'
                )::text[] AS roles
            FROM user_tokens ut
            JOIN users u ON u.id = ut.user_id
            LEFT JOIN user_roles ur ON ur.user_id = u.id
            LEFT JOIN roles r ON r.id = ur.role_id
            WHERE ut.token = $1
            GROUP BY u.id, u.email
            "#,
        )
            .bind(token)
            .fetch_optional(self.db)
            .await
    }

    pub async fn get_roles_by_user_id(&self, user_id: i64) -> Result<Vec<String>, Error> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT r.name
            FROM roles r
            JOIN user_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = $1
            "#,
        )
            .bind(user_id)
            .fetch_all(self.db)
            .await
    }

    pub async fn change_password(&self, user_id: &i64, new_password: &String) -> Result<(), Error> {
        sqlx::query(
            "UPDATE users SET password = $1 WHERE id = $2"
        )
            .bind(new_password)
            .bind(user_id)
            .execute(self.db)
            .await?;
        Ok(())
    }

    pub async fn create_token(&self, user_id: &i64, token: &String) -> Result<UserTokens, Error> {
        sqlx::query_as::<_, UserTokens>(
            "INSERT INTO user_tokens(user_id, token) VALUES($1, $2) RETURNING *"
        )
            .bind(user_id)
            .bind(token)
            .fetch_one(self.db)
            .await
    }

    pub async fn remove_all_tokens(&self, user_id: i64) -> Result<(), Error> {
        sqlx::query(
            "DELETE FROM user_tokens WHERE user_id = $1"
        )
            .bind(user_id)
            .execute(self.db)
            .await?;
        Ok(())
    }

    pub async fn add_role(&self, user_id: i64, role_id: i64) -> Result<(), Error> {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT (user_id, role_id) DO NOTHING"
        )
            .bind(user_id)
            .bind(role_id)
            .execute(self.db)
            .await?;
        Ok(())
    }

    pub async fn get_role_id_by_name(&self, role_name: &str) -> Result<i64, Error> {
        // get base role for user from 003_ - migration
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM roles WHERE name = $1 LIMIT 1"
        )
            .bind(role_name)
            .fetch_one(self.db)
            .await
    }

    pub async fn has_admin(&self) -> Result<bool, Error> {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1
                FROM users u
                JOIN user_roles ur ON u.id = ur.user_id
                JOIN roles r ON r.id = ur.role_id
                WHERE r.name = 'admin'
            )"
        )
            .fetch_one(self.db)
            .await
    }
}