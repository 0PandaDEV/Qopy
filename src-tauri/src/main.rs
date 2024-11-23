#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod api;
mod utils;

use tauri::window::{Effect, EffectState, EffectsBuilder};
use tauri::Manager;
use tauri::WebviewUrl;
use tauri::WebviewWindow;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_prevent_default::Flags;

fn main() {
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

            let main_window = if let Some(window) = app.get_webview_window("main") {
                window
            } else {
                WebviewWindow::builder(app.handle(), "main", WebviewUrl::App("index.html".into()))
                    .title("Qopy")
                    .resizable(false)
                    .fullscreen(false)
                    .inner_size(750.0, 474.0)
                    .focused(true)
                    .skip_taskbar(true)
                    .visible(false)
                    .decorations(false)
                    .transparent(true)
                    .always_on_top(true)
                    .content_protected(true)
                    .visible_on_all_workspaces(true)
                    .build()?
            };

            let _ = api::database::setup(app);
            api::hotkeys::setup(app_handle.clone());
            api::tray::setup(app)?;
            api::clipboard::setup(app.handle());
            let _ = api::clipboard::start_monitor(app_handle.clone());

            utils::commands::center_window_on_current_monitor(&main_window);
            main_window.hide()?;

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            api::clipboard::set_app_data_dir(app_data_dir);

            tauri::async_runtime::spawn(async move {
                api::updater::check_for_updates(app_handle).await;
            });

            main_window.set_effects(
                EffectsBuilder::new()
                    .effect(Effect::Popover)
                    .state(EffectState::Active)
                    .build(),
            )?;

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
        .invoke_handler(tauri::generate_handler![
            api::clipboard::get_image_path,
            api::clipboard::write_and_paste,
            api::clipboard::read_image,
            api::database::save_keybind,
            api::database::get_keybind
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
