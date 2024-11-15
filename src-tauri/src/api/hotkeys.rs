use crate::api::database::get_keybind;
use crate::utils::commands::center_window_on_current_monitor;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::str::FromStr;
use tauri::{AppHandle, Listener, Manager};

pub fn setup(app_handle: tauri::AppHandle) {
    let app_handle_clone = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        match get_keybind(app_handle_clone.clone()).await {
            Ok(keybind) => {
                if !keybind.is_empty() {
                    let keybind_str = keybind.join("+");
                    println!("Keybind: {:?}", keybind_str);
                    if let Err(e) = register_shortcut(&app_handle_clone, &keybind_str) {
                        eprintln!("Error registering shortcut: {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error getting keybind: {:?}", e);
            }
        }
    });

    let app_handle_for_listener = app_handle.clone();
    app_handle.listen("update-shortcut", move |event| {
        let payload_str = event.payload().to_string();
        if let Err(e) = register_shortcut(&app_handle_for_listener, &payload_str) {
            eprintln!("Error re-registering shortcut: {:?}", e);
        }
    });

    let app_handle_for_hotkey = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            if let Ok(_) = GlobalHotKeyEvent::receiver().recv() {
                handle_hotkey_event(&app_handle_for_hotkey);
            }
        }
    });
}

fn register_shortcut(
    _app_handle: &tauri::AppHandle,
    shortcut: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let manager = GlobalHotKeyManager::new()?;
    let hotkey = parse_hotkey(shortcut)?;
    manager.register(hotkey)?;

    println!("Listening for keybind: {}", shortcut);
    Ok(())
}

fn parse_hotkey(shortcut: &str) -> Result<HotKey, Box<dyn std::error::Error>> {
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in shortcut.split('+') {
        let part = part;
        if part.to_lowercase().starts_with("ctrl") || part.to_lowercase().starts_with("control") {
            modifiers |= Modifiers::CONTROL;
        } else if part.to_lowercase().starts_with("alt") {
            modifiers |= Modifiers::ALT;
        } else if part.to_lowercase().starts_with("shift") {
            modifiers |= Modifiers::SHIFT;
        } else if part.to_lowercase().starts_with("super") || part.to_lowercase().starts_with("meta") || part.to_lowercase().starts_with("cmd") {
            
            modifiers |= Modifiers::META;
        } else {
            let pascal_case_key = part
                .split(|c: char| !c.is_alphanumeric())
                .map(|word| {
                    let mut chars = word.chars();
                    let first_char = chars.next().unwrap().to_uppercase().collect::<String>();
                    let rest = chars.as_str();
                    first_char + rest
                })
                .collect::<String>();
            code = Some(
                Code::from_str(&pascal_case_key)
                    .map_err(|_| format!("Invalid key: {}", pascal_case_key))?,
            );
        }
    }

    Ok(HotKey::new(Some(modifiers), code.unwrap()))
}

fn handle_hotkey_event(app_handle: &AppHandle) {
    let window = app_handle.get_webview_window("main").unwrap();
    if window.is_visible().unwrap() {
        window.hide().unwrap();
    } else {
        window.show().unwrap();
        window.set_focus().unwrap();
        center_window_on_current_monitor(&window);
    }
}
