use rdev::{listen, EventType, Key};
use std::sync::{Arc, Mutex, mpsc};
use tauri::Manager;

pub fn setup(app_handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();
    let meta_pressed = Arc::new(Mutex::new(false));

    std::thread::spawn({
        let meta_pressed = Arc::clone(&meta_pressed);
        move || {
            listen(move |event| {
                match event.event_type {
                    EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => {
                        let mut meta = meta_pressed.lock().unwrap();
                        *meta = true;
                        tx.send(event).unwrap();
                    }
                    EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => {
                        let mut meta = meta_pressed.lock().unwrap();
                        *meta = false;
                    }
                    _ => {
                        tx.send(event).unwrap();
                    }
                }
            })
            .unwrap();
        }
    });

    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            let meta = meta_pressed.lock().unwrap();
            if *meta && matches!(event.event_type, EventType::KeyPress(Key::KeyV)) {
                println!("Meta and Key V pressed");
                let window = app_handle.get_webview_window("main").unwrap();
                let is_visible = window.is_visible().unwrap();
                if is_visible {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
        }
    });
}
