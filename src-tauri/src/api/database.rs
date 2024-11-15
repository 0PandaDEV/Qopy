use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::fs;
use tauri::{Manager, Emitter};
use tokio::runtime::Runtime;

#[derive(Deserialize, Serialize)]
struct KeybindSetting {
    keybind: Vec<String>,
}

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
        // Setup settings table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create settings table");

        let existing_keybind = sqlx::query_scalar::<_, Option<String>>(
            "SELECT value FROM settings WHERE key = 'keybind'"
        )
        .fetch_one(&pool)
        .await;

        match existing_keybind {
            Ok(Some(_)) => {
            },
            Ok(None) => {
                let default_keybind = KeybindSetting {
                    keybind: vec!["Meta".to_string(), "V".to_string()],
                };
                let json = serde_json::to_string(&default_keybind).unwrap();

                sqlx::query(
                    "INSERT INTO settings (key, value) VALUES ('keybind', ?)"
                )
                .bind(json)
                .execute(&pool)
                .await
                .expect("Failed to insert default keybind");
            },
            Err(e) => {
                eprintln!("Failed to check existing keybind: {}", e);
            }
        }

        // Setup history table
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
        .expect("Failed to create history table");

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

#[tauri::command]
pub async fn save_keybind(
    app_handle: tauri::AppHandle,
    keybind: Vec<String>,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<(), String> {
    let json = serde_json::to_string(&keybind).map_err(|e| e.to_string())?;

    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('keybind', ?)")
        .bind(json)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let keybind_str = keybind.join("+");
    app_handle
        .emit("update-shortcut", keybind_str)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_keybind(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let pool = app_handle.state::<SqlitePool>();

    let result =
        sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = 'keybind'")
            .fetch_optional(&*pool)
            .await
            .map_err(|e| e.to_string())?;

    match result {
        Some(json) => {
            let keybind: Vec<String> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
            Ok(keybind)
        }
        None => {
            let default_keybind = vec!["Meta".to_string(), "V".to_string()];
            Ok(default_keybind)
        }
    }
}
