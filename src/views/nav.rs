use crate::{LOGO, components::AddItem, route::Route};
use dioxus::prelude::*;
use std::{collections::BTreeMap, path::PathBuf};

#[component]
pub fn NavBar() -> Element {
    let mut show_modal = use_signal(|| false);
    let db = use_signal(BTreeMap::<String, PathBuf>::new);

    let open_modal = move |_| {
        show_modal.set(true);
    };

    rsx! {
        div {
            div { style: "display: flex; align-items: center; justify-content: space-between;",
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
