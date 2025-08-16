#![cfg_attr(feature = "windows-bundle", windows_subsystem = "windows")]

use bibcitex_ui::{App, utils::observe_app};
#[cfg(not(target_os = "macos"))]
use dioxus::desktop::tao::window::Icon;
#[cfg(target_os = "macos")]
use dioxus::desktop::tao::{
    event::{Event, WindowEvent},
    platform::macos::{ActivationPolicy, EventLoopWindowTargetExtMacOS},
};
use dioxus::{
    LaunchBuilder,
    desktop::{Config, WindowBuilder},
};

#[cfg(not(target_os = "macos"))]
static WINDOW_ICON: &[u8] = include_bytes!("../icons/windowicon.png");

#[cfg(not(target_os = "macos"))]
fn load_window_icon() -> Option<Icon> {
    if let Ok(image) = image::load_from_memory(WINDOW_ICON) {
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

    let config = Config::new()
        .with_custom_index(index_html)
        .with_window(window_builder)
        .with_data_directory(dirs::config_dir().unwrap().join("BibCiTeX"))
        .with_custom_event_handler(
            #[allow(unused_variables)]
            |event, event_loop_window_target| {
                #[cfg(target_os = "macos")]
                {
                    match event {
                        Event::WindowEvent {
                            event: WindowEvent::CloseRequested,
                            ..
                        } => {
                            event_loop_window_target
                                .set_activation_policy_at_runtime(ActivationPolicy::Accessory);
                        }
                        Event::WindowEvent {
                            event: WindowEvent::Focused(true),
                            ..
                        } => {
                            event_loop_window_target
                                .set_activation_policy_at_runtime(ActivationPolicy::Regular);
                        }
                        _ => {}
                    }
                }
            },
        );
    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
