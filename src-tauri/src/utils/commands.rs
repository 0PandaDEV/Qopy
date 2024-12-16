use active_win_pos_rs::get_active_window;
use base64::{engine::general_purpose::STANDARD, Engine};
use image::codecs::png::PngEncoder;
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

pub fn get_app_info() -> (String, Option<String>) {
    match get_active_window() {
        Ok(window) => {
            let app_name = window.app_name;
            (app_name, None)
        }
        Err(_) => ("System".to_string(), None),
    }
}

fn _process_icon_to_base64(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let img = image::open(path)?;
    let resized = img.resize(128, 128, image::imageops::FilterType::Lanczos3);
    let mut png_buffer = Vec::new();
    resized.write_with_encoder(PngEncoder::new(&mut png_buffer))?;
    Ok(STANDARD.encode(png_buffer))
}
