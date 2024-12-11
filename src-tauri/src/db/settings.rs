use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use serde_json;
use tauri::{Emitter, Manager};
use sqlx::Row;

#[derive(Deserialize, Serialize)]
struct KeybindSetting {
    keybind: Vec<String>,
}

pub async fn initialize_settings(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let default_keybind = KeybindSetting {
        keybind: vec!["Meta".to_string(), "V".to_string()],
    };
    let json = serde_json::to_string(&default_keybind)?;

    sqlx::query(
        "INSERT INTO settings (key, value) VALUES ('keybind', ?)"
    )
    .bind(json)
    .execute(pool)
    .await?;

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
pub async fn get_setting(
    pool: tauri::State<'_, SqlitePool>,
    key: String
) -> Result<String, String> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(row.map(|r| r.get("value")).unwrap_or_default())
}

#[tauri::command]
pub async fn save_setting(
    pool: tauri::State<'_, SqlitePool>,
    key: String,
    value: String
) -> Result<(), String> {
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_keybind(
    app_handle: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let pool = app_handle.state::<SqlitePool>();
    
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'keybind'")
        .fetch_optional(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let json = row
        .map(|r| r.get::<String, _>("value"))
        .unwrap_or_else(|| {
            serde_json::to_string(&vec!["Meta".to_string(), "V".to_string()])
                .expect("Failed to serialize default keybind")
        });

    serde_json::from_str::<Vec<String>>(&json)
        .map_err(|e| e.to_string())
}