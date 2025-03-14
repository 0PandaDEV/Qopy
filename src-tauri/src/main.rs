#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod api;
mod db;
mod utils;

use sqlx::sqlite::SqlitePoolOptions;
use std::fs;
use tauri::Manager;
use tauri_plugin_aptabase::{ EventTracker, InitOptions };
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_prevent_default::Flags;

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let _guard = runtime.enter();

    tauri::Builder
        ::default()
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(
            tauri_plugin_aptabase::Builder
                ::new("A-SH-8937252746")
                .with_options(InitOptions {
                    host: Some("https://aptabase.pandadev.net".to_string()),
                    flush_interval: None,
                })
                .with_panic_hook(
                    Box::new(|client, info, msg| {
                        let location = info
                            .location()
                            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
                            .unwrap_or_else(|| "".to_string());

                        let _ = client.track_event(
                            "panic",
                            Some(
                                serde_json::json!({
                            "info": format!("{} ({})", msg, location),
                        })
                            )
                        );
                    })
                )
                .build()
        )
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
        .plugin(
            tauri_plugin_prevent_default::Builder
                ::new()
                .with_flags(Flags::all().difference(Flags::CONTEXT_MENU))
                .build()
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            
            let app_data_dir = app.path().app_data_dir().unwrap();
            utils::logger::init_logger(&app_data_dir).expect("Failed to initialize logger");

            fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("data.db");
            let is_new_db = !db_path.exists();
            if is_new_db {
                fs::File::create(&db_path).expect("Failed to create database file");
            }

            let db_url = format!("sqlite:{}", db_path.to_str().unwrap());

            let app_handle = app.handle().clone();

            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&db_url).await
                    .expect("Failed to create pool");

                app_handle_clone.manage(pool);
            });

            let main_window = app.get_webview_window("main");

            let _ = db::database::setup(app);
            api::hotkeys::setup(app_handle.clone());
            api::tray::setup(app)?;
            api::clipboard::setup(app.handle());
            let _ = api::clipboard::start_monitor(app_handle.clone());

            utils::commands::center_window_on_current_monitor(main_window.as_ref().unwrap());
            main_window
                .as_ref()
                .map(|w| w.hide())
                .unwrap_or(Ok(()))?;

            let _ = app.track_event("app_started", None);

            tauri::async_runtime::spawn(async move {
                api::updater::check_for_updates(app_handle, false).await;
            });

            Ok(())
        })
        .on_window_event(|_app, _event| {
            #[cfg(not(dev))]
            if let tauri::WindowEvent::Focused(false) = _event {
                if let Some(window) = _app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(
            tauri::generate_handler![
                api::clipboard::write_and_paste,
                db::history::get_history,
                db::history::add_history_item,
                db::history::search_history,
                db::history::load_history_chunk,
                db::history::delete_history_item,
                db::history::clear_history,
                db::history::read_image,
                db::settings::get_setting,
                db::settings::save_setting,
                utils::commands::fetch_page_meta
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}