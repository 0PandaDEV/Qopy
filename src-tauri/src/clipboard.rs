use arboard::Clipboard;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::io::Reader as ImageReader;
use image::ImageFormat;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rdev::{simulate, EventType, Key};
use regex::Regex;
use reqwest::Client;
use sqlx::SqlitePool;
use std::io::Cursor;
use std::thread;
use std::time::Duration;
use tauri::Manager;
use tokio::runtime::Runtime;
use url::Url;

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
    let is_processing = std::sync::Arc::new(std::sync::Mutex::new(false));

    std::thread::spawn({
        let is_processing = std::sync::Arc::clone(&is_processing);
        move || {
            let mut clipboard = Clipboard::new().unwrap();
            let mut last_text = String::new();
            let mut last_image = Vec::new();

            loop {
                let mut is_processing = is_processing.lock().unwrap();
                if !*is_processing {
                    *is_processing = true;
                    let pool = app_handle.state::<SqlitePool>();
                    let rt = app_handle.state::<Runtime>();

                    if let Ok(content) = clipboard.get_text() {
                        if content != last_text {
                            last_text = content.clone();
                            rt.block_on(async {
                                insert_content_if_not_exists(&pool, "text", content).await;
                            });
                        }
                    }

                    if let Ok(image) = clipboard.get_image() {
                        let image_bytes = image.bytes.to_vec();
                        if image_bytes != last_image {
                            last_image = image_bytes.clone();
                            rt.block_on(async {
                                match convert_to_png(image_bytes) {
                                    Ok(png_image) => {
                                        let base64_image = STANDARD.encode(&png_image);
                                        insert_content_if_not_exists(&pool, "image", base64_image).await;
                                    }
                                    Err(e) => {
                                        println!("Failed to convert image to PNG: {}", e);
                                    }
                                }
                            });
                        }
                    }
                    *is_processing = false;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    });
}

fn convert_to_png(image_bytes: Vec<u8>) -> Result<Vec<u8>, image::ImageError> {
    match image::guess_format(&image_bytes) {
        Ok(format) => println!("Image format: {:?}", format),
        Err(e) => println!("Failed to guess image format: {}", e),
    }
    let img = image::load_from_memory(&image_bytes)?;
    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)?;
    Ok(png_bytes)
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

        let url_regex = Regex::new(r"^https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$").unwrap();
        let favicon_base64 = if content_type == "text" && url_regex.is_match(&content) {
            match Url::parse(&content) {
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
        .bind(content)
        .bind(favicon_base64)
        .execute(pool)
        .await;
    }
}

async fn fetch_favicon_as_base64(url: Url) -> Result<Option<String>, Box<dyn std::error::Error>> {
    println!(
        "Checking for favicon at URL: {}",
        url.origin().ascii_serialization()
    );
    let client = Client::new();
    let favicon_url = format!("https://icon.horse/icon/{}", url.host_str().unwrap());
    let response = client.get(&favicon_url).send().await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()?
            .decode()?;
        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)?;
        Ok(Some(STANDARD.encode(&png_bytes)))
    } else {
        Ok(None)
    }
}