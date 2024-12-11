use sqlx::{Row, SqlitePool};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use crate::utils::types::{HistoryItem, ContentType};
use std::fs;
use base64::{Engine, engine::general_purpose::STANDARD};

pub async fn initialize_history(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    sqlx::query(
        "INSERT INTO history (id, content_type, content, timestamp) VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(id)
    .bind("text")
    .bind("Welcome to your clipboard history!")
    .execute(pool)
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn get_history(pool: tauri::State<'_, SqlitePool>) -> Result<Vec<HistoryItem>, String> {
    let rows = sqlx::query(
        "SELECT id, content_type, content, favicon, timestamp FROM history ORDER BY timestamp DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows.iter().map(|row| HistoryItem {
        id: row.get("id"),
        content_type: ContentType::from(row.get::<String, _>("content_type")),
        content: row.get("content"),
        favicon: row.get("favicon"),
        timestamp: row.get("timestamp"),
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn add_history_item(
    pool: tauri::State<'_, SqlitePool>,
    item: HistoryItem,
) -> Result<(), String> {
    let (id, content_type, content, favicon, timestamp) = item.to_row();
    
    sqlx::query(
        "INSERT INTO history (id, content_type, content, favicon, timestamp) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(content_type)
    .bind(content)
    .bind(favicon)
    .bind(timestamp)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn search_history(
    pool: tauri::State<'_, SqlitePool>,
    query: String
) -> Result<Vec<HistoryItem>, String> {
    let query = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, content_type, content, favicon, timestamp FROM history WHERE content LIKE ? ORDER BY timestamp DESC"
    )
    .bind(query)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows.iter().map(|row| HistoryItem {
        id: row.get("id"),
        content_type: ContentType::from(row.get::<String, _>("content_type")),
        content: row.get("content"),
        favicon: row.get("favicon"),
        timestamp: row.get("timestamp"),
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn load_history_chunk(
    pool: tauri::State<'_, SqlitePool>,
    offset: i64,
    limit: i64
) -> Result<Vec<HistoryItem>, String> {
    let rows = sqlx::query(
        "SELECT id, content_type, content, favicon, timestamp FROM history ORDER BY timestamp DESC LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows.iter().map(|row| HistoryItem {
        id: row.get("id"),
        content_type: ContentType::from(row.get::<String, _>("content_type")),
        content: row.get("content"),
        favicon: row.get("favicon"),
        timestamp: row.get("timestamp"),
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn delete_history_item(
    pool: tauri::State<'_, SqlitePool>,
    id: String
) -> Result<(), String> {
    sqlx::query("DELETE FROM history WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn clear_history(pool: tauri::State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("DELETE FROM history")
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn read_image(filename: String) -> Result<String, String> {
    let bytes = fs::read(filename).map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(bytes))
}