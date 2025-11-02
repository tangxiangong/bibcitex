#![cfg_attr(feature = "windows-bundle", windows_subsystem = "windows")]

use bibcitex_ui::App;
#[cfg(not(target_os = "macos"))]
use dioxus::desktop::tao::window::Icon as TaoIcon;
#[cfg(target_os = "macos")]
use dioxus::desktop::tao::{
    event::{Event, WindowEvent},
    platform::macos::{ActivationPolicy, EventLoopWindowTargetExtMacOS},
};
use dioxus::{
    LaunchBuilder,
    desktop::{Config, WindowBuilder, muda::*},
};
use xpaste::observe_app;

static ABOUT_ICON: &[u8] = include_bytes!("../assets/transparent_logo.png");

#[cfg(not(target_os = "macos"))]
static WINDOW_ICON: &[u8] = include_bytes!("../icons/windowicon.png");

#[cfg(not(target_os = "macos"))]
fn load_window_icon() -> Option<TaoIcon> {
    if let Ok(image) = image::load_from_memory(WINDOW_ICON) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        TaoIcon::from_rgba(rgba.into_raw(), width, height).ok()
    } else {
        None
    }
}

fn main() {
    observe_app();

    // Custom HTML
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

    // Custom MENU
    let menu = Menu::new();
    let home_menu = Submenu::new("Home", true);

    let about_icon = if let Ok(image) = image::load_from_memory(ABOUT_ICON) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Icon::from_rgba(rgba.into_raw(), width, height).ok()
    } else {
        None
    };

    let mut about_metadata = from_cargo_metadata!();
    about_metadata.icon = about_icon;
    about_metadata.name = Some("BibCiTeX - BibTeX 快捷引用工具".to_string());
    about_metadata.copyright = Some("Copyright 2025 -- present tangxiangong".to_string());

    home_menu
        .append_items(&[
            &PredefinedMenuItem::about(Some("About BibCiTeX"), Some(about_metadata)),
            &MenuItem::with_id("helper", "快捷助手", true, None),
            &MenuItem::with_id("check_update", "检查更新", true, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::hide(Some("Hide BibCiTeX")),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::close_window(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(Some("Quit BibCiTeX")),
        ])
        .unwrap();
    menu.append_items(&[&home_menu]).unwrap();

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
        )
        .with_menu(menu);
    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
