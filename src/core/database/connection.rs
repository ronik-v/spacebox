use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use redis::{Client, aio::ConnectionManager};
use anyhow::Result;
use crate::config::AppConfig;

pub async fn create_connection_db(cfg: &AppConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(cfg.database_max_connections)
        .min_connections(cfg.database_min_connections)
        .connect(&cfg.database_uri)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub async fn create_connection_redis(cfg: &AppConfig) -> Result<ConnectionManager> {
    let client = Client::open(cfg.redis_uri.as_str())?;
    let manager = ConnectionManager::new(client).await?;

    Ok(manager)
}