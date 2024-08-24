use tauri::PhysicalPosition;

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