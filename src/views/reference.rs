use crate::{
    CURRENT_REF,
    components::{FilterFieldSelector, FilterTypeSelector, ReferenceComponent},
};
use bibcitex_core::{
    bib::Reference, filter::*, search_references, search_references_by_author,
    search_references_by_journal, search_references_by_title, search_references_by_year,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum FilterField {
    Author,
    Title,
    Journal,
    Year,
    All,
}

#[derive(Clone, Copy, PartialEq)]
pub enum FilterType {
    Book,
    Article,
    Thesis,
    TechReport,
    Misc,
    Booklet,
    InBook,
    InCollection,
    InProceedings,
    All,
}

impl std::fmt::Display for FilterField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterField::Author => write!(f, "Author"),
            FilterField::Title => write!(f, "Title"),
            FilterField::Journal => write!(f, "Journal"),
            FilterField::Year => write!(f, "Year"),
            FilterField::All => write!(f, "Field"),
        }
    }
}

impl std::fmt::Display for FilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterType::Book => write!(f, "Book"),
            FilterType::Article => write!(f, "Article"),
            FilterType::Thesis => write!(f, "Thesis"),
            FilterType::TechReport => write!(f, "TechReport"),
            FilterType::Misc => write!(f, "Misc"),
            FilterType::Booklet => write!(f, "Booklet"),
            FilterType::InBook => write!(f, "InBook"),
            FilterType::InCollection => write!(f, "InCollection"),
            FilterType::InProceedings => write!(f, "InProceedings"),
            FilterType::All => write!(f, "Type"),
        }
    }
}

#[component]
pub fn References() -> Element {
    let total_num = CURRENT_REF().unwrap_or_default().len();
    let mut query = use_context_provider(|| Signal::new(String::new()));
    let mut is_input = use_signal(|| false);
    let mut search_result = use_context_provider(|| Signal::new(Vec::<Reference>::new()));
    let filter_field = use_context_provider(|| Signal::new(FilterField::All));
    let filter_type = use_context_provider(|| Signal::new(FilterType::All));
    let refs = use_memo(move || {
        let total_refs = CURRENT_REF().unwrap_or_default();
        match filter_type() {
            FilterType::Book => filter_book(total_refs),
            FilterType::Article => filter_article(total_refs),
            FilterType::Thesis => filter_thesis(total_refs),
            FilterType::TechReport => filter_techreport(total_refs),
            FilterType::Misc => filter_misc(total_refs),
            FilterType::Booklet => filter_booklet(total_refs),
            FilterType::InBook => filter_inbook(total_refs),
            FilterType::InCollection => filter_incollection(total_refs),
            FilterType::InProceedings => filter_inproceedings(total_refs),
            FilterType::All => total_refs,
        }
    });
    let show_type = use_memo(move || match filter_type() {
        FilterType::All => "References".to_string(),
        FilterType::Article => "Articles".to_string(),
        FilterType::Book => "Books".to_string(),
        FilterType::Thesis => "Thesis".to_string(),
        FilterType::TechReport => "TechReports".to_string(),
        FilterType::Misc => "Misc".to_string(),
        FilterType::Booklet => "Booklets".to_string(),
        FilterType::InBook => "InBooks".to_string(),
        FilterType::InCollection => "InCollections".to_string(),
        FilterType::InProceedings => "InProceedings".to_string(),
    });

    use_effect(move || {
        if query().is_empty() {
            is_input.set(false);
        }
    });

    let search = move |e: Event<FormData>| {
        is_input.set(true);
        query.set(e.value());
        let result = match filter_field() {
            FilterField::Author => search_references_by_author(&refs(), &query()),
            FilterField::Title => search_references_by_title(&refs(), &query()),
            FilterField::Journal => search_references_by_journal(&refs(), &query()),
            FilterField::Year => search_references_by_year(&refs(), &query()),
            FilterField::All => search_references(&refs(), &query()),
        };
        search_result.set(result);
    };

    rsx! {
        div { class: "flex flex-col h-full overflow-hidden",
            // Fixed search bar at top
            div { class: "flex-shrink-0 p-4 bg-base-100 border-b border-base-300 overflow-hidden",
                div { class: "join w-full max-w-full overflow-hidden",
                    FilterTypeSelector { refs }
                    FilterFieldSelector { refs }
                    input {
                        r#type: "search",
                        class: "input input-primary join-item flex-1 min-w-0",
                        placeholder: "搜索文献...",
                        value: "{query}",
                        oninput: search,
                    }
                }
            }

            // Scrollable content area
            div { class: "flex-1 overflow-y-auto overflow-x-hidden",
                if !is_input() {
                    h2 { class: "text-lg p-2", "{show_type()} ({refs().len()}/{total_num})" }
                    for reference in refs() {
                        ReferenceComponent { entry: reference }
                    }
                } else {
                    h2 { class: "text-lg p-2",
                        "{show_type()} ({search_result().len()}/{refs().len()})"
                    }
                    if !search_result().is_empty() {
                        for reference in search_result() {
                            ReferenceComponent { entry: reference }
                        }
                    } else {
                        p { class: "p-2 text-lg text-red-500", "No results" }
                    }
                }
            }
        }
    }
}
