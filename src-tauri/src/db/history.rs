use crate::utils::types::{ ContentType, HistoryItem };
use base64::{ engine::general_purpose::STANDARD, Engine };
use rand::{ rng, Rng };
use rand::distr::Alphanumeric;
use sqlx::{ Row, SqlitePool };
use std::fs;
use tauri_plugin_aptabase::EventTracker;
use tauri::Emitter;

pub async fn initialize_history(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let id: String = rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    sqlx::query(
        "INSERT INTO history (id, source, content_type, content, timestamp) VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(id)
    .bind("System")
    .bind("text")
    .bind("Welcome to your clipboard history!")
    .execute(pool).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_history(pool: tauri::State<'_, SqlitePool>) -> Result<Vec<HistoryItem>, String> {
    let rows = sqlx
        ::query(
            "SELECT id, source, source_icon, content_type, content, favicon, timestamp, language FROM history ORDER BY timestamp DESC"
        )
        .fetch_all(&*pool).await
        .map_err(|e| e.to_string())?;

    let items = rows
        .iter()
        .map(|row| HistoryItem {
            id: row.get("id"),
            source: row.get("source"),
            source_icon: row.get("source_icon"),
            content_type: ContentType::from(row.get::<String, _>("content_type")),
            content: row.get("content"),
            favicon: row.get("favicon"),
            timestamp: row.get("timestamp"),
            language: row.get("language"),
        })
        .collect();

    Ok(items)
}

#[tauri::command]
pub async fn add_history_item(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    item: HistoryItem
) -> Result<(), String> {
    let (id, source, source_icon, content_type, content, favicon, timestamp, language) =
        item.to_row();

    let existing = sqlx
        ::query("SELECT id FROM history WHERE content = ? AND content_type = ?")
        .bind(&content)
        .bind(&content_type)
        .fetch_optional(&*pool).await
        .map_err(|e| e.to_string())?;

    match existing {
        Some(_) => {
            sqlx
                ::query(
                    "UPDATE history SET source = ?, source_icon = ?, timestamp = strftime('%Y-%m-%dT%H:%M:%f+00:00', 'now'), favicon = ?, language = ? WHERE content = ? AND content_type = ?"
                )
                .bind(&source)
                .bind(&source_icon)
                .bind(&favicon)
                .bind(&language)
                .bind(&content)
                .bind(&content_type)
                .execute(&*pool).await
                .map_err(|e| e.to_string())?;
        }
        None => {
            sqlx
                ::query(
                    "INSERT INTO history (id, source, source_icon, content_type, content, favicon, timestamp, language) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(id)
                .bind(source)
                .bind(source_icon)
                .bind(content_type)
                .bind(content)
                .bind(favicon)
                .bind(timestamp)
                .bind(language)
                .execute(&*pool).await
                .map_err(|e| e.to_string())?;
        }
    }

    let _ = app_handle.track_event(
        "history_item_added",
        Some(serde_json::json!({
        "content_type": item.content_type.to_string()
    }))
    );
    
    let _ = app_handle.emit("clipboard-content-updated", ());

    Ok(())
}

#[tauri::command]
pub async fn search_history(
    pool: tauri::State<'_, SqlitePool>,
    query: String
) -> Result<Vec<HistoryItem>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let query = format!("%{}%", query);
    
    let rows = sqlx
        ::query(
            "SELECT id, source, source_icon, content_type, content, favicon, timestamp, language 
             FROM history 
             WHERE content LIKE ? 
             ORDER BY timestamp DESC
             LIMIT 100"
        )
        .bind(query)
        .fetch_all(&*pool).await
        .map_err(|e| e.to_string())?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows.iter() {
        items.push(HistoryItem {
            id: row.get("id"),
            source: row.get("source"),
            source_icon: row.get("source_icon"),
            content_type: ContentType::from(row.get::<String, _>("content_type")),
            content: row.get("content"),
            favicon: row.get("favicon"),
            timestamp: row.get("timestamp"),
            language: row.get("language"),
        });
    }

    Ok(items)
}

#[tauri::command]
pub async fn load_history_chunk(
    pool: tauri::State<'_, SqlitePool>,
    offset: i64,
    limit: i64
) -> Result<Vec<HistoryItem>, String> {
    let rows = sqlx
        ::query(
            "SELECT id, source, source_icon, content_type, content, favicon, timestamp, language FROM history ORDER BY timestamp DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*pool).await
        .map_err(|e| e.to_string())?;

    let items = rows
        .iter()
        .map(|row| HistoryItem {
            id: row.get("id"),
            source: row.get("source"),
            source_icon: row.get("source_icon"),
            content_type: ContentType::from(row.get::<String, _>("content_type")),
            content: row.get("content"),
            favicon: row.get("favicon"),
            timestamp: row.get("timestamp"),
            language: row.get("language"),
        })
        .collect();

    Ok(items)
}

#[tauri::command]
pub async fn delete_history_item(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>,
    id: String
) -> Result<(), String> {
    sqlx
        ::query("DELETE FROM history WHERE id = ?")
        .bind(id)
        .execute(&*pool).await
        .map_err(|e| e.to_string())?;

    let _ = app_handle.track_event("history_item_deleted", None);
    let _ = app_handle.emit("clipboard-content-updated", ());

    Ok(())
}

#[tauri::command]
pub async fn clear_history(
    app_handle: tauri::AppHandle,
    pool: tauri::State<'_, SqlitePool>
) -> Result<(), String> {
    sqlx
        ::query("DELETE FROM history")
        .execute(&*pool).await
        .map_err(|e| e.to_string())?;

    let _ = app_handle.track_event("history_cleared", None);
    let _ = app_handle.emit("clipboard-content-updated", ());

    Ok(())
}

#[tauri::command]
pub async fn read_image(filename: String) -> Result<String, String> {
    let bytes = fs::read(filename).map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(bytes))
}
