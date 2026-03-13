use sqlx::{Error, PgPool};

use crate::models::user::{User, UserShort, UserTokens};

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

    pub async fn get_by_email_password(&self, email: &String, password: &String) -> Result<Option<User>, Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users u WHERE u.email = $1 AND u.password = $2"
        )
            .bind(email)
            .bind(password)
            .fetch_optional(self.db)
            .await
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(self.db)
            .await
    }

    pub async fn get_by_token(&self, token: &String) -> Result<UserShort, Error> {
        sqlx::query_as::<_, UserShort>(
            "SELECT u.id, u.email FROM users u JOIN user_tokens ut ON u.id = ut.user_id WHERE ut.token = $1"
        )
            .bind(token)
            .fetch_one(self.db)
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
}