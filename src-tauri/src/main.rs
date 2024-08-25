#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod utils;

use tauri::{Manager, Listener};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_prevent_default::Flags;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(api::updater::init())
        .plugin(api::database::init())
        .plugin(api::tray::init())
        .plugin(api::hotkeys::init())
        .plugin(api::clipboard::init())
        .plugin(
            tauri_plugin_prevent_default::Builder::new()
                .with_flags(Flags::all().difference(Flags::CONTEXT_MENU))
                .build(),
        )
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            api::clipboard::set_app_data_dir(app_data_dir);

            if let Some(window) = app.get_webview_window("main") {
                utils::commands::center_window_on_current_monitor(&window);
                window.hide().unwrap();
            }

            let update_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                api::updater::check_for_updates(update_handle).await;
            });

            let monitor_handle = app_handle.clone();
            app_handle.listen("database_initialized", move |_| {
                let _ = api::clipboard::start_monitor(monitor_handle.clone());
            });

            Ok(())
        })
        .on_window_event(|app, event| {
            #[cfg(not(dev))]
            if let tauri::WindowEvent::Focused(false) = event {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            api::clipboard::simulate_paste,
            api::clipboard::get_image_path,
            api::clipboard::read_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}