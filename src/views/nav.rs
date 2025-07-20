use crate::{LOGO, components::AddItem, route::Route};
use dioxus::prelude::*;
use std::{collections::BTreeMap, path::PathBuf};

static NAV_CSS: Asset = asset!("/assets/styling/nav.css");

#[component]
pub fn NavBar() -> Element {
    let mut show_modal = use_signal(|| false);
    let db = use_signal(BTreeMap::<String, PathBuf>::new);

    let open_modal = move |_| {
        show_modal.set(true);
    };

    rsx! {
        document::Link { rel: "stylesheet", href: NAV_CSS }
        div {
            div { id: "navbar-link",
                Link { to: Route::Home {},
                    img { src: LOGO, width: "100px" }
                }
                button { onclick: open_modal, "添加文献库" }
            }
            Outlet::<Route> {}

            if show_modal() {
                AddItem { db, show: show_modal }
            }
        }
    }
}
