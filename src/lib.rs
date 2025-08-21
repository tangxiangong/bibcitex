//! Collections of components, views and tests.

use crate::views::{UpdateWindow, open_spotlight_window};
use bibcitex_core::{Setting, bib::Reference};
use dioxus::{
    desktop::{
        HotKeyState, WindowCloseBehaviour,
        tao::keyboard::ModifiersState,
        trayicon::{DioxusTrayIcon, DioxusTrayMenu, init_tray_icon, menu},
        use_global_shortcut, use_muda_event_handler, use_tray_menu_event_handler, window,
    },
    prelude::*,
};

pub mod components;
pub mod route;
pub mod views;

// icon assets
pub static LOGO: Asset = asset!("assets/transparent_logo.png");
pub static ERR_ICON: Asset = asset!("assets/icons/error.svg");
pub static OK_ICON: Asset = asset!("assets/icons/ok.svg");
pub static COPY_ICON: Asset = asset!("assets/icons/copy.svg");
pub static ADD_ICON: Asset = asset!("assets/icons/add.svg");
pub static CANCEL_ICON: Asset = asset!("assets/icons/cancel.svg");
pub static DELETE_ICON: Asset = asset!("assets/icons/delete.svg");
pub static DETAILS_ICON: Asset = asset!("assets/icons/details.svg");

static TRAY_ICON: &[u8] = include_bytes!("../icons/trayicon.png");

/// global state
pub static STATE: GlobalSignal<Setting> = Signal::global(Setting::load);
pub static CURRENT_REF: GlobalSignal<Option<Vec<Reference>>> = Signal::global(|| None);
pub static DRAWER_OPEN: GlobalSignal<bool> = Signal::global(|| false);
pub static DRAWER_REFERENCE: GlobalSignal<Option<Reference>> = Signal::global(|| None);

// tailwindcss
pub static TAILWINDCSS: Asset = asset!("assets/tailwind.css");

#[component]
pub fn App() -> Element {
    use_hook(|| {
        window().set_close_behavior(WindowCloseBehaviour::WindowHides);
        let tray_icon = if let Ok(image) = image::load_from_memory(TRAY_ICON) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            DioxusTrayIcon::from_rgba(rgba.into_raw(), width, height).ok()
        } else {
            None
        };
        let tray_menu = DioxusTrayMenu::new();
        let helper_item = menu::IconMenuItemBuilder::new()
            .id("helper".into())
            .text("快捷助手(暂不可用)")
            .build();
        tray_menu
            .append_items(&[&helper_item, &menu::PredefinedMenuItem::quit(None)])
            .unwrap();
        // FIXME: BUG: set_show_menu_on_left_click(true) always raises main window
        // https://github.com/DioxusLabs/dioxus/issues/4430
        init_tray_icon(tray_menu, tray_icon);
    });

    // FIXME: BUG: tray menu event handler is not working
    // https://github.com/DioxusLabs/dioxus/issues/4495
    use_tray_menu_event_handler(move |menu_event| {
        if menu_event.id() == "helper" {
            spawn(async move {
                open_spotlight_window().await;
            });
        }
    });

    // TODO: Error handling
    let _ = use_global_shortcut(
        (ModifiersState::SUPER | ModifiersState::SHIFT, KeyCode::K),
        move |hotkey_state: HotKeyState| {
            if hotkey_state == HotKeyState::Pressed {
                spawn(async move {
                    open_spotlight_window().await;
                });
            }
        },
    );

    let mut show_check_window = use_signal(|| false);

    use_muda_event_handler(move |muda_event| {
        if muda_event.id() == "helper" {
            spawn(async move {
                open_spotlight_window().await;
            });
        } else if muda_event.id() == "check_update" {
            show_check_window.set(true);
        }
    });

    rsx! {
        document::Stylesheet { href: TAILWINDCSS }
        Router::<route::Route> {}
        if show_check_window() {
            div { class: "modal modal-open",
                div { class: "modal-box w-1/2",
                    UpdateWindow { show_window: show_check_window }
                }
                div {
                    class: "modal-backdrop",
                    onclick: move |_| show_check_window.set(false),
                }
            }
        }
    }
}
