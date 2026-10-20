use std::process::exit;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::Duration;

use redis::{aio::ConnectionManager, cmd};
use sqlx::PgPool;
use tokio::time::timeout;

const CHECK_TIMEOUT: Duration = Duration::from_secs(2);

fn fail(msg: &str) -> ! {
    tracing::error!("[FAIL] {msg}");
    eprintln!("[FAIL] {msg}");
    exit(1);
}

pub async fn is_db_alive(pool: &PgPool) {
    if pool.is_closed() {
        fail("Postgres: connect poll close");
    }
    match timeout(CHECK_TIMEOUT, sqlx::query("SELECT 1").execute(pool)).await {
        Err(_) => fail(&format!("Postgres: without answer {:?}", CHECK_TIMEOUT)),
        Ok(Err(e)) => fail(&format!("Postgres: call err: {e}")),
        Ok(Ok(_)) => tracing::info!("[SUCCESS] Postgres: work"),
    }
}

pub async fn is_redis_alive(conn: &mut ConnectionManager) {
    match timeout(CHECK_TIMEOUT, cmd("PING").query_async::<String>(conn)).await {
        Err(_) => fail(&format!("Redis: PING not work {:?}", CHECK_TIMEOUT)),
        Ok(Err(e)) => fail(&format!("Redis: PING end with err: {e}")),
        Ok(Ok(pong)) if pong == "PONG" => tracing::info!("[SUCCESS] Redis: work"),
        Ok(Ok(other)) => fail(&format!("Redis: answer on PING: {other}")),
    }
}

pub fn start_watchdog(what: &'static str, secs: u64) -> Arc<AtomicBool> {
    let done = Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(secs));
        if !flag.load(Ordering::SeqCst) {
            eprintln!("[FAIL] {what}: no connection to {secs}sec, exit");
            exit(1);
        }
    });
    done
}