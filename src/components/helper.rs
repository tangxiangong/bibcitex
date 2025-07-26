use crate::{
    components::Entry,
    views::{HELPER_BIB, HELPER_WINDOW_OPEN, paste_to_active_app},
};
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
                let mut current_ref = HELPER_BIB.write();
                selected_bib.set(Some((
                    name_clone.clone(),
                    path_clone.clone(),
                    updated_at_clone.clone(),
                )));
                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                    *current_ref = Some(refs);
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
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/bib.css") }
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
pub fn SearchBib() -> Element {
    let mut query = use_context::<Signal<String>>();
    let mut result = use_signal(Vec::<Reference>::new);
    let current_bib = HELPER_BIB().unwrap();
    let search = move |e: Event<FormData>| {
        query.set(e.value());
        let res = search_references(&current_bib, &query());
        result.set(res);
    };
    rsx! {
        input {
            class: "helper-input",
            r#type: "text",
            placeholder: "搜索文献、作者、标题...",
            value: "{query}",
            oninput: search,
            onkeydown: move |evt| {
                if evt.key() == Key::Enter && !query().is_empty() {
                    let text = query().clone();
                    let window = use_window();
                    window.close();
                    HELPER_WINDOW_OPEN.write().take();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        if let Err(e) = paste_to_active_app(&text) {
                            eprintln!("跨应用粘贴失败: {e}");
                        }
                    });
                }
            },
            autofocus: true,
        }

        if !query().is_empty() {
            div { class: "helper-results",
                div { class: "helper-no-results",
                    for bib in result() {
                        Entry { entry: bib }
                    }
                }
            }
        }
    }
}
