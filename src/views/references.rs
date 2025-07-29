use crate::{CURRENT_REF, components::Entry};
use bibcitex_core::{
    bib::Reference, search_references, search_references_by_author, search_references_by_journal,
    search_references_by_title, search_references_by_year,
};
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
enum FilterField {
    Author,
    Title,
    Journal,
    Year,
    All,
}

#[derive(Clone, Copy, PartialEq)]
enum FilterType {
    Book,
    Article,
    PhdThesis,
    All,
}

impl std::fmt::Display for FilterField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterField::Author => write!(f, "Author"),
            FilterField::Title => write!(f, "Title"),
            FilterField::Journal => write!(f, "Journal"),
            FilterField::Year => write!(f, "Year"),
            FilterField::All => write!(f, "All"),
        }
    }
}

impl std::fmt::Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterType::Book => write!(f, "Book"),
            FilterType::Article => write!(f, "Article"),
            FilterType::PhdThesis => write!(f, "PhD Thesis"),
            FilterType::All => write!(f, "All"),
        }
    }
}

#[component]
pub fn References() -> Element {
    let refs = Arc::new(CURRENT_REF().unwrap());
    let raw_refs = CURRENT_REF().unwrap();
    let mut query = use_signal(String::new);
    let mut is_input = use_signal(|| false);
    let mut search_result = use_signal(Vec::<Reference>::new);
    let mut filter_field = use_signal(|| FilterField::All);

    // TODO: Unimplemented
    let filter_type = use_signal(|| FilterType::All);

    use_effect(move || {
        if query().is_empty() {
            is_input.set(false);
        }
    });

    let refs_for_search = refs.clone();
    let search = move |e: Event<FormData>| {
        is_input.set(true);
        query.set(e.value());
        let current_filter = filter_field();
        let result = match current_filter {
            FilterField::Author => search_references_by_author(&refs_for_search, &query()),
            FilterField::Title => search_references_by_title(&refs_for_search, &query()),
            FilterField::Journal => search_references_by_journal(&refs_for_search, &query()),
            FilterField::Year => search_references_by_year(&refs_for_search, &query()),
            FilterField::All => search_references(&refs_for_search, &query()),
        };
        search_result.set(result);
    };

    let refs_for_filter = refs.clone();
    let on_filter_change = move |e: Event<FormData>| {
        let selected_value = e.value();
        let new_filter = match selected_value.as_str() {
            "Author" => FilterField::Author,
            "Title" => FilterField::Title,
            "Journal" => FilterField::Journal,
            "Year" => FilterField::Year,
            _ => FilterField::All,
        };
        filter_field.set(new_filter);

        // Re-run search if there's a query
        if !query().is_empty() {
            let result = match new_filter {
                FilterField::Author => search_references_by_author(&refs_for_filter, &query()),
                FilterField::Title => search_references_by_title(&refs_for_filter, &query()),
                FilterField::Journal => search_references_by_journal(&refs_for_filter, &query()),
                FilterField::Year => search_references_by_year(&refs_for_filter, &query()),
                FilterField::All => search_references(&refs_for_filter, &query()),
            };
            search_result.set(result);
        }
    };

    rsx! {
        div { class: "join p-4",
            select { class: "select join-item w-24", onchange: on_filter_change,
                option {
                    value: "{FilterField::All}",
                    selected: filter_field() == FilterField::All,
                    "{FilterField::All}"
                }
                option {
                    value: "{FilterField::Author}",
                    selected: filter_field() == FilterField::Author,
                    "{FilterField::Author}"
                }
                option {
                    value: "{FilterField::Journal}",
                    selected: filter_field() == FilterField::Journal,
                    "{FilterField::Journal}"
                }
                option {
                    value: "{FilterField::Title}",
                    selected: filter_field() == FilterField::Title,
                    "{FilterField::Title}"
                }
                option {
                    value: "{FilterField::Year}",
                    selected: filter_field() == FilterField::Year,
                    "{FilterField::Year}"
                }
            }
            // TODO: Add type filter, such as "book", "article", etc.
            select { class: "select join-item w-24",
                // onchange: on_type_change,
                option {
                    value: "{FilterType::All}",
                    selected: filter_type() == FilterType::All,
                    "{FilterType::All}"
                }
                option {
                    value: "{FilterType::Book}",
                    selected: filter_type() == FilterType::Book,
                    "{FilterType::Book}"
                }
                option {
                    value: "{FilterType::Article}",
                    selected: filter_type() == FilterType::Article,
                    "{FilterType::Article}"
                }
                option {
                    value: "{FilterType::PhdThesis}",
                    selected: filter_type() == FilterType::PhdThesis,
                    "{FilterType::PhdThesis}"
                }
            }
            div {
                div {
                    input {
                        r#type: "search",
                        class: "input input-primary join-item w-100",
                        value: "{query}",
                        oninput: search,
                    }
                }
            }
        }
        div {
            if !is_input() {
                h2 { class: "text-lg p-2", "References (total: {refs.len()})" }
                for reference in raw_refs {
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
