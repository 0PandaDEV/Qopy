#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod api;
mod db;
mod utils;
mod sync;

use sqlx::sqlite::SqlitePoolOptions;
use std::fs;
use tauri::Manager;
use tauri_plugin_aptabase::{ EventTracker, InitOptions };
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_prevent_default::Flags;
use sync::sync::ClipboardSync;
use sync::pairing::PairingManager;
use tokio::sync::Mutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {
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

            // Create the pool in a separate tokio runtime
            let pool = tokio::runtime::Runtime
                ::new()
                .unwrap()
                .block_on(async {
                    SqlitePoolOptions::new()
                        .max_connections(5)
                        .connect(&db_url).await
                        .expect("Failed to create pool")
                });

            app_handle.manage(pool);

            let main_window = app.get_webview_window("main");

            db::database::setup(app).expect("Failed to setup database");
            api::hotkeys::setup(app_handle.clone());
            api::tray::setup(app).expect("Failed to setup tray");
            api::clipboard::setup(&app_handle);
            api::clipboard::start_monitor(app_handle.clone()).expect("Failed to start monitor");

            let pairing_manager = PairingManager::new();
            let encryption_key = pairing_manager.get_encryption_key().clone();
            let nonce = pairing_manager.get_nonce().clone();
            app_handle.manage(pairing_manager);

            let clipboard_sync = ClipboardSync::new(&encryption_key, &nonce);
            let clipboard_sync_arc = Arc::new(Mutex::new(clipboard_sync));
            app_handle.manage(clipboard_sync_arc.clone());

            let clipboard_sync_clone = clipboard_sync_arc.clone();
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let sync = clipboard_sync_clone.lock().await;
                sync.listen_webhook(app_handle_clone, clipboard_sync_clone).await;
            });

            utils::commands::center_window_on_current_monitor(main_window.as_ref().unwrap());
            let _ = main_window
                .as_ref()
                .map(|w| w.hide())
                .expect("Failed to hide window");

            app.track_event("app_started", None).expect("Failed to track event");

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
                utils::commands::fetch_page_meta,
                sync::pairing::initiate_pairing,
                sync::pairing::complete_pairing,
                sync::sync::send_clipboard_data,
                sync::sync::receive_clipboard_data
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
