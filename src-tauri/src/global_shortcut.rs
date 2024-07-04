use rdev::{listen, EventType, Key};
use std::sync::mpsc;
use tauri::Manager;

pub fn setup(app_handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        listen(move |event| match event.event_type {
            EventType::KeyPress(Key::MetaLeft | Key::MetaRight) => {
                let _ = tx.send(true);
            }
            EventType::KeyRelease(Key::KeyV) => {
                if rx.try_recv().is_ok() {
                    let window = app_handle.get_window("main").unwrap();
                    let is_visible = window.is_visible().unwrap();
                    if is_visible {
                        window.hide().unwrap();
                    } else {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
            }
            _ => {}
        })
        .unwrap();
    });
}
