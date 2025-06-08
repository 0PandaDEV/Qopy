use tauri_plugin_aptabase::EventTracker;
use base64::{ engine::general_purpose::STANDARD, Engine };
// use hyperpolyglot;
use lazy_static::lazy_static;
use rdev::{ simulate, EventType, Key };
use regex::Regex;
use sqlx::SqlitePool;
use std::fs;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::{ thread, time::Duration };
use tauri::{ AppHandle, Emitter, Listener, Manager };
use tauri_plugin_clipboard::Clipboard;
use tokio::runtime::Runtime as TokioRuntime;
use url::Url;
use uuid::Uuid;

use crate::db;
use crate::utils::commands::get_app_info;
use crate::utils::favicon::fetch_favicon_as_base64;
use crate::utils::types::{ ContentType, HistoryItem };

lazy_static! {
    static ref IS_PROGRAMMATIC_PASTE: AtomicBool = AtomicBool::new(false);
}

#[tauri::command]
pub async fn write_and_paste(
    app_handle: AppHandle,
    content: String,
    content_type: String
) -> Result<(), String> {
    let clipboard = app_handle.state::<Clipboard>();

    match content_type.as_str() {
        "text" => clipboard.write_text(content).map_err(|e| e.to_string())?,
        "link" => clipboard.write_text(content).map_err(|e| e.to_string())?,
        "color" => clipboard.write_text(content).map_err(|e| e.to_string())?,
        "image" => {
            clipboard.write_image_base64(content).map_err(|e| e.to_string())?;
        }
        "files" => {
            clipboard
                .write_files_uris(
                    content
                        .split(", ")
                        .map(|file| file.to_string())
                        .collect::<Vec<String>>()
                )
                .map_err(|e| e.to_string())?;
        }
        _ => {
            return Err("Unsupported content type".to_string());
        }
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
            EventType::KeyRelease(modifier_key)
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

    let _ = app_handle.track_event(
        "clipboard_paste",
        Some(serde_json::json!({
        "content_type": content_type
    }))
    );

    Ok(())
}

pub fn setup(app: &AppHandle) {
    let app_handle = app.clone();
    let runtime = TokioRuntime::new().expect("Failed to create Tokio runtime");

    app_handle.clone().listen("plugin:clipboard://clipboard-monitor/update", move |_event| {
        let app_handle = app_handle.clone();
        runtime.block_on(async move {
            if IS_PROGRAMMATIC_PASTE.load(Ordering::SeqCst) {
                return;
            }

            let clipboard = app_handle.state::<Clipboard>();
            let available_types = clipboard.available_types().unwrap();

            let (app_name, app_icon) = get_app_info();

            match get_pool(&app_handle).await {
                Ok(pool) => {
                    if available_types.image {
                        println!("Handling image change");
                        if let Ok(image_data) = clipboard.read_image_base64() {
                            let file_path = save_image_to_file(&app_handle, &image_data).await
                                .map_err(|e| e.to_string())
                                .unwrap_or_else(|e| e);
                            let _ = db::history::add_history_item(
                                app_handle.clone(),
                                pool,
                                HistoryItem::new(
                                    app_name,
                                    ContentType::Image,
                                    file_path,
                                    None,
                                    app_icon,
                                    None
                                )
                            ).await;
                        }
                    } else if available_types.files {
                        println!("Handling files change");
                        if let Ok(files) = clipboard.read_files() {
                            for file in files {
                                let _ = db::history::add_history_item(
                                    app_handle.clone(),
                                    pool.clone(),
                                    HistoryItem::new(
                                        app_name.clone(),
                                        ContentType::File,
                                        file,
                                        None,
                                        app_icon.clone(),
                                        None
                                    )
                                ).await;
                            }
                        }
                    } else if available_types.text {
                        println!("Handling text change");
                        if let Ok(text) = clipboard.read_text() {
                            let text = text.to_string();
                            let url_regex = Regex::new(
                                r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"
                            ).unwrap();

                            if url_regex.is_match(&text) {
                                if let Ok(url) = Url::parse(&text) {
                                    let favicon = match fetch_favicon_as_base64(url).await {
                                        Ok(Some(f)) => Some(f),
                                        _ => None,
                                    };

                                    let _ = db::history::add_history_item(
                                        app_handle.clone(),
                                        pool,
                                        HistoryItem::new(
                                            app_name,
                                            ContentType::Link,
                                            text,
                                            favicon,
                                            app_icon,
                                            None
                                        )
                                    ).await;
                                }
                            } else {
                                if text.is_empty() {
                                    return;
                                }

                                // Temporarily disabled code detection
                                /*if let Some(detection) = hyperpolyglot::detect_from_text(&text) {
                                        let language = match detection {
                                            hyperpolyglot::Detection::Heuristics(lang) => lang.to_string(),
                                            _ => detection.language().to_string(),
                                        };

                                        let _ = db::history::add_history_item(
                                            pool,
                                            HistoryItem::new(app_name, ContentType::Code, text, None, app_icon, Some(language))
                                        ).await;
                                    } else*/ if crate::utils::commands::detect_color(&text) {
                                    let _ = db::history::add_history_item(
                                        app_handle.clone(),
                                        pool,
                                        HistoryItem::new(
                                            app_name,
                                            ContentType::Color,
                                            text,
                                            None,
                                            app_icon,
                                            None
                                        )
                                    ).await;
                                } else {
                                    let _ = db::history::add_history_item(
                                        app_handle.clone(),
                                        pool,
                                        HistoryItem::new(
                                            app_name,
                                            ContentType::Text,
                                            text.clone(),
                                            None,
                                            app_icon,
                                            None
                                        )
                                    ).await;
                                }
                            }
                        }
                    } else {
                        println!("Unknown clipboard content type");
                    }
                }
                Err(e) => {
                    println!("Failed to get database pool: {}", e);
                }
            }

            let _ = app_handle.track_event(
                "clipboard_copied",
                Some(
                    serde_json::json!({
                    "content_type": if available_types.image { "image" }
                        else if available_types.files { "files" }
                        else if available_types.text { "text" }
                        else { "unknown" }
                })
                )
            );
        });
    });
}

async fn get_pool(
    app_handle: &AppHandle
) -> Result<tauri::State<'_, SqlitePool>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(app_handle.state::<SqlitePool>())
}

#[tauri::command]
pub fn start_monitor(app_handle: AppHandle) -> Result<(), String> {
    let clipboard = app_handle.state::<Clipboard>();
    clipboard.start_monitor(app_handle.clone()).map_err(|e| e.to_string())?;
    app_handle
        .emit("plugin:clipboard://clipboard-monitor/status", true)
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn save_image_to_file(
    app_handle: &AppHandle,
    base64_data: &str
) -> Result<String, Box<dyn std::error::Error>> {
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let images_dir = app_data_dir.join("images");
    fs::create_dir_all(&images_dir)?;

    let file_name = format!("{}.png", Uuid::new_v4());
    let file_path = images_dir.join(&file_name);

    let bytes = STANDARD.decode(base64_data)?;
    fs::write(&file_path, bytes)?;

    Ok(file_path.to_string_lossy().into_owned())
}
