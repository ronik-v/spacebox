use std::sync::Arc;
use redis::aio::ConnectionManager;
use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<AppConfig>,
    pub db: sqlx::PgPool,
    pub redis: Arc<ConnectionManager>
}