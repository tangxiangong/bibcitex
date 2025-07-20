//! Collections of components, views and tests.

use dioxus::prelude::*;

pub mod components;
pub mod route;
pub mod views;
pub const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
pub const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
pub const LOGO: Asset = asset!("/assets/transparent_logo.png");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Router::<route::Route> {}
    }
}
