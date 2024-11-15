use crate::api::database::get_keybind;
use crate::utils::commands::center_window_on_current_monitor;
use rdev::{listen, EventType, Key};
use tauri::Manager;

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr::null_mut;
    use winapi::um::winuser::{AttachThreadInput, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow};

    pub fn manage_focus(window: &tauri::Window) {
        unsafe {
            let foreground_window = GetForegroundWindow();
            let current_thread_id = GetWindowThreadProcessId(foreground_window, null_mut());
            let target_thread_id = GetWindowThreadProcessId(window.hwnd() as _, null_mut());

            AttachThreadInput(current_thread_id, target_thread_id, 1);
            SetForegroundWindow(window.hwnd() as _);
            AttachThreadInput(current_thread_id, target_thread_id, 0);
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use cocoa::appkit::NSWindow;
    use cocoa::base::id;
    use objc::runtime::YES;

    pub fn manage_focus(window: &tauri::Window) {
        unsafe {
            let ns_window: id = window.ns_window().unwrap() as _;
            ns_window.makeKeyAndOrderFront_(YES);
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use x11::xlib::{Display, XSetInputFocus, XDefaultRootWindow, XOpenDisplay, XCloseDisplay, RevertToParent};

    pub fn manage_focus(window: &tauri::Window) {
        unsafe {
            let display: *mut Display = XOpenDisplay(null_mut());
            let root_window = XDefaultRootWindow(display);
            XSetInputFocus(display, root_window, RevertToParent, 0);
            XCloseDisplay(display);
        }
    }
}

fn key_to_string(key: &Key) -> String {
    format!("{:?}", key)
}

#[warn(dead_code)]
pub fn setup(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let keybind = tauri::async_runtime::block_on(async { get_keybind(app_handle.clone()).await.unwrap_or_default() });

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
                center_window_on_current_monitor(&window);
                platform::manage_focus(&window);
            }
        })
        .unwrap();
    });
}
