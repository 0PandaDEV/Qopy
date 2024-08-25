#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod utils;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_prevent_default::Flags;

fn main() {
    #[allow(unused_variables)]
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(
            tauri_plugin_prevent_default::Builder::new()
                .with_flags(Flags::all().difference(Flags::CONTEXT_MENU))
                .build(),
        )
        .setup(|app| {
            let app_handle = app.handle().clone();

            api::hotkeys::setup(app_handle.clone());
            api::tray::setup(app)?;
            api::database::setup(app)?;
            api::clipboard::setup(app.handle());
            let _ = api::clipboard::start_monitor(app_handle.clone());

            if let Some(window) = app.get_webview_window("main") {
                utils::commands::center_window_on_current_monitor(&window);
                window.hide().unwrap();
            }

            // #[cfg(dev)]
            // {
            //     let window = app.get_webview_window("main").unwrap();
            //     window.open_devtools();
            //     window.close_devtools();
            // }

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            api::clipboard::set_app_data_dir(app_data_dir);

            tauri::async_runtime::spawn(async move {
                api::updater::check_for_updates(app_handle).await;
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
            api::clipboard::get_image_path,
            api::clipboard::write_and_paste,
            api::clipboard::read_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
