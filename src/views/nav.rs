use crate::{LOGO, route::Route};
use dioxus::prelude::*;

static NAV_CSS: Asset = asset!("/assets/styling/nav.css");

#[component]
pub fn NavBar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: NAV_CSS }
        div {
            div { id: "navbar-link",
                Link { to: Route::Home {},
                    img { src: LOGO, width: "100px" }
                }
            }
            Outlet::<Route> {}
        }
    }
}
