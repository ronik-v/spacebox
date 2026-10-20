use std::process::exit;
use std::time::Duration;
use redis::aio::ConnectionManager;
use redis::cmd;
use sqlx::PgPool;

pub async fn is_db_alive(conn: &PgPool) {
    if conn.is_closed() {
        println!("Db connection close");
        exit(0)
    }
}

pub async fn is_redis_alive(conn: & mut ConnectionManager) {
    let mut conn_clone = conn.clone();
    let task = tokio::spawn(async move {
        cmd("PING").query_async::<String>(&mut conn_clone).await
    });
    let res = tokio::time::timeout(Duration::from_secs(1), task).await;

    if !matches!(res, Ok(Ok(Ok(ref pong))) if pong.to_string() == "PONG") {
        println!("Redis connection error");
        exit(0);
    }
}
