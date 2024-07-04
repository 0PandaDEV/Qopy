use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    image::Image,
};

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}