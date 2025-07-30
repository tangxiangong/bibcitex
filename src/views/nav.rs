use crate::{LOGO, route::Route, views::open_spotlight_window};
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
                    button { class: "btn btn-secondary", onclick: helper,
                        "快捷助手"
                        kbd { class: "kbd", "{cmd_ctrl}" }
                        kbd { class: "kbd", "K" }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}
