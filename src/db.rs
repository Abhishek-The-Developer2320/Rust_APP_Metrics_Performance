use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::config::database_url;

/// Initialize SQLite database pool and create table if it doesn't exist
pub async fn init_db() -> SqlitePool {
    let db_url = database_url();

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    // Create `task` table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS task (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0
        );
        "#
    )
    .execute(&pool)
    .await
    .expect("Failed to create task table");

    pool
}
