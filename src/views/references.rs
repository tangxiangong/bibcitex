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
        div { class: "join p-4",
            select { class: "select join-item w-24",
                option { disabled: true, selected: true, "Filter" }
                option { "Author" }
                option { "Title" }
                option { "Journal" }
            }
            div {
                div {
                    input {
                        r#type: "search",
                        class: "input input-primary  join-item w-100",
                        value: "{query}",
                        oninput: search,
                    }
                }
            }
        }
        div {
            if !is_input() {
                h2 { class: "text-lg p-2", "References (total: {refs.len()})" }
                for reference in refs {
                    Entry { entry: reference }
                }
            } else {
                h2 { class: "text-lg p-2", "References ({search_result().len()}/{refs.len()})" }
                if !search_result().is_empty() {
                    for reference in search_result() {
                        Entry { entry: reference }
                    }
                } else {
                    p { class: "p-2 text-lg text-red-500", "No results" }
                }
            }
        }
    }
}
