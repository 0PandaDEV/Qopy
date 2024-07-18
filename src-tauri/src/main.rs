#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod clipboard;
mod database;
mod hotkeys;
mod tray;
mod updater;

use tauri::Manager;
use tauri::PhysicalPosition;
use tauri_plugin_autostart::MacosLauncher;

pub fn center_window_on_current_monitor(window: &tauri::WebviewWindow) {
    if let Some(monitor) = window.available_monitors().unwrap().iter().find(|m| {
        let primary_monitor = window
            .primary_monitor()
            .unwrap()
            .expect("Failed to get primary monitor");
        let mouse_position = primary_monitor.position();
        let monitor_position = m.position();
        let monitor_size = m.size();
        mouse_position.x >= monitor_position.x
            && mouse_position.x < monitor_position.x + monitor_size.width as i32
            && mouse_position.y >= monitor_position.y
            && mouse_position.y < monitor_position.y + monitor_size.height as i32
    }) {
        let monitor_size = monitor.size();
        let window_size = window.outer_size().unwrap();

        let x = (monitor_size.width as i32 - window_size.width as i32) / 2;
        let y = (monitor_size.height as i32 - window_size.height as i32) / 2;

        window
            .set_position(PhysicalPosition::new(
                monitor.position().x + x,
                monitor.position().y + y,
            ))
            .unwrap();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            let app_handle = app.handle().clone();

            hotkeys::setup(app_handle.clone());
            tray::setup(app)?;
            database::setup(app)?;
            clipboard::setup(app_handle.clone());

            if let Some(window) = app.get_webview_window("main") {
                center_window_on_current_monitor(&window);
                window.hide().unwrap();
            }

            #[cfg(dev)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data directory");
            clipboard::set_app_data_dir(app_data_dir);

            tauri::async_runtime::spawn(async move {
                updater::check_for_updates(app_handle).await;
            });

            Ok(())
        })
        .on_window_event(|app, event| match event {
            #[cfg(not(dev))]
            tauri::WindowEvent::Focused(false) => {
                if let Some(window) = app.get_webview_window("main") {
                    window.hide().unwrap();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            clipboard::simulate_paste,
            clipboard::get_image_path,
            clipboard::read_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}