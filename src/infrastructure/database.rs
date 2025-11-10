use anyhow::Result;
use once_cell::sync::OnceCell;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

pub static DB_POOL: OnceCell<MySqlPool> = OnceCell::new();

pub async fn init_db_pool(database_url: &str) -> Result<()> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    DB_POOL
        .set(pool)
        .expect("❌ DB_POOL is already initialized");
    Ok(())
}

pub fn db_pool() -> &'static MySqlPool {
    DB_POOL.get().expect("❌ DB_POOL is not initialized")
}
