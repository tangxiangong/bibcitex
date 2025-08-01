//! Collections of components, views and tests.

use crate::views::open_spotlight_window;
use bibcitex_core::{Setting, bib::Reference};
use dioxus::{desktop::use_global_shortcut, prelude::*};
use std::{
    sync::atomic::{AtomicI64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
pub mod components;
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
static TAILWINDCSS: Asset = asset!("/assets/tailwind.css");

// Debounce mechanism to prevent double triggering
static LAST_SHORTCUT_TIME: AtomicI64 = AtomicI64::new(0);
const DEBOUNCE_MS: u64 = 200; // 200ms debounce

#[component]
pub fn App() -> Element {
    // TODO: Error handling
    let _ = use_global_shortcut("cmd+k", move || {
        // FIX BUG NOT MERGED IN DIOXUS V0.6
        // https://github.com/DioxusLabs/dioxus/pull/3822
        // 在 v0.7 版本不需要了
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        // Get last trigger time
        let last_time = LAST_SHORTCUT_TIME.load(Ordering::Relaxed);

        // Only trigger if enough time has passed (debounce)
        if now - last_time > DEBOUNCE_MS as i64 {
            LAST_SHORTCUT_TIME.store(now, Ordering::Relaxed);
            open_spotlight_window();
        }
    });

    rsx! {
        document::Stylesheet { href: TAILWINDCSS }
        Router::<route::Route> {}
    }
}
