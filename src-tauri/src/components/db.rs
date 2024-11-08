use crate::components::monitor::LiveUser;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Error, Pool, Sqlite};
use std::str::FromStr;
use std::sync::atomic::AtomicI32;
pub struct AppState {
    pub pool: Pool<Sqlite>,
    pub current_id: AtomicI32,
    pub max_id: AtomicI32,
}

pub async fn get_instance() -> Result<Pool<Sqlite>, Error> {
    let database_url = "sqlite://user.db"; // 数据库文件路径
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // 创建表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            hook TEXT NOT NULL,
            status INTEGER DEFAULT 1,
            created_at DATETIME DEFAULT (datetime('now', '+8 hours')),
            updated_at DATETIME DEFAULT '1970-01-01 00:00:00'
        );
        "#,
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

pub async fn get_last_user(pool: &Pool<Sqlite>) -> Option<LiveUser> {
    let query_str =
        "SELECT id, name, url, hook, status, created_at, updated_at FROM users ORDER BY id DESC";
    match sqlx::query_as::<_, LiveUser>(&query_str)
        .fetch_one(pool)
        .await
    {
        Ok(row) => Some(row),
        Err(_e) => None,
    }
}

pub async fn get_user_by_id(id: i32, pool: &Pool<Sqlite>) -> Option<LiveUser> {
    let query_str = format!(
        "SELECT id, name, url, hook, status, created_at, updated_at FROM users WHERE id={}",
        id
    );
    match sqlx::query_as::<_, LiveUser>(&query_str)
        .fetch_one(pool)
        .await
    {
        Ok(row) => Some(row),
        Err(_e) => None,
    }
}

pub async fn set_user_state(id: i32, status: bool, pool: &Pool<Sqlite>) -> bool {
    let query_str = format!(
        "UPDATE users SET status={}, updated_at = (datetime('now', '+8 hours')) WHERE id = {}",
        status, id
    );
    match sqlx::query(&query_str).execute(pool).await {
        Ok(_res) => true,
        Err(e) => {
            eprintln!("set user state err: {}", e);
            false
        }
    }
}
