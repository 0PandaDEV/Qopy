use rdev::{listen, simulate, EventType, Key};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::Manager;
use arboard::{Clipboard, ImageData};
use sqlx::SqlitePool;
use tokio::runtime::Runtime;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

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

    std::thread::spawn(move || {
        listen(move |event| match event.event_type {
            EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => {
                let _ = tx.send(true);
            }
            EventType::KeyRelease(Key::KeyC) => {
                if rx.try_recv().is_ok() {
                    let mut clipboard = Clipboard::new().unwrap();
                    let pool = app_handle.state::<SqlitePool>();
                    let rt = app_handle.state::<Runtime>();

                    if let Ok(content) = clipboard.get_text() {
                        rt.block_on(async {
                            insert_content_if_not_exists(&pool, "text", content).await;
                        });
                    }

                    match clipboard.get_image() {
                        Ok(image) => {
                            println!("Image found in clipboard");
                            rt.block_on(async {
                                let base64_image = STANDARD.encode(&image.bytes);
                                println!("Image encoded to base64");
                                insert_content_if_not_exists(&pool, "image", base64_image).await;
                                println!("Image inserted into database");
                            });
                        },
                        Err(e) => {
                            println!("Error reading image from clipboard: {:?}", e);
                        }
                    }
                }
            }
            _ => {}
        })
        .unwrap();
    });
}

async fn insert_content_if_not_exists(pool: &SqlitePool, content_type: &str, content: String) {
    // Check if content already exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM history WHERE content_type = ? AND content = ?)")
        .bind(content_type)
        .bind(&content)
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if !exists {
        let id: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let _ = sqlx::query("INSERT INTO history (id, content_type, content) VALUES (?, ?, ?)")
            .bind(id)
            .bind(content_type)
            .bind(content)
            .execute(pool)
            .await;
    }
}