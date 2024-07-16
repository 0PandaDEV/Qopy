use sqlx::sqlite::SqlitePoolOptions;
use std::fs;
use tokio::runtime::Runtime;
use tauri::Manager;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    let app_data_dir = app.path().app_data_dir().unwrap();
    fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    let db_path = app_data_dir.join("data.db");
    let is_new_db = !db_path.exists();
    if is_new_db {
        fs::File::create(&db_path).expect("Failed to create database file");
    }

    let db_url = format!("sqlite:{}", db_path.to_str().unwrap());
    let pool = rt.block_on(async {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(async {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                content_type TEXT NOT NULL,
                content TEXT NOT NULL,
                favicon TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create table");

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON history (timestamp)"
        )
        .execute(&pool)
        .await
        .expect("Failed to create index");

        if is_new_db {
            let id: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            sqlx::query("INSERT INTO history (id, content_type, content, timestamp) VALUES (?, ?, ?, CURRENT_TIMESTAMP)")
                .bind(id)
                .bind("text")
                .bind("Welcome to your clipboard history!")
                .execute(&pool)
                .await
                .expect("Failed to insert welcome message");
        }
    });

    app.manage(pool);
    app.manage(rt);

    Ok(())
}