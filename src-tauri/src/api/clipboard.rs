use lazy_static::lazy_static;
use rdev::{simulate, EventType, Key};
use sqlx::SqlitePool;
use uuid::Uuid;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter, Listener, Manager, Runtime};
use tauri_plugin_clipboard::Clipboard;
use tokio::runtime::Runtime as TokioRuntime;
use regex::Regex;
use url::Url;
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::utils::favicon::fetch_favicon_as_base64;
use crate::db;
use crate::utils::types::{ContentType, HistoryItem};

lazy_static! {
    static ref IS_PROGRAMMATIC_PASTE: AtomicBool = AtomicBool::new(false);
}

#[tauri::command]
pub async fn write_and_paste<R: Runtime>(
    app_handle: tauri::AppHandle<R>,
    content: String,
    content_type: String,
) -> Result<(), String> {
    let clipboard = app_handle.state::<Clipboard>();

    match content_type.as_str() {
        "text" => clipboard.write_text(content).map_err(|e| e.to_string())?,
        "image" => {
            clipboard
                .write_image_base64(content)
                .map_err(|e| e.to_string())?;
        }
        "files" => {
            clipboard
                .write_files_uris(
                    content
                        .split(", ")
                        .map(|file| file.to_string())
                        .collect::<Vec<String>>(),
                )
                .map_err(|e| e.to_string())?;
        }
        _ => return Err("Unsupported content type".to_string()),
    }

    IS_PROGRAMMATIC_PASTE.store(true, Ordering::SeqCst);

    thread::spawn(|| {
        thread::sleep(Duration::from_millis(100));

        #[cfg(target_os = "macos")]
        let modifier_key = Key::MetaLeft;
        #[cfg(not(target_os = "macos"))]
        let modifier_key = Key::ControlLeft;

        let events = vec![
            EventType::KeyPress(modifier_key),
            EventType::KeyPress(Key::KeyV),
            EventType::KeyRelease(Key::KeyV),
            EventType::KeyRelease(modifier_key),
        ];

        for event in events {
            if let Err(e) = simulate(&event) {
                println!("Simulation error: {:?}", e);
            }
            thread::sleep(Duration::from_millis(20));
        }
    });

    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        IS_PROGRAMMATIC_PASTE.store(false, Ordering::SeqCst);
    });

    Ok(())
}

pub fn setup<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    let runtime = TokioRuntime::new().expect("Failed to create Tokio runtime");

    app.clone().listen(
        "plugin:clipboard://clipboard-monitor/update",
        move |_event| {
            let app = app.clone();
            runtime.block_on(async move {
                if IS_PROGRAMMATIC_PASTE.load(Ordering::SeqCst) {
                    return;
                }

                let clipboard = app.state::<Clipboard>();
                let available_types = clipboard.available_types().unwrap();

                match get_pool(&app).await {
                    Ok(pool) => {
                        if available_types.image {
                            println!("Handling image change");
                            if let Ok(image_data) = clipboard.read_image_base64() {
                                let file_path = save_image_to_file(&app, &image_data)
                                    .await
                                    .map_err(|e| e.to_string())
                                    .unwrap_or_else(|e| e);
                                let _ = db::history::add_history_item(
                                    pool,
                                    HistoryItem::new(ContentType::Image, file_path, None),
                                ).await;
                            }
                            let _ = app.emit("plugin:clipboard://image-changed", ());
                        } else if available_types.files {
                            println!("Handling files change");
                            if let Ok(files) = clipboard.read_files() {
                                let files_str = files.join(", ");
                                let _ = db::history::add_history_item(
                                    pool,
                                    HistoryItem::new(ContentType::File, files_str, None),
                                ).await;
                            }
                            let _ = app.emit("plugin:clipboard://files-changed", ());
                        } else if available_types.text {
                            println!("Handling text change");
                            if let Ok(text) = clipboard.read_text() {
                                let url_regex = Regex::new(r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)$").unwrap();
                                
                                if url_regex.is_match(&text) {
                                    if let Ok(url) = Url::parse(&text) {
                                        let favicon = match fetch_favicon_as_base64(url).await {
                                            Ok(Some(f)) => Some(f),
                                            _ => None,
                                        };
                                        
                                        let _ = db::history::add_history_item(
                                            pool,
                                            HistoryItem::new(ContentType::Link, text, favicon)
                                        ).await;
                                    }
                                } else {
                                    if text.is_empty() {
                                        return;
                                    }
                                    let _ = db::history::add_history_item(
                                        pool,
                                        HistoryItem::new(ContentType::Text, text, None)
                                    ).await;
                                }
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
        },
    );
}

async fn get_pool<R: Runtime>(
    app_handle: &AppHandle<R>,
) -> Result<tauri::State<'_, SqlitePool>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(app_handle.state::<SqlitePool>())
}

#[tauri::command]
pub fn start_monitor(app_handle: AppHandle) -> Result<(), String> {
    let clipboard = app_handle.state::<Clipboard>();
    clipboard
        .start_monitor(app_handle.clone())
        .map_err(|e| e.to_string())?;
    app_handle
        .emit("plugin:clipboard://clipboard-monitor/status", true)
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn save_image_to_file<R: Runtime>(app_handle: &AppHandle<R>, base64_data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let images_dir = app_data_dir.join("images");
    fs::create_dir_all(&images_dir)?;
    
    let file_name = format!("{}.png", Uuid::new_v4());
    let file_path = images_dir.join(&file_name);
    
    let bytes = STANDARD.decode(base64_data)?;
    fs::write(&file_path, bytes)?;
    
    Ok(file_path.to_string_lossy().into_owned())
}
