use rdev::{listen, simulate, EventType, Key};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use sqlx::SqlitePool;
use tokio::runtime::Runtime;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

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
                    match app_handle.clipboard().read_text() {
                        Ok(content) => {
                            let pool = app_handle.state::<SqlitePool>();
                            let rt = app_handle.state::<Runtime>();
                            rt.block_on(async {
                                let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM history WHERE content = ?)")
                                    .bind(&content)
                                    .fetch_one(&*pool)
                                    .await
                                    .unwrap_or(false);

                                if !exists {
                                    let id: String = thread_rng()
                                        .sample_iter(&Alphanumeric)
                                        .take(16)
                                        .map(char::from)
                                        .collect();
                                    let _ = sqlx::query("INSERT INTO history (id, content) VALUES (?, ?)")
                                        .bind(id)
                                        .bind(content)
                                        .execute(&*pool)
                                        .await;
                                }
                            });
                        },
                        Err(e) => eprintln!("Error reading clipboard: {:?}", e),
                    }
                }
            }
            _ => {}
        })
        .unwrap();
    });
}
