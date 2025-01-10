use serde::{ Deserialize, Serialize };
use serde_json;
use sqlx::Row;
use sqlx::SqlitePool;
use tauri::{ Emitter, Manager };
use tauri_plugin_aptabase::EventTracker;

#[derive(Deserialize, Serialize)]
struct KeybindSetting {
    keybind: Vec<String>,
}

pub async fn initialize_settings(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let default_keybind = KeybindSetting {
        keybind: vec!["Meta".to_string(), "V".to_string()],
    };
    let json = serde_json::to_string(&default_keybind)?;

    sqlx
        ::query("INSERT INTO settings (key, value) VALUES ('keybind', ?)")
        .bind(json)
        .execute(pool).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_setting(
    pool: tauri::State<'_, SqlitePool>,
    key: String
) -> Result<String, String> {
    let row = sqlx
        ::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(&*pool).await
        .map_err(|e| e.to_string())?;

    Ok(row.map(|r| r.get("value")).unwrap_or_default())
}

#[tauri::command]
pub async fn save_setting(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    key: String,
    value: String
) -> Result<(), String> {
    sqlx
        ::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .bind(key.clone())
        .bind(value.clone())
        .execute(&*pool).await
        .map_err(|e| e.to_string())?;

    let _ = app_handle.track_event(
        "setting_saved",
        Some(serde_json::json!({
        "key": key
    }))
    );

    if key == "keybind" {
        let _ = app_handle.emit("update-shortcut", &value).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_keybind(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let pool = app_handle.state::<SqlitePool>();

    let row = sqlx
        ::query("SELECT value FROM settings WHERE key = 'keybind'")
        .fetch_optional(&*pool).await
        .map_err(|e| e.to_string())?;

    let json = row
        .map(|r| r.get::<String, _>("value"))
        .unwrap_or_else(|| {
            serde_json
                ::to_string(&vec!["MetaLeft".to_string(), "KeyV".to_string()])
                .expect("Failed to serialize default keybind")
        });

    serde_json::from_str::<Vec<String>>(&json).map_err(|e| e.to_string())
}
