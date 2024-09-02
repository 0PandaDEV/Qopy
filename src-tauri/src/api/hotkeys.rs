use crate::api::database::get_keybind;
use crate::utils::commands::center_window_on_current_monitor;
use rdev::{listen, EventType, Key};
use tauri::Manager;

fn key_to_string(key: &Key) -> String {
    format!("{:?}", key)
}

#[warn(dead_code)]
pub fn setup(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let pool = app_handle.state::<sqlx::SqlitePool>();
        let rt = app_handle.state::<tokio::runtime::Runtime>();

        let keybind = rt.block_on(async { get_keybind(pool).await.unwrap_or_default() });

        println!("Listening for keybind: {:?}", keybind);

        let mut pressed_keys = vec![false; keybind.len()];

        listen(move |event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    if let Some(index) = keybind.iter().position(|k| k == &key_to_string(&key)) {
                        pressed_keys[index] = true;
                    }
                }
                EventType::KeyRelease(key) => {
                    if let Some(index) = keybind.iter().position(|k| k == &key_to_string(&key)) {
                        pressed_keys[index] = false;
                    }
                }
                _ => {}
            }

            if pressed_keys.iter().all(|&k| k) {
                pressed_keys.iter_mut().for_each(|k| *k = false);
                let window = app_handle.get_webview_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
                center_window_on_current_monitor(&window);
            }
        })
        .unwrap();
    });
}