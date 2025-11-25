use crate::views::{FilterField, FilterType};
use bibcitex_core::{
    bib::Reference, search_references, search_references_by_author, search_references_by_journal,
    search_references_by_title, search_references_by_year,
};
use dioxus::prelude::*;

#[component]
pub fn FilterFieldSelector(refs: Memo<Vec<Reference>>) -> Element {
    let filter_type = use_context::<Signal<FilterType>>();
    let mut filter_field = use_context::<Signal<FilterField>>();
    let query = use_context::<Signal<String>>();
    let mut search_result = use_context::<Signal<Vec<Reference>>>();
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
                FilterField::Author => search_references_by_author(&refs(), &query()),
                FilterField::Title => search_references_by_title(&refs(), &query()),
                FilterField::Journal => search_references_by_journal(&refs(), &query()),
                FilterField::Year => search_references_by_year(&refs(), &query()),
                FilterField::All => search_references(&refs(), &query()),
            };
            search_result.set(result);
        }
    };
    rsx! {
        match filter_type() {
            FilterType::All | FilterType::Article => rsx! {
                select { class: "select join-item w-24 shrink-0", onchange: on_filter_change,
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
            },
            _ => rsx! {
                select { class: "select join-item w-24 shrink-0", onchange: on_filter_change,
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
            },
        }
    }
}

#[component]
pub fn FilterTypeSelector(refs: Memo<Vec<Reference>>) -> Element {
    let mut filter_type = use_context::<Signal<FilterType>>();
    let filter_field = use_context::<Signal<FilterField>>();
    let query = use_context::<Signal<String>>();
    let mut search_result = use_context::<Signal<Vec<Reference>>>();

    let on_type_change = move |e: Event<FormData>| {
        let selected_value = e.value();
        let new_type = match selected_value.as_str() {
            "Article" => FilterType::Article,
            "Book" => FilterType::Book,
            "Thesis" => FilterType::Thesis,
            "TechReport" => FilterType::TechReport,
            "Misc" => FilterType::Misc,
            "Booklet" => FilterType::Booklet,
            "InBook" => FilterType::InBook,
            "InCollection" => FilterType::InCollection,
            "InProceedings" => FilterType::InProceedings,
            _ => FilterType::All,
        };
        filter_type.set(new_type);
        // Re-run search if there's a query
        if !query().is_empty() {
            let result = match filter_field() {
                FilterField::Author => search_references_by_author(&refs(), &query()),
                FilterField::Title => search_references_by_title(&refs(), &query()),
                FilterField::Journal => search_references_by_journal(&refs(), &query()),
                FilterField::Year => search_references_by_year(&refs(), &query()),
                FilterField::All => search_references(&refs(), &query()),
            };
            search_result.set(result);
        }
    };
    rsx! {
        select { class: "select join-item w-35 shrink-0", onchange: on_type_change,
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
                value: "{FilterType::Thesis}",
                selected: filter_type() == FilterType::Thesis,
                "{FilterType::Thesis}"
            }
            option {
                value: "{FilterType::TechReport}",
                selected: filter_type() == FilterType::TechReport,
                "{FilterType::TechReport}"
            }
            option {
                value: "{FilterType::Misc}",
                selected: filter_type() == FilterType::Misc,
                "{FilterType::Misc}"
            }
            option {
                value: "{FilterType::Booklet}",
                selected: filter_type() == FilterType::Booklet,
                "{FilterType::Booklet}"
            }
            option {
                value: "{FilterType::InBook}",
                selected: filter_type() == FilterType::InBook,
                "{FilterType::InBook}"
            }
            option {
                value: "{FilterType::InCollection}",
                selected: filter_type() == FilterType::InCollection,
                "{FilterType::InCollection}"
            }
            option {
                value: "{FilterType::InProceedings}",
                selected: filter_type() == FilterType::InProceedings,
                "{FilterType::InProceedings}"
            }
        }
    }
}
