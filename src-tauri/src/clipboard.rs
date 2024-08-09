use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tauri::{AppHandle, Manager, Runtime, Emitter, Listener};
use tauri_plugin_clipboard::Clipboard;
use tokio::runtime::Runtime as TokioRuntime;
use regex::Regex;
use sqlx::SqlitePool;
use std::{
    fs,
    sync::Mutex,
    thread,
    time::Duration,
};
use rand::Rng;
use sha2::{Sha256, Digest};
use rdev::{simulate, Key, EventType};
use lazy_static::lazy_static;
use image::ImageFormat;

lazy_static! {
    static ref APP_DATA_DIR: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);
}

pub fn set_app_data_dir(path: std::path::PathBuf) {
    let mut dir = APP_DATA_DIR.lock().unwrap();
    *dir = Some(path);
}

#[tauri::command]
pub fn read_image(filename: String) -> Result<Vec<u8>, String> {
    let app_data_dir = APP_DATA_DIR.lock().unwrap();
    let app_data_dir = app_data_dir.as_ref().expect("App data directory not set");
    let image_path = app_data_dir.join("images").join(filename);
    fs::read(image_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn simulate_paste() {
    let mut events = vec![
        EventType::KeyPress(Key::ControlLeft),
        EventType::KeyPress(Key::KeyV),
        EventType::KeyRelease(Key::KeyV),
        EventType::KeyRelease(Key::ControlLeft),
    ];

    thread::sleep(Duration::from_millis(100));

    for event in events.drain(..) {
        simulate(&event).unwrap();
        thread::sleep(Duration::from_millis(20));
    }
}

#[tauri::command]
pub fn get_image_path(app_handle: tauri::AppHandle, filename: String) -> String {
    let app_data_dir = app_handle.path().app_data_dir().expect("Failed to get app data directory");
    let image_path = app_data_dir.join("images").join(filename);
    image_path.to_str().unwrap_or("").to_string()
}

pub fn setup<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    let runtime = TokioRuntime::new().expect("Failed to create Tokio runtime");

    app.clone().listen("plugin:clipboard://clipboard-monitor/update", move |_event| {
        let app = app.clone();
        runtime.block_on(async move {
            let clipboard = app.state::<Clipboard>();
            let available_types = clipboard.available_types().unwrap();
            
            println!("Clipboard update detected");

            match get_pool(&app).await {
                Ok(pool) => {
                    if available_types.image {
                        println!("Handling image change");
                        if let Ok(image_data) = clipboard.read_image_base64() {
                            let base64_image = STANDARD.encode(&image_data);
                            insert_content_if_not_exists(app.clone(), pool.clone(), "image", base64_image).await;
                        }
                        let _ = app.emit("plugin:clipboard://image-changed", ());
                    } else if available_types.rtf {
                        println!("Handling RTF change");
                        if let Ok(rtf) = clipboard.read_rtf() {
                            insert_content_if_not_exists(app.clone(), pool.clone(), "rtf", rtf).await;
                        }
                        let _ = app.emit("plugin:clipboard://rtf-changed", ());
                    } else if available_types.files {
                        println!("Handling files change");
                        if let Ok(files) = clipboard.read_files() {
                            let files_str = files.join(", ");
                            insert_content_if_not_exists(app.clone(), pool.clone(), "files", files_str).await;
                        }
                        let _ = app.emit("plugin:clipboard://files-changed", ());
                    } else if available_types.text {
                        println!("Handling text change");
                        if let Ok(text) = clipboard.read_text() {
                            insert_content_if_not_exists(app.clone(), pool.clone(), "text", text).await;
                        }
                        let _ = app.emit("plugin:clipboard://text-changed", ());
                    } else {
                        println!("Unknown clipboard content type");
                    }
                }
                Err(e) => {
                    println!("Failed to get database pool: {}", e);
                }
            }
        });
    });
}

async fn get_pool<R: Runtime>(app_handle: &AppHandle<R>) -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>> {
    let app_data_dir = app_handle.path().app_data_dir().expect("Failed to get app data directory");
    let db_path = app_data_dir.join("data.db");
    let database_url = format!("sqlite:{}", db_path.to_str().unwrap());
    SqlitePool::connect(&database_url).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
}

async fn insert_content_if_not_exists<R: Runtime>(app_handle: AppHandle<R>, pool: SqlitePool, content_type: &str, content: String) {
    let last_content: Option<String> = sqlx::query_scalar(
        "SELECT content FROM history WHERE content_type = ? ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(content_type)
    .fetch_one(&pool)
    .await
    .unwrap_or(None);

    let content = if content_type == "image" {
        match save_image(&app_handle, &content).await {
            Ok(path) => path,
            Err(e) => {
                println!("Failed to save image: {}", e);
                content
            }
        }
    } else {
        content
    };

    if last_content.as_deref() != Some(&content) {
        let id: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let url_regex = Regex::new(r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)$").unwrap();
        let favicon_base64 = if content_type == "text" && url_regex.is_match(&content) {
            match url::Url::parse(&content) {
                Ok(url) => match fetch_favicon_as_base64(url).await {
                    Ok(Some(favicon)) => Some(favicon),
                    Ok(None) => None,
                    Err(e) => {
                        println!("Failed to fetch favicon: {}", e);
                        None
                    }
                },
                Err(e) => {
                    println!("Failed to parse URL: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let _ = sqlx::query(
            "INSERT INTO history (id, content_type, content, favicon) VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(content_type)
        .bind(&content)
        .bind(favicon_base64)
        .execute(&pool)
        .await;
    }
}

async fn save_image<R: Runtime>(app_handle: &AppHandle<R>, base64_image: &str) -> Result<String, Box<dyn std::error::Error>> {
    let image_data = STANDARD.decode(base64_image)?;
    let mut hasher = Sha256::new();
    hasher.update(&image_data);
    let hash = hasher.finalize();
    let filename = format!("{:x}.png", hash);
    
    let app_data_dir = app_handle.path().app_data_dir().expect("Failed to get app data directory");
    let images_dir = app_data_dir.join("images");
    let path = images_dir.join(&filename);
    
    if !path.exists() {
        fs::create_dir_all(&images_dir)?;
        fs::write(&path, &image_data)?;
    }
    
    Ok(path.to_str().unwrap().to_string())
}

async fn fetch_favicon_as_base64(url: url::Url) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let favicon_url = format!("https://favicone.com/{}", url.host_str().unwrap());
    let response = client.get(&favicon_url).send().await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let img = image::load_from_memory(&bytes)?;
        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)?;
        Ok(Some(STANDARD.encode(&png_bytes)))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn start_monitor(app_handle: AppHandle) -> Result<(), String> {
    let clipboard = app_handle.state::<Clipboard>();
    clipboard.start_monitor(app_handle.clone()).map_err(|e| e.to_string())?;
    app_handle.emit("plugin:clipboard://clipboard-monitor/status", true).map_err(|e| e.to_string())?;
    Ok(())
}