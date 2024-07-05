use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use clipboard_win::{formats, get_clipboard, is_format_avail};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rdev::{listen, simulate, EventType, Key};
use sqlx::SqlitePool;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::Manager;
use tokio::runtime::Runtime;

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
                    let pool = app_handle.state::<SqlitePool>();
                    let rt = app_handle.state::<Runtime>();

                    if let Ok(content) = get_clipboard(formats::Unicode) {
                        rt.block_on(async {
                            insert_content_if_not_exists(&pool, "text", content).await;
                        });
                    }

                    if is_format_avail(formats::Bitmap.into()) {
                        match get_clipboard(formats::Bitmap) {
                            Ok(image) => {
                                rt.block_on(async {
                                    let base64_image = STANDARD.encode(&image);
                                    insert_content_if_not_exists(&pool, "image", base64_image)
                                        .await;
                                });
                            }
                            Err(e) => {
                                println!("Error reading image from clipboard: {:?}", e);
                            }
                        }
                    } else {
                        println!("No image format available in clipboard");
                    }
                }
            }
            _ => {}
        })
        .unwrap();
    });
}

async fn insert_content_if_not_exists(pool: &SqlitePool, content_type: &str, content: String) {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM history WHERE content_type = ? AND content = ?)",
    )
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
