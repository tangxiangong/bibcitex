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
                    class: "flex-1 overflow-y-auto",
                    style: {
                        let base_height = MAX_HEIGHT - MIN_HEIGHT;
                        let error_height = if error_message().is_some() { 60 } else { 0 };
                        format!("scroll-behavior: smooth; max-height: {}px;", base_height - error_height)
                    },
                    for (i , bib_info) in bibs().iter().enumerate() {
                        div {
                            key: "{i}",
                            "data-item-index": "{i}",
                            class: if Some(i) == bib_selected_index() { "shrink-0 px-5 py-3 bg-base-200 cursor-pointer rounded-lg" } else { "shrink-0 px-5 py-3 hover:bg-base-200 hover:rounded-lg cursor-pointer" },
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
                            div { class: "text-base font-medium text-base-content mb-1",
                                if bib_info.4 {
                                    div { class: "inline-grid *:[grid-area:1/1]",
                                        div { class: "status status-success animate-ping" }
                                        div { class: "status status-success" }
                                    }
                                } else {
                                    div { class: "inline-grid *:[grid-area:1/1]",
                                        div { class: "status status-error animate-ping" }
                                        div { class: "status status-error" }
                                    }
                                }
                                span { class: "ml-1", "{bib_info.0}" }
                                if let Some(ref desc) = bib_info.3 {
                                    span { class: "text-sm ml-2 text-base-content/70",
                                        "{desc}"
                                    }
                                }
                            }
                            div { class: "text-sm text-base-content/70 flex justify-between",
                                span { "{bib_info.1}" }
                                span { "{bib_info.2}" }
                            }
                        }
                    }
                }
                if let Some(ref error_msg) = error_message() {
                    div { class: "shrink-0 px-5 py-3 text-red-600 text-sm bg-red-50 border-t border-red-200 font-medium rounded-b-xl",
                        "{error_msg}"
                    }
                }
            }
        }
    }
}
