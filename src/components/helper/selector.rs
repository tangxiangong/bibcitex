use crate::views::{MAX_HEIGHT, MIN_HEIGHT};
use dioxus::prelude::*;

/// 文献库选择组件
#[component]
pub fn BibliographySelector(
    #[allow(clippy::type_complexity)] bibs: Memo<
        Vec<(String, String, String, Option<String>, bool)>,
    >,
    bib_selected_index: Signal<Option<usize>>,
    on_bib_click: EventHandler<(String, String)>,
    error_message: Signal<Option<String>>,
) -> Element {
    let bib_item_elements = use_signal(std::collections::HashMap::<usize, MountedEvent>::new);
    // 当选中索引变化时，滚动到对应项
    use_effect(move || {
        if let Some(index) = bib_selected_index() {
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let elements = bib_item_elements.read();
                if let Some(element) = elements.get(&index) {
                    let _ = element
                        .scroll_to(dioxus::html::ScrollBehavior::Smooth)
                        .await;
                }
            });
        }
    });

    if bibs().is_empty() {
        rsx! {
            div { class: "shrink-0 px-5 py-10 text-center text-base-content/60 text-sm",
                "未找到文献库，请先在主页添加文献库"
            }
        }
    } else {
        rsx! {
            div { class: "flex flex-col h-full",
                div {
                    class: "flex-1 overflow-y-auto p-2 space-y-2",
                    style: {
                        let base_height = MAX_HEIGHT - MIN_HEIGHT;
                        let error_height = if error_message().is_some() { 60 } else { 0 };
                        format!("scroll-behavior: smooth; max-height: {}px;", base_height - error_height)
                    },
                    for (i , bib_info) in bibs().iter().enumerate() {
                        div {
                            key: "{i}",
                            "data-item-index": "{i}",
                            class: if Some(i) == bib_selected_index() { "rounded-lg bg-primary/10 text-primary shadow-sm mx-2 cursor-pointer transition-all duration-200" } else { "rounded-lg hover:bg-base-200/50 hover:shadow-sm mx-2 cursor-pointer transition-all duration-200 border border-transparent" },
                            onmounted: {
                                let mut bib_item_elements = bib_item_elements;
                                move |event| {
                                    bib_item_elements.write().insert(i, event);
                                }
                            },
                            onclick: {
                                let bib_path = bib_info.1.clone();
                                let bib_name = bib_info.0.clone();
                                move |_| on_bib_click.call((bib_name.clone(), bib_path.clone()))
                            },
                            div { class: "p-3",
                                div { class: "flex items-center justify-between",
                                    div { class: "flex items-center gap-2",
                                        if bib_info.4 {
                                            div { class: "badge badge-success badge-xs gap-1 border-none",
                                                div { class: "w-1.5 h-1.5 rounded-full bg-white animate-pulse" }
                                                "Ready"
                                            }
                                        } else {
                                            div { class: "badge badge-error badge-xs gap-1 border-none",
                                                div { class: "w-1.5 h-1.5 rounded-full bg-white" }
                                                "Error"
                                            }
                                        }
                                        h3 { class: "font-bold text-base", "{bib_info.0}" }
                                    }
                                    span { class: "text-xs opacity-60 font-mono", "{bib_info.2}" }
                                }
                                if let Some(ref desc) = bib_info.3 {
                                    p { class: "text-sm opacity-80 mt-1", "{desc}" }
                                }
                                p { class: "text-xs opacity-50 truncate mt-1 font-mono",
                                    "{bib_info.1}"
                                }
                            }
                        }
                    }
                }
                if let Some(ref error_msg) = error_message() {
                    div { class: "alert alert-error shadow-lg m-2",
                        span { "{error_msg}" }
                    }
                }
            }
        }
    }
}
