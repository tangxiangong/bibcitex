//! Collections of components, views and tests.

use dioxus::prelude::*;

pub mod components;
pub mod route;
pub mod views;
pub static MAIN_CSS: Asset = asset!("/assets/styling/main.css");
pub static LOGO: Asset = asset!("/assets/transparent_logo.png");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<route::Route> {}
    }
}
