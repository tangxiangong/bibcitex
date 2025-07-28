use crate::{LOGO, route::Route};
use dioxus::prelude::*;

#[component]
pub fn NavBar() -> Element {
    rsx! {
        div {
            div { class: "navbar bg-base-100 shadow-sm",
                div { class: "navbar-start",
                    Link { to: Route::Home {},
                        img { src: LOGO, width: "50px" }
                    }
                }

                div { class: "navbar-center", "" }

                div { class: "navbar-end", "" }
            }
            Outlet::<Route> {}
        }
    }
}
