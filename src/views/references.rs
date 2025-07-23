use crate::{CURRENT_REF, components::Entry};
use bibcitex_core::{bib::Reference, search_references};
use dioxus::prelude::*;

#[component]
pub fn References() -> Element {
    let refs = CURRENT_REF().unwrap();
    let mut query = use_signal(String::new);
    let mut is_input = use_signal(|| false);
    let mut search_result = use_signal(Vec::<Reference>::new);

    use_effect(move || {
        if query().is_empty() {
            is_input.set(false);
        }
    });

    let search = move |e: Event<FormData>| {
        is_input.set(true);
        query.set(e.value());
        let result = search_references(&CURRENT_REF().unwrap(), &query());
        search_result.set(result);
    };

    rsx! {
        div {
            input { r#type: "text", value: "{query}", oninput: search }
            if !is_input() {
                h2 { "References (total: {refs.len()})" }
                for reference in refs {
                    Entry { entry: reference }
                }
            } else {
                h2 { "References ({search_result().len()}/{refs.len()})" }
                if !search_result().is_empty() {
                    for reference in search_result() {
                        Entry { entry: reference }
                    }
                } else {
                    p { "No results" }
                }
            }
        }
    }
}
