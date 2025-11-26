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
            "Win"
        }
    };
    let helper = move |_| {
        spawn(async move {
            open_spotlight_window().await;
        });
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
        div { class: "h-screen flex flex-col bg-base-100",
            div { class: "navbar bg-base-100/80 backdrop-blur-md border-b border-base-content/5 shrink-0 z-40 sticky top-0",
                div { class: "navbar-start pl-4",
                    Link {
                        to: Route::Home {},
                        class: "flex items-center gap-3 hover:opacity-80 transition-opacity",
                        div { class: "w-10 h-10 relative",
                            img {
                                src: LOGO,
                                class: "w-full h-full object-contain",
                            }
                        }
                        span { class: "font-bold text-xl tracking-tight gradient-text hidden sm:block",
                            "BibCiTeX"
                        }
                    }
                }

                div { class: "navbar-center", "" }

                div { class: "navbar-end pr-4",
                    button {
                        class: "btn btn-ghost btn-sm gap-2 hover:bg-base-content/5 font-normal text-base-content/70",
                        onclick: helper,
                        span { "快捷助手" }
                        div { class: "hidden md:flex gap-1",
                            kbd { class: "kbd kbd-sm font-mono bg-base-200 border-base-300",
                                "{cmd_ctrl}"
                            }
                            kbd { class: "kbd kbd-sm font-mono bg-base-200 border-base-300",
                                "shift"
                            }
                            kbd { class: "kbd kbd-sm font-mono bg-base-200 border-base-300",
                                "k"
                            }
                        }
                    }
                }
            }
            div { class: "drawer drawer-end flex-1 overflow-hidden relative",
                input {
                    id: "global-drawer",
                    r#type: "checkbox",
                    class: "drawer-toggle",
                    checked: DRAWER_OPEN(),
                }
                div { class: "drawer-content h-full overflow-hidden bg-base-200/30",
                    Outlet::<Route> {}
                }
                div { class: "drawer-side z-50",
                    label {
                        class: "drawer-overlay backdrop-blur-sm",
                        r#for: "global-drawer",
                        onclick: move |_| *DRAWER_OPEN.write() = false,
                    }
                    div { class: "min-h-full w-[480px] max-w-[90vw] bg-base-100 shadow-2xl p-0 flex flex-col border-l border-base-content/5",
                        div { class: "p-4 border-b border-base-content/5 flex justify-between items-start bg-base-100/95 backdrop-blur sticky top-0 z-10",
                            div { class: "flex-1 pr-4",
                                h3 { class: "text-lg font-bold leading-tight",
                                    ChunksComp {
                                        chunks: drawer_title,
                                        cite_key: drawer_key,
                                    }
                                }
                            }
                            button {
                                class: "btn btn-sm btn-circle btn-ghost",
                                onclick: move |_| *DRAWER_OPEN.write() = false,
                                "✕"
                            }
                        }
                        div { class: "flex-1 overflow-y-auto p-4",
                            if let Some(entry) = DRAWER_REFERENCE() {
                                ReferenceDrawer { entry }
                            } else {
                                div { class: "flex flex-col items-center justify-center h-full text-base-content/50",
                                    span { class: "text-4xl mb-2", "📄" }
                                    span { "尚未选择任何参考文献" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
