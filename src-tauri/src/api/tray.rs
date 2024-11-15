use tauri::{
    menu::{MenuBuilder, MenuItemBuilder}, tray::TrayIconBuilder, Emitter, Manager
};

#[cfg(target_os = "windows")]
mod platform {
    use std::ptr::null_mut;
    use winapi::um::winuser::{AttachThreadInput, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow};

    pub fn manage_focus(window: &tauri::Window) {
        unsafe {
            let foreground_window = GetForegroundWindow();
            let current_thread_id = GetWindowThreadProcessId(foreground_window, null_mut());
            let target_thread_id = GetWindowThreadProcessId(window.hwnd() as _, null_mut());

            AttachThreadInput(current_thread_id, target_thread_id, 1);
            SetForegroundWindow(window.hwnd() as _);
            AttachThreadInput(current_thread_id, target_thread_id, 0);
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use cocoa::appkit::NSWindow;
    use cocoa::base::id;
    use objc::runtime::YES;

    pub fn manage_focus(window: &tauri::Window) {
        unsafe {
            let ns_window: id = window.ns_window().unwrap() as _;
            ns_window.makeKeyAndOrderFront_(YES);
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use x11::xlib::{Display, XSetInputFocus, XDefaultRootWindow, XOpenDisplay, XCloseDisplay, RevertToParent};

    pub fn manage_focus(window: &tauri::Window) {
        unsafe {
            let display: *mut Display = XOpenDisplay(null_mut());
            let root_window = XDefaultRootWindow(display);
            XSetInputFocus(display, root_window, RevertToParent, 0);
            XCloseDisplay(display);
        }
    }
}

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").unwrap();
    let window_clone_for_tray = window.clone();

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
                    platform::manage_focus(&window_clone_for_tray);
                }
                window_clone_for_tray.emit("main_route", ()).unwrap();
            }
            "keybind" => {
                window_clone_for_tray.emit("change_keybind", ()).unwrap();
            }
            _ => (),
        })
        .icon(icon)
        .build(app)?;

    Ok(())
}
