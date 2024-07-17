use rdev::{listen, EventType, Key};
use tauri::Manager;

pub fn setup(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut meta_pressed = false;
        listen(move |event| {
            match event.event_type {
                EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => {
                    meta_pressed = true;
                }
                EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => {
                    meta_pressed = false;
                }
                EventType::KeyPress(Key::KeyV) => {
                    if meta_pressed {
                        meta_pressed = false;
                        let window = app_handle.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                _ => {}
            }
        })
        .unwrap();
    });
}