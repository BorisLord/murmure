use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Listener, Manager};

const TRAY_ID: &str = "main";

fn desaturate_rgba(rgba: &[u8]) -> Vec<u8> {
    let mut out = rgba.to_vec();
    for chunk in out.chunks_exact_mut(4) {
        let lum = (0.299 * chunk[0] as f32
            + 0.587 * chunk[1] as f32
            + 0.114 * chunk[2] as f32) as u8;
        chunk[0] = lum;
        chunk[1] = lum;
        chunk[2] = lum;
    }
    out
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", "Open Murmure", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    let builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });

    #[cfg(target_os = "linux")]
    let builder = builder
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(true)
        .icon_as_template(true);
    #[cfg(target_os = "windows")]
    let builder = builder.icon(app.default_window_icon().unwrap().clone());
    // On macOS, use a dedicated monochrome template icon so the menu bar
    // renders it as a template (adapts to Light/Dark mode and matte/full
    // menu bar styles), matching Apple HIG for status items.
    #[cfg(target_os = "macos")]
    let builder = {
        let tray_icon_bytes = include_bytes!("../../icons/tray-template.png");
        let tray_icon = tauri::image::Image::from_bytes(tray_icon_bytes)?;
        builder.icon(tray_icon).icon_as_template(true)
    };

    let _tray = builder.build(app)?;

    // On Linux/Windows the tray uses the colourful default window icon, so
    // desaturate it when the model is idle-unloaded. macOS uses a monochrome
    // template icon the OS already renders consistently — no swap needed.
    #[cfg(not(target_os = "macos"))]
    {
        let default_img = app.default_window_icon().unwrap();
        let width = default_img.width();
        let height = default_img.height();
        let normal_rgba: Vec<u8> = default_img.rgba().to_vec();
        let grey_rgba = desaturate_rgba(&normal_rgba);

        let app_handle = app.clone();
        app.listen("model-state-changed", move |event| {
            let Some(tray) = app_handle.tray_by_id(TRAY_ID) else {
                return;
            };
            let rgba: &[u8] = if event.payload().contains("unloaded") {
                &grey_rgba
            } else {
                &normal_rgba
            };
            let _ = tray.set_icon(Some(Image::new(rgba, width, height)));
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desaturate_applies_luminance_and_preserves_alpha() {
        // Two RGBA pixels: pure red, pure green. Alpha 255 and 128.
        let input: &[u8] = &[255, 0, 0, 255, 0, 255, 0, 128];
        let out = desaturate_rgba(input);

        // Red luminance: 0.299 * 255 ≈ 76
        assert_eq!(out[0], 76);
        assert_eq!(out[1], 76);
        assert_eq!(out[2], 76);
        assert_eq!(out[3], 255, "alpha must be untouched");

        // Green luminance: 0.587 * 255 ≈ 149
        assert_eq!(out[4], 149);
        assert_eq!(out[5], 149);
        assert_eq!(out[6], 149);
        assert_eq!(out[7], 128, "alpha must be untouched");
    }
}
