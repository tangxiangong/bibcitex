//! Collections of components, views and tests.

use dioxus::prelude::*;

pub(crate) mod components;
#[cfg(feature = "test-ui")]
pub(crate) mod tests;

pub(crate) const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
pub(crate) const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
pub(crate) const LOGO: Asset = asset!("/assets/transparent_logo.png");

#[cfg(feature = "build-ui")]
#[component]
pub(crate) fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        img { src: LOGO, width: "100px" }
    }
}

pub fn run_app() {
    #[cfg(feature = "build-ui")]
    dioxus::launch(App);
    #[cfg(feature = "test-ui")]
    dioxus::launch(tests::TestApp);
}
