use std::sync::Arc;
use redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: Arc<ConnectionManager>
}