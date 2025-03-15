use applications::{AppInfoContext, AppInfo, AppTrait, utils::image::RustImage};
use base64::{ engine::general_purpose::STANDARD, Engine };
use tauri::PhysicalPosition;
use meta_fetcher;

pub fn center_window_on_current_monitor(window: &tauri::WebviewWindow) {
    if
        let Some(monitor) = window
            .available_monitors()
            .unwrap()
            .iter()
            .find(|m| {
                let primary_monitor = window
                    .primary_monitor()
                    .unwrap()
                    .expect("Failed to get primary monitor");
                let mouse_position = primary_monitor.position();
                let monitor_position = m.position();
                let monitor_size = m.size();
                mouse_position.x >= monitor_position.x &&
                    mouse_position.x < monitor_position.x + (monitor_size.width as i32) &&
                    mouse_position.y >= monitor_position.y &&
                    mouse_position.y < monitor_position.y + (monitor_size.height as i32)
            })
    {
        let monitor_size = monitor.size();
        let window_size = window.outer_size().unwrap();

        let x = ((monitor_size.width as i32) - (window_size.width as i32)) / 2;
        let y = ((monitor_size.height as i32) - (window_size.height as i32)) / 2;

        window
            .set_position(PhysicalPosition::new(monitor.position().x + x, monitor.position().y + y))
            .unwrap();
    }
}

pub fn get_app_info() -> (String, Option<String>) {
    let mut ctx = AppInfoContext::new(vec![]);
    ctx.refresh_apps().unwrap();
    match ctx.get_frontmost_application() {
        Ok(window) => {
            let name = window.name.clone();
            let icon = window
                .load_icon()
                .ok()
                .map(|i| {
                    let png = i.to_png().unwrap();
                    STANDARD.encode(png.get_bytes())
                });
            (name, icon)
        }
        Err(_) => ("System".to_string(), None),
    }
}

pub fn detect_color(color: &str) -> bool {
    let color = color.trim().to_lowercase();

    // hex
    if color.starts_with('#') && color.len() == color.trim_end_matches(char::is_whitespace).len() {
        let hex = &color[1..];
        return match hex.len() {
            3 | 6 | 8 => hex.chars().all(|c| c.is_ascii_hexdigit()),
            _ => false,
        };
    }

    // rgb/rgba
    if
        (color.starts_with("rgb(") || color.starts_with("rgba(")) &&
        color.ends_with(")") &&
        !color[..color.len() - 1].contains(")")
    {
        let values = color
            .trim_start_matches("rgba(")
            .trim_start_matches("rgb(")
            .trim_end_matches(')')
            .split(',')
            .collect::<Vec<&str>>();

        return match values.len() {
            3 | 4 => values.iter().all(|v| v.trim().parse::<f32>().is_ok()),
            _ => false,
        };
    }

    // hsl/hsla
    if
        (color.starts_with("hsl(") || color.starts_with("hsla(")) &&
        color.ends_with(")") &&
        !color[..color.len() - 1].contains(")")
    {
        let values = color
            .trim_start_matches("hsla(")
            .trim_start_matches("hsl(")
            .trim_end_matches(')')
            .split(',')
            .collect::<Vec<&str>>();

        return match values.len() {
            3 | 4 => values.iter().all(|v| v.trim().parse::<f32>().is_ok()),
            _ => false,
        };
    }

    false
}

#[tauri::command]
pub async fn fetch_page_meta(url: String) -> Result<(String, Option<String>), String> {
    let metadata = meta_fetcher
        ::fetch_metadata(&url)
        .map_err(|e| format!("Failed to fetch metadata: {}", e))?;

    Ok((metadata.title.unwrap_or_else(|| "No title found".to_string()), metadata.image))
}
