use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rdev::{listen, simulate, EventType, Key};
use sqlx::SqlitePool;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::Manager;
use tokio::runtime::Runtime;
use url::Url;
use reqwest::Client;
use arboard::Clipboard;
use regex::Regex;
use image::ImageFormat;
use image::DynamicImage;
use std::io::Cursor;

#[tauri::command]
pub fn simulate_paste() {
    let mut events = vec![
        EventType::KeyPress(Key::MetaLeft),
        EventType::KeyPress(Key::KeyV),
        EventType::KeyRelease(Key::KeyV),
        EventType::KeyRelease(Key::MetaLeft),
    ];

    thread::sleep(Duration::from_millis(100));

    for event in events.drain(..) {
        simulate(&event).unwrap();
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn setup(app_handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();
    let is_processing = std::sync::Arc::new(std::sync::Mutex::new(false));

    std::thread::spawn({
        let is_processing = std::sync::Arc::clone(&is_processing);
        move || {
            listen(move |event| match event.event_type {
                EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => {
                    let _ = tx.send(true);
                }
                EventType::KeyRelease(Key::KeyC) => {
                    let mut is_processing = is_processing.lock().unwrap();
                    if rx.try_recv().is_ok() && !*is_processing {
                        *is_processing = true;
                        let pool = app_handle.state::<SqlitePool>();
                        let rt = app_handle.state::<Runtime>();

                        let mut clipboard = Clipboard::new().unwrap();

                        if let Ok(content) = clipboard.get_text() {
                            rt.block_on(async {
                                insert_content_if_not_exists(&pool, "text", content).await;
                            });
                        }

                        if let Ok(image) = clipboard.get_image() {
                            rt.block_on(async {
                                let png_image = convert_to_png(image.bytes.to_vec());
                                let base64_image = STANDARD.encode(&png_image);
                                insert_content_if_not_exists(&pool, "image", base64_image).await;
                            });
                        }
                        *is_processing = false;
                    }
                }
                EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => {
                    let mut is_processing = is_processing.lock().unwrap();
                    *is_processing = false;
                }
                _ => {}
            })
            .unwrap();
        }
    });
}

fn convert_to_png(image_bytes: Vec<u8>) -> Vec<u8> {
    let img = image::load_from_memory(&image_bytes).unwrap();
    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png).unwrap();
    png_bytes
}

async fn insert_content_if_not_exists(pool: &SqlitePool, content_type: &str, content: String) {
    let last_content: Option<String> = sqlx::query_scalar(
        "SELECT content FROM history WHERE content_type = ? ORDER BY timestamp DESC LIMIT 1",
    )
    .bind(content_type)
    .fetch_one(pool)
    .await
    .unwrap_or(None);

    if last_content.as_deref() != Some(&content) {
        let id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let url_regex = Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)").unwrap();
        let favicon_base64 = if content_type == "text" {
            if let Some(url_match) = url_regex.find(&content) {
                let url_str = url_match.as_str();
                match Url::parse(url_str) {
                    Ok(url) => {
                        match fetch_favicon_as_base64(url).await {
                            Ok(Some(favicon)) => {
                                println!("Favicon fetched successfully.");
                                Some(favicon)
                            },
                            Ok(None) => {
                                println!("No favicon found.");
                                None
                            },
                            Err(e) => {
                                println!("Failed to fetch favicon: {}", e);
                                None
                            }
                        }
                    },
                    Err(e) => {
                        println!("Failed to parse URL: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let _ = sqlx::query("INSERT INTO history (id, content_type, content, favicon) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(content_type)
            .bind(content)
            .bind(favicon_base64)
            .execute(pool)
            .await;
    }
}

async fn fetch_favicon_as_base64(url: Url) -> Result<Option<String>, reqwest::Error> {
    println!("Checking for favicon at URL: {}", url.origin().ascii_serialization());
    let client = Client::new();
    let favicon_url = format!("https://icon.horse/icon/{}", url.host_str().unwrap());
    let response = client.get(&favicon_url).send().await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        Ok(Some(STANDARD.encode(&bytes)))
    } else {
        Ok(None)
    }
}