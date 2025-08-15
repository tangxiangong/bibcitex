#![cfg_attr(feature = "windows-bundle", windows_subsystem = "windows")]

use bibcitex_ui::{App, utils::observe_app};
use dioxus::desktop::WindowBuilder;
#[cfg(not(target_os = "macos"))]
use dioxus::desktop::tao::window::Icon;

#[cfg(not(target_os = "macos"))]
fn load_window_icon() -> Option<Icon> {
    if let Ok(icon_bytes) = std::fs::read("header.png")
        && let Ok(image) = image::load_from_memory(&icon_bytes)
    {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Icon::from_rgba(rgba.into_raw(), width, height).ok()
    } else {
        None
    }
}

fn main() {
    observe_app();

    let index_html = r#"
        <!doctype html>
        <html>
            <head>
                <title>BibCiTeX</title>
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"
                />
            </head>

            <body>
                <div id="main"></div>
            </body>
        </html>
"#.to_string();
    let window_builder = {
        #[cfg(not(target_os = "macos"))]
        {
            WindowBuilder::new()
                .with_title("BibCiTeX")
                .with_window_icon(load_window_icon())
        }
        #[cfg(target_os = "macos")]
        {
            WindowBuilder::new().with_title("BibCiTeX")
        }
    };

    let config = dioxus::desktop::Config::new()
        .with_custom_index(index_html)
        .with_window(window_builder);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}
