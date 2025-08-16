//! Collections of components, views and tests.

use crate::views::open_spotlight_window;
use bibcitex_core::{Setting, bib::Reference};
use dioxus::{
    desktop::{
        HotKeyState, WindowCloseBehaviour,
        tao::keyboard::ModifiersState,
        trayicon::{DioxusTrayIcon, default_tray_icon, init_tray_icon},
        use_global_shortcut, window,
    },
    prelude::*,
};

pub mod components;
pub mod platforms;
pub mod route;
pub mod utils;
pub mod views;

// icon assets
pub static LOGO: Asset = asset!("assets/transparent_logo.png");
pub static ERR_ICON: Asset = asset!("assets/icons/error.svg");
pub static OK_ICON: Asset = asset!("assets/icons/ok.svg");
pub static COPY_ICON: Asset = asset!("assets/icons/copy.svg");
pub static ADD_ICON: Asset = asset!("assets/icons/add.svg");
pub static CANCEL_ICON: Asset = asset!("assets/icons/cancel.svg");
pub static DELETE_ICON: Asset = asset!("assets/icons/delete.svg");
pub static DETAILS_ICON: Asset = asset!("assets/icons/detail.svg");

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
        init_tray_icon(default_tray_icon(), tray_icon)
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

    rsx! {
        document::Stylesheet { href: TAILWINDCSS }
        Router::<route::Route> {}
    }
}
