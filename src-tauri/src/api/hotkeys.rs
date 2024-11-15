use crate::utils::commands::center_window_on_current_monitor;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Listener};

lazy_static::lazy_static! {
    static ref HOTKEY_MANAGER: Mutex<Option<GlobalHotKeyManager>> = Mutex::new(None);
}

pub fn setup(app_handle: tauri::AppHandle) {
    let app_handle_clone = app_handle.clone();

    let manager = GlobalHotKeyManager::new().expect("Failed to initialize hotkey manager");
    *HOTKEY_MANAGER.lock().unwrap() = Some(manager);

    let rt = app_handle.state::<tokio::runtime::Runtime>();
    let initial_keybind = rt.block_on(crate::api::database::get_keybind(app_handle_clone.clone()))
        .expect("Failed to get initial keybind");
    let initial_shortcut = initial_keybind.join("+");
    
    let initial_shortcut_for_update = initial_shortcut.clone();
    let initial_shortcut_for_save = initial_shortcut.clone();
    
    if let Err(e) = register_shortcut(&initial_shortcut) {
        eprintln!("Error registering initial shortcut: {:?}", e);
    }

    app_handle.listen("update-shortcut", move |event| {
        let payload_str = event.payload().to_string();
        
        if let Ok(old_hotkey) = parse_hotkey(&initial_shortcut_for_update) {
            if let Some(manager) = HOTKEY_MANAGER.lock().unwrap().as_ref() {
                let _ = manager.unregister(old_hotkey);
            }
        }

        if let Err(e) = register_shortcut(&payload_str) {
            eprintln!("Error re-registering shortcut: {:?}", e);
        }
    });

    app_handle.listen("save_keybind", move |event| {
        let payload_str = event.payload().to_string();
        
        if let Ok(old_hotkey) = parse_hotkey(&initial_shortcut_for_save) {
            if let Some(manager) = HOTKEY_MANAGER.lock().unwrap().as_ref() {
                let _ = manager.unregister(old_hotkey);
            }
        }

        if let Err(e) = register_shortcut(&payload_str) {
            eprintln!("Error registering saved shortcut: {:?}", e);
        }
    });

    let app_handle_for_hotkey = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            match GlobalHotKeyEvent::receiver().recv() {
                Ok(event) => {
                    if event.state == HotKeyState::Released {
                        continue;
                    }
                    handle_hotkey_event(&app_handle_for_hotkey);
                }
                Err(e) => {
                    eprintln!("Error receiving hotkey event: {:?}", e);
                }
            }
        }
    });
}

fn register_shortcut(shortcut: &str) -> Result<(), Box<dyn std::error::Error>> {
    let hotkey = parse_hotkey(shortcut)?;
    if let Some(manager) = HOTKEY_MANAGER.lock().unwrap().as_ref() {
        manager.register(hotkey)?;
    }
    Ok(())
}

fn parse_hotkey(shortcut: &str) -> Result<HotKey, Box<dyn std::error::Error>> {
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    let shortcut = shortcut.replace("\"", "");

    for part in shortcut.split('+') {
        let part = part.trim().to_lowercase();
        match part.as_str() {
            "ctrl" | "control" | "controlleft" => modifiers |= Modifiers::CONTROL,
            "alt" | "altleft" | "optionleft" => modifiers |= Modifiers::ALT,
            "shift" | "shiftleft" => modifiers |= Modifiers::SHIFT,
            "super" | "meta" | "cmd" | "metaleft" => modifiers |= Modifiers::META,
            key => {
                let key_code = if key.starts_with("key") {
                    "Key".to_string() + &key[3..].to_uppercase()
                } else if key.len() == 1 && key.chars().next().unwrap().is_alphabetic() {
                    "Key".to_string() + &key.to_uppercase()
                } else {
                    key.to_string()
                };
                
                code = Some(Code::from_str(&key_code)
                    .map_err(|_| format!("Invalid key code: {}", key_code))?);
            }
        }
    }

    let key_code = code.ok_or_else(|| format!("No valid key code found in shortcut: {}", shortcut))?;
    Ok(HotKey::new(Some(modifiers), key_code))
}

fn handle_hotkey_event(app_handle: &AppHandle) {
    let window = app_handle.get_webview_window("main").unwrap();
    if window.is_visible().unwrap() {
        window.hide().unwrap();
    } else {
        window.set_always_on_top(true).unwrap();
        window.show().unwrap();
        window.set_focus().unwrap();

        let window_clone = window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            window_clone.set_always_on_top(false).unwrap();
        });

        center_window_on_current_monitor(&window);
    }
}
