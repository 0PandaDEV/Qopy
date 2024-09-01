use rdev::{listen, Event, EventType, Key};
use tauri::{Manager, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use serde::Serialize;

use crate::utils::commands::center_window_on_current_monitor;

static IS_CAPTURING_KEYBIND: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
struct CapturedKeybind {
    modifiers: Vec<String>,
    key: String,
}

struct KeybindState {
    pressed_keys: HashSet<Key>,
}

impl KeybindState {
    fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }
}

pub fn setup(app_handle: tauri::AppHandle) {
    let app_handle_clone = app_handle.clone();
    let keybind_state = Arc::new(Mutex::new(KeybindState::new()));

    std::thread::spawn(move || {
        if let Err(e) = listen(move |event| {
            let mut state = keybind_state.lock().unwrap();
            if IS_CAPTURING_KEYBIND.load(Ordering::SeqCst) {
                handle_keybind_capture(&app_handle_clone, event, &mut state);
            } else {
                handle_normal_hotkey(&app_handle_clone, event, &mut state);
            }
        }) {
            eprintln!("Error setting up event listener: {:?}", e);
        }
    });
}

fn handle_normal_hotkey(app_handle: &tauri::AppHandle, event: Event, state: &mut KeybindState) {
    match event.event_type {
        EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => {
            state.pressed_keys.insert(Key::MetaLeft);
        }
        EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => {
            state.pressed_keys.remove(&Key::MetaLeft);
        }
        EventType::KeyPress(Key::KeyV) => {
            if state.pressed_keys.contains(&Key::MetaLeft) {
                state.pressed_keys.clear();
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    center_window_on_current_monitor(&window);
                }
            }
        }
        _ => {}
    }
}

fn handle_keybind_capture(app_handle: &tauri::AppHandle, event: Event, state: &mut KeybindState) {
    match event.event_type {
        EventType::KeyPress(key) => {
            state.pressed_keys.insert(key);
            update_captured_keybind(app_handle, &state.pressed_keys);
        }
        EventType::KeyRelease(key) => {
            state.pressed_keys.remove(&key);
        }
        _ => {}
    }
}

fn update_captured_keybind(app_handle: &tauri::AppHandle, pressed_keys: &HashSet<Key>) {
    let modifiers: Vec<String> = vec![Key::ControlLeft, Key::ShiftLeft, Key::Alt, Key::MetaLeft]
        .into_iter()
        .filter(|key| pressed_keys.contains(key))
        .map(|key| key_to_string(key))
        .collect();

    let key = pressed_keys.iter()
        .find(|&&key| !vec![Key::ControlLeft, Key::ShiftLeft, Key::Alt, Key::MetaLeft].contains(&key))
        .map(|&key| key_to_string(key));

    if let Some(key) = key {
        let captured_keybind = CapturedKeybind {
            modifiers,
            key,
        };
        if let Err(e) = app_handle.emit("keybind_captured", captured_keybind) {
            eprintln!("Error emitting keybind_captured event: {:?}", e);
        }
    }
}

fn key_to_string(key: Key) -> String {
    match key {
        Key::ControlLeft | Key::ControlRight => "Ctrl".to_string(),
        Key::ShiftLeft | Key::ShiftRight => "Shift".to_string(),
        Key::Alt => "Alt".to_string(),
        Key::MetaLeft | Key::MetaRight => "Meta".to_string(),
        _ => format!("{:?}", key),
    }
}

#[tauri::command]
pub fn start_keybind_capture() {
    IS_CAPTURING_KEYBIND.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn stop_keybind_capture() {
    IS_CAPTURING_KEYBIND.store(false, Ordering::SeqCst);
}