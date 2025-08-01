use crate::{
    DRAWER_OPEN, DRAWER_REFERENCE, LOGO,
    components::{ChunksComp, reference::ReferenceDrawer},
    route::Route,
    views::open_spotlight_window,
};
use dioxus::prelude::*;

#[component]
pub fn NavBar() -> Element {
    let cmd_ctrl = {
        #[cfg(target_os = "macos")]
        {
            "⌘"
        }
        #[cfg(not(target_os = "macos"))]
        {
            "Ctrl"
        }
    };
    let helper = move |_| {
        open_spotlight_window();
    };
    let (drawer_title, drawer_key) = if let Some(ref entry) = DRAWER_REFERENCE() {
        (
            entry.title.clone().unwrap_or_default(),
            entry.cite_key.clone(),
        )
    } else {
        (vec![], String::new())
    };
    rsx! {
        div {
            div { class: "navbar bg-base-100 shadow-sm",
                div { class: "navbar-start",
                    div { class: "tooltip tooltip-right",
                        div { class: "tooltip-content bg-base-100 text-red-400", "Home" }
                        Link { to: Route::Home {},
                            img { src: LOGO, width: "60px" }
                        }
                    }
                }

                div { class: "navbar-center", "" }

                div { class: "navbar-end p-2",
                    button { class: "btn btn-outline", onclick: helper,
                        "快捷助手"
                        kbd { class: "kbd", "{cmd_ctrl}" }
                        kbd { class: "kbd", "K" }
                    }
                }
            }
            div { class: "drawer drawer-end",
                input {
                    id: "global-drawer",
                    r#type: "checkbox",
                    class: "drawer-toggle",
                    checked: DRAWER_OPEN(),
                }
                div { class: "drawer-content", Outlet::<Route> {} }
                div { class: "drawer-side z-50",
                    label {
                        class: "drawer-overlay",
                        r#for: "global-drawer",
                        onclick: move |_| *DRAWER_OPEN.write() = false,
                    }
                    div { class: "min-h-full w-96 bg-base-200 p-4",
                        div { class: "flex justify-between items-center mb-4",
                            h3 { class: "text-grey-900 font-serif text-lg",
                                ChunksComp {
                                    chunks: drawer_title,
                                    cite_key: drawer_key,
                                }
                            }
                            button {
                                class: "btn btn-sm btn-circle btn-ghost",
                                onclick: move |_| *DRAWER_OPEN.write() = false,
                                "✕"
                            }
                        }
                        if let Some(entry) = DRAWER_REFERENCE() {
                            ReferenceDrawer { entry }
                        } else {
                            div { class: "text-center text-gray-500", "No reference selected" }
                        }
                    }
                }
            }
        }
    }
}
