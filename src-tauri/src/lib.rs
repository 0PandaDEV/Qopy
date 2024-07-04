use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rdev::{listen, EventType, Key};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::fs;
use std::sync::mpsc;
use tokio::runtime::Runtime;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager
};
use tauri::image::Image;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = mpsc::channel();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            let window = app.get_window("main").unwrap();
            let window_clone_for_tray = window.clone();
            let window_clone_for_click = window.clone();

            let _tray = TrayIconBuilder::new()
                .menu(
                    &MenuBuilder::new(app)
                        .items(&[&MenuItemBuilder::with_id("show", "Show/Hide").build(app)?])
                        .items(&[&MenuItemBuilder::with_id("quit", "Quit").build(app)?])
                        .build()?,
                )
                .on_menu_event(move |_app, event| match event.id().as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        let is_visible = window_clone_for_tray.is_visible().unwrap();
                        if is_visible {
                            window_clone_for_tray.hide().unwrap();
                        } else {
                            window_clone_for_tray.show().unwrap();
                            window_clone_for_tray.set_focus().unwrap();
                        }
                    }
                    _ => (),
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == MouseButton::Left {
                            let is_visible = window_clone_for_click.is_visible().unwrap();
                            if is_visible {
                                window_clone_for_click.hide().unwrap();
                            } else {
                                window_clone_for_click.show().unwrap();
                            }
                        }
                    }
                })
                .icon(Image::from_path("icons/Square71x71Logo.png").unwrap())
                .build(app)?;

            //////////////////////////////////////
            
            let rt = Runtime::new().expect("Failed to create Tokio runtime");

            let app_data_dir = app.path().app_data_dir().unwrap();
            fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("data.db");
            let is_new_db = !db_path.exists();
            if is_new_db {
                fs::File::create(&db_path).expect("Failed to create database file");
            }

            let db_url = format!("sqlite:{}", db_path.to_str().unwrap());
            let pool = rt.block_on(async {
                SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&db_url)
                    .await
                    .expect("Failed to create pool")
            });

            rt.block_on(async {
                sqlx::query(
                    "CREATE TABLE IF NOT EXISTS history (
                        id TEXT PRIMARY KEY,
                        content TEXT NOT NULL,
                        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
                    )"
                )
                .execute(&pool)
                .await
                .expect("Failed to create table");

                if is_new_db {
                    let id: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(16)
                        .map(char::from)
                        .collect();
                    sqlx::query("INSERT INTO history (id, content) VALUES (?, ?)")
                        .bind(id)
                        .bind("Welcome to your clipboard history!")
                        .execute(&pool)
                        .await
                        .expect("Failed to insert welcome message");
                }
            });

            app.manage(pool);
            app.manage(rt);

            if let Some(window) = app.get_window("main") {
                let _ = window.restore_state(StateFlags::POSITION);
                window.show().unwrap();
            }

            std::thread::spawn(move || {
                listen(move |event| match event.event_type {
                    EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => {
                        let _ = tx.send(true);
                    }
                    EventType::KeyRelease(Key::KeyC) => {
                        if rx.try_recv().is_ok() {
                            match app_handle.clipboard().read_text() {
                                Ok(content) => {
                                    let pool = app_handle.state::<SqlitePool>();
                                    let rt = app_handle.state::<Runtime>();
                                    rt.block_on(async {
                                        let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM history WHERE content = ?)")
                                            .bind(&content)
                                            .fetch_one(&*pool)
                                            .await
                                            .unwrap_or(false);

                                        if !exists {
                                            let id: String = thread_rng()
                                                .sample_iter(&Alphanumeric)
                                                .take(16)
                                                .map(char::from)
                                                .collect();
                                            let _ = sqlx::query("INSERT INTO history (id, content) VALUES (?, ?)")
                                                .bind(id)
                                                .bind(content)
                                                .execute(&*pool)
                                                .await;
                                        }
                                    });
                                },
                                Err(e) => eprintln!("Error reading clipboard: {:?}", e),
                            }
                        }
                    }
                    _ => {}
                })
                .unwrap();
            });
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
