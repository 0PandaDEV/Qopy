use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_aptabase::EventTracker;

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").unwrap();
    let is_visible = window.is_visible().unwrap();
    let _ = app.track_event("tray_toggle", Some(serde_json::json!({
        "action": if is_visible { "hide" } else { "show" }
    })));

    let icon_bytes = include_bytes!("../../icons/Square71x71Logo.png");
    let icon = tauri::image::Image::from_bytes(icon_bytes).unwrap();

    let _tray = TrayIconBuilder::new()
        .menu(
            &MenuBuilder::new(app)
                .items(&[&MenuItemBuilder::with_id("app_name", "Qopy")
                    .enabled(false)
                    .build(app)?])
                .items(&[&MenuItemBuilder::with_id("show", "Show/Hide").build(app)?])
                .items(&[&MenuItemBuilder::with_id("keybind", "Change keybind").build(app)?])
                .items(&[&MenuItemBuilder::with_id("check_updates", "Check for updates").build(app)?])
                .items(&[&MenuItemBuilder::with_id("quit", "Quit").build(app)?])
                .build()?,
        )
        .on_menu_event(move |_app, event| match event.id().as_ref() {
            "quit" => {
                let _ = _app.track_event("app_quit", None);
                std::process::exit(0);
            }
            "show" => {
                let _ = _app.track_event("tray_toggle", Some(serde_json::json!({
                    "action": if is_visible { "hide" } else { "show" }
                })));
                let is_visible = window.is_visible().unwrap();
                if is_visible {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                window.emit("main_route", ()).unwrap();
            }
            "keybind" => {
                let _ = _app.track_event("tray_keybind_change", None);
                window.emit("change_keybind", ()).unwrap();
            }
            "check_updates" => {
                let _ = _app.track_event("tray_check_updates", None);
                let app_handle = _app.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    crate::api::updater::check_for_updates(app_handle, true).await;
                });
            }
            _ => (),
        })
        .icon(icon)
        .build(app)?;

    Ok(())
}
