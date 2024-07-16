use rdev::{listen, EventType, Key};
use std::sync::mpsc;
use tauri::Manager;

pub fn setup(app_handle: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        listen(move |event| {
            tx.send(event).unwrap();
        })
        .unwrap();
    });

    std::thread::spawn(move || {
        let mut meta_pressed = false;
        
        while let Ok(event) = rx.recv() {
            match event.event_type {
                EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => {
                    meta_pressed = true;
                    println!("Meta key pressed");
                }
                EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => {
                    meta_pressed = false;
                    println!("Meta key released");
                }
                EventType::KeyPress(Key::KeyV) => {
                    println!("V key pressed");
                    if meta_pressed {
                        println!("Meta+V detected");
                        let window = app_handle.get_webview_window("main").unwrap();
                        let is_visible = window.is_visible().unwrap();
                        if is_visible {
                            println!("Hiding window");
                            window.hide().unwrap();
                        } else {
                            println!("Showing window");
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
    });
}
