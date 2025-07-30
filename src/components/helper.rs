use crate::{
    components::Entry,
    views::{HELPER_BIB, HELPER_WINDOW_OPEN, set_helper_bib},
};
use arboard::Clipboard;
use bibcitex_core::{
    bib::{Reference, parse},
    search_references,
    utils::read_bibliography,
};
use dioxus::{desktop::use_window, prelude::*};

#[component]
pub fn BibItem(name: String, path: String, updated_at: String) -> Element {
    let path_clone = path.clone();
    let name_clone = name.clone();
    let updated_at_clone = updated_at.clone();
    let mut selected_bib = use_context::<Signal<Option<(String, String, String)>>>();
    let mut error_message = use_context::<Signal<Option<String>>>();

    let handle_click = move |_| {
        error_message.set(None);
        match parse(&path_clone) {
            Ok(bib) => {
                let refs = read_bibliography(bib);
                selected_bib.set(Some((
                    name_clone.clone(),
                    path_clone.clone(),
                    updated_at_clone.clone(),
                )));
                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                    set_helper_bib(Some(refs));
                });
            }
            Err(e) => {
                error_message.set(Some(e.to_string()));
            }
        }
    };
    rsx! {
        div { class: "item", onclick: handle_click,
            h3 { {name} }
            p { "{path} ({updated_at})" }
        }
    }
}

#[component]
pub fn SelectBib(bibs: Memo<Vec<(String, String, String)>>) -> Element {
    let error_message = use_context_provider(|| Signal::new(None::<String>));
    let selected_bib = use_context_provider(|| Signal::new(None::<(String, String, String)>));
    rsx! {

        div {
            p { "请选择文献库" }
            for (name , path , updated_at) in bibs() {
                BibItem { name, path, updated_at }
            }
            if let Some(error) = error_message() {
                p { "{error}" }
            }
            if let Some(selected) = selected_bib() {
                p { "已选择: {selected.0}" }
            }
        }
    }
}

#[component]
pub fn SearchRef() -> Element {
    let mut query = use_context::<Signal<String>>();
    let mut result = use_signal(Vec::<Reference>::new);
    let current_bib = HELPER_BIB().unwrap();
    let search = move |e: Event<FormData>| {
        query.set(e.value());
        let res = search_references(&current_bib, &query());
        result.set(res);
    };
    let mut selected_index = use_signal(|| None::<usize>);
    let max_index = use_memo(move || {
        let len = result().len();
        if len > 0 { len - 1 } else { 0 }
    });
    let handle_keydown = move |evt: Event<KeyboardData>| {
        if !query().is_empty() {
            match evt.key() {
                Key::Enter => {
                    if let Some(index) = selected_index() {
                        let text = result()[index].cite_key.clone();
                        let window = use_window();
                        window.close();
                        HELPER_WINDOW_OPEN.write().take();
                        let mut clipboard = Clipboard::new().unwrap();
                        clipboard.set_text(text.to_string()).unwrap();
                        // spawn(async move {
                        //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        //     if let Err(e) = paste_to_active_app(&text) {
                        //         eprintln!("跨应用粘贴失败: {e}");
                        //     }
                        // });
                    }
                }
                Key::ArrowDown => {
                    if let Some(index) = selected_index() {
                        let update_index = (index + 1).min(max_index());
                        selected_index.set(Some(update_index));
                    } else {
                        selected_index.set(Some(0));
                    }
                }
                Key::ArrowUp => {
                    if let Some(index) = selected_index() {
                        let update_index = if index > 0 { index - 1 } else { 0 };
                        selected_index.set(Some(update_index));
                    }
                }
                _ => {}
            }
        }
    };

    // Pure Rust scrolling implementation
    let scroll_top = use_signal(|| 0.0);
    // Calculate real heights based on actual CSS:
    // .helper-item: padding: 8px (top) + 8px (bottom) = 16px
    // .helper-item: margin: 2px (top) + 2px (bottom) = 4px
    // .entry: margin: 10px (top) + 10px (bottom) = 20px
    // Estimated text content height: ~20px
    // Total: 16 + 4 + 20 + 20 + 20 = 80px per item
    let item_height = 80.0;

    use_effect(move || {
        if let Some(index) = selected_index() {
            // Simply scroll to the selected item position, no assumptions
            let target_scroll = index as f64 * item_height;
            // Use spawn to break the reactive cycle
            let mut scroll_top = scroll_top;
            spawn(async move {
                scroll_top.set(target_scroll);
            });
        }
    });

    rsx! {
        input {
            class: "helper-input",
            r#type: "text",
            placeholder: "搜索文献、作者、标题...",
            value: "{query}",
            oninput: search,
            onkeydown: handle_keydown,
            autofocus: true,
        }

        if !query().is_empty() {
            if result().is_empty() {
                div { class: "helper-results",
                    p { "没有找到结果" }
                }
            } else {
                div { class: "helper-results",
                    div {
                        class: "helper-no-results",
                        style: "transform: translateY(-{scroll_top()}px); transition: transform 0.2s ease;",
                        for (index , bib) in result().into_iter().enumerate() {
                            div {
                                id: format!("helper-item-{}", index),
                                class: if selected_index() == Some(index) { "helper-item-selected" } else { "helper-item" },
                                Entry { entry: bib }
                            }
                        }
                    }
                }
            }
        }
    }
}
