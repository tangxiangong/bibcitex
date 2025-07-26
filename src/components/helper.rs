use bibcitex_core::{bib::parse, utils::read_bibliography};
use dioxus::prelude::*;

use crate::views::HELPER_BIB;

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
                *current_ref = Some(refs);
                selected_bib.set(Some((
                    name_clone.clone(),
                    path_clone.clone(),
                    updated_at_clone.clone(),
                )));
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
