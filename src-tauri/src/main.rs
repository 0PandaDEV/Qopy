#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod clipboard_listener;
mod database;
mod global_shortcut;
mod tray;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();

            global_shortcut::setup(app_handle.clone());
            tray::setup(app)?;
            database::setup(app)?;
            clipboard_listener::setup(app_handle);

            if let Some(window) = app.get_window("main") {
                let _ = window.restore_state(StateFlags::POSITION);
                window.show().unwrap();
            }

            Ok(())
        })
        .on_window_event(|app, event| match event {
            tauri::WindowEvent::CloseRequested { .. }
            | tauri::WindowEvent::Destroyed
            | tauri::WindowEvent::Focused(false) => {
                let _ = AppHandleExt::save_window_state(app.app_handle(), StateFlags::POSITION);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![clipboard_listener::simulate_paste])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
