//! Collections of components, views and tests.

use crate::views::open_spotlight_window;
use bibcitex_core::{Setting, bib::Reference};
use dioxus::{
    desktop::{HotKeyState, tao::keyboard::ModifiersState, use_global_shortcut},
    prelude::*,
};

pub mod components;
pub mod platforms;
pub mod route;
pub mod views;

// icon assets
pub static LOGO: Asset = asset!("/assets/transparent_logo.png");
pub static ERR_ICON: Asset = asset!("/assets/icons/error.svg");
pub static OK_ICON: Asset = asset!("/assets/icons/ok.svg");
pub static COPY_ICON: Asset = asset!("/assets/icons/copy.svg");
pub static ADD_ICON: Asset = asset!("/assets/icons/add.svg");
pub static CANCEL_ICON: Asset = asset!("/assets/icons/cancel.svg");
pub static DELETE_ICON: Asset = asset!("/assets/icons/delete.svg");
pub static DETAILS_ICON: Asset = asset!("/assets/icons/detail.svg");

/// global state
pub static STATE: GlobalSignal<Setting> = Signal::global(Setting::load);
pub static CURRENT_REF: GlobalSignal<Option<Vec<Reference>>> = Signal::global(|| None);
pub static DRAWER_OPEN: GlobalSignal<bool> = Signal::global(|| false);
pub static DRAWER_REFERENCE: GlobalSignal<Option<Reference>> = Signal::global(|| None);

// tailwindcss
pub static TAILWINDCSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    // TODO: Error handling
    let _ = use_global_shortcut(
        (ModifiersState::SUPER, KeyCode::K),
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
