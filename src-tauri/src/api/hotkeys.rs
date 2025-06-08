use crate::utils::commands::center_window_on_current_monitor;
use crate::utils::keys::KeyCode;
use global_hotkey::{
    hotkey::{ Code, HotKey, Modifiers },
    GlobalHotKeyEvent,
    GlobalHotKeyManager,
    HotKeyState,
};
use parking_lot::Mutex;
use std::str::FromStr;
use std::sync::Arc;
use tauri::{ AppHandle, Manager, Listener };
use tauri_plugin_aptabase::EventTracker;

#[derive(Default)]
struct HotkeyState {
    manager: Option<GlobalHotKeyManager>,
    registered_hotkey: Option<HotKey>,
}

unsafe impl Send for HotkeyState {}

pub fn setup(app_handle: tauri::AppHandle) {
    let state = Arc::new(Mutex::new(HotkeyState::default()));
    let manager = match GlobalHotKeyManager::new() {
        Ok(manager) => manager,
        Err(err) => {
            eprintln!("Failed to initialize hotkey manager: {:?}", err);
            return;
        }
    };

    {
        let mut hotkey_state = state.lock();
        hotkey_state.manager = Some(manager);
    }

    let rt = app_handle.state::<tokio::runtime::Runtime>();
    let initial_keybind = rt
        .block_on(crate::db::settings::get_keybind(app_handle.clone()))
        .expect("Failed to get initial keybind");

    if let Err(e) = register_shortcut(&state, &initial_keybind) {
        eprintln!("Error registering initial shortcut: {:?}", e);
    }

    let state_clone = Arc::clone(&state);
    app_handle.listen("update-shortcut", move |event| {
        let payload_str = event.payload().replace("\\\"", "\"");
        let trimmed_str = payload_str.trim_matches('"');
        unregister_current_hotkey(&state_clone);
        
        let payload: Vec<String> = serde_json::from_str(trimmed_str).unwrap_or_default();
        if let Err(e) = register_shortcut(&state_clone, &payload) {
            eprintln!("Error re-registering shortcut: {:?}", e);
        }
    });

    let state_clone = Arc::clone(&state);
    app_handle.listen("save_keybind", move |event| {
        let payload_str = event.payload().to_string();
        unregister_current_hotkey(&state_clone);
        
        let payload: Vec<String> = serde_json::from_str(&payload_str).unwrap_or_default();
        if let Err(e) = register_shortcut(&state_clone, &payload) {
            eprintln!("Error registering saved shortcut: {:?}", e);
        }
    });

    setup_hotkey_receiver(app_handle);
}

fn setup_hotkey_receiver(app_handle: AppHandle) {
    std::thread::spawn(move || {
        loop {
            match GlobalHotKeyEvent::receiver().recv() {
                Ok(event) => {
                    if event.state == HotKeyState::Released {
                        continue;
                    }
                    handle_hotkey_event(&app_handle);
                }
                Err(e) => eprintln!("Error receiving hotkey event: {:?}", e),
            }
        }
    });
}

fn unregister_current_hotkey(state: &Arc<Mutex<HotkeyState>>) {
    let mut hotkey_state = state.lock();
    if let Some(old_hotkey) = hotkey_state.registered_hotkey.take() {
        if let Some(manager) = &hotkey_state.manager {
            let _ = manager.unregister(old_hotkey);
        }
    }
}

fn register_shortcut(state: &Arc<Mutex<HotkeyState>>, shortcut: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let hotkey = parse_hotkey(shortcut)?;
    let mut hotkey_state = state.lock();

    if let Some(manager) = &hotkey_state.manager {
        manager.register(hotkey.clone())?;
        hotkey_state.registered_hotkey = Some(hotkey);
        Ok(())
    } else {
        Err("Hotkey manager not initialized".into())
    }
}

fn parse_hotkey(shortcut: &[String]) -> Result<HotKey, Box<dyn std::error::Error>> {
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in shortcut {
        match part.as_str() {
            "ControlLeft" => modifiers |= Modifiers::CONTROL,
            "AltLeft" => modifiers |= Modifiers::ALT,
            "ShiftLeft" => modifiers |= Modifiers::SHIFT,
            "MetaLeft" => modifiers |= Modifiers::META,
            key => code = Some(Code::from(KeyCode::from_str(key)?)),
        }
    }

    let key_code = code.ok_or_else(|| "No valid key code found".to_string())?;
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

    let _ = app_handle.track_event(
        "hotkey_triggered",
        Some(
            serde_json::json!({
                "action": if window.is_visible().unwrap() { "hide" } else { "show" }
            })
        )
    );
}