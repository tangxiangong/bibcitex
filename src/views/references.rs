use crate::{
    CURRENT_REF,
    components::{Article, Book, Entry, Thesis},
};
use bibcitex_core::{
    bib::Reference, filter_article, filter_book, filter_thesis, search_references,
    search_references_by_author, search_references_by_journal, search_references_by_title,
    search_references_by_year,
};
use biblatex::EntryType;
use dioxus::prelude::*;

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
    Thesis,
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
            FilterType::All => write!(f, "Type"),
        }
    }
}

#[component]
pub fn References() -> Element {
    let total_num = CURRENT_REF().unwrap().len();
    let mut query = use_signal(String::new);
    let mut is_input = use_signal(|| false);
    let mut search_result = use_signal(Vec::<Reference>::new);
    let mut filter_field = use_signal(|| FilterField::All);
    let mut filter_type = use_signal(|| FilterType::All);
    let refs = use_memo(move || {
        let total_refs = CURRENT_REF().unwrap();
        match filter_type() {
            FilterType::Book => filter_book(total_refs),
            FilterType::Article => filter_article(total_refs),
            FilterType::Thesis => filter_thesis(total_refs),
            FilterType::All => total_refs,
        }
    });
    let show_type = use_memo(move || match filter_type() {
        FilterType::All => "References".to_string(),
        FilterType::Article => "Articles".to_string(),
        FilterType::Book => "Books".to_string(),
        FilterType::Thesis => "Thesis".to_string(),
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

    let on_type_change = move |e: Event<FormData>| {
        let selected_value = e.value();
        let new_type = match selected_value.as_str() {
            "Article" => FilterType::Article,
            "Book" => FilterType::Book,
            "Thesis" => FilterType::Thesis,
            _ => FilterType::All,
        };
        filter_type.set(new_type);
    };

    rsx! {
        div { class: "p-4",
            div { class: "join w-full",
                select { class: "select join-item w-24", onchange: on_type_change,
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
                }
                {
                    match filter_type() {
                        FilterType::All => rsx! {
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
                        },
                        FilterType::Article => rsx! {
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
                        },
                        FilterType::Book => rsx! {
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
                        FilterType::Thesis => rsx! {
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
                input {
                    r#type: "search",
                    class: "input input-primary join-item flex-1",
                    placeholder: "搜索文献...",
                    value: "{query}",
                    oninput: search,
                }
            }
        }
        div {
            if !is_input() {
                h2 { class: "text-lg p-2", "{show_type()} ({refs().len()}/{total_num})" }
                for reference in refs() {
                    match reference.type_.clone() {
                        EntryType::Article => rsx! {
                            Article { entry: reference, is_helper: false }
                        },
                        EntryType::Book => rsx! {
                            Book { entry: reference, is_helper: false }
                        },
                        EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => {
                            rsx! {
                                Thesis { entry: reference, is_helper: false }
                            }
                        }
                        _ => rsx! {
                            Entry { entry: reference }
                        },
                    }
                }
            } else {
                h2 { class: "text-lg p-2", "{show_type()} ({search_result().len()}/{refs().len()})" }
                if !search_result().is_empty() {
                    for reference in search_result() {
                        match reference.type_.clone() {
                            EntryType::Article => rsx! {
                                Article { entry: reference, is_helper: false }
                            },
                            EntryType::Book => rsx! {
                                Book { entry: reference, is_helper: false }
                            },
                            EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => {
                                rsx! {
                                    Thesis { entry: reference, is_helper: false }
                                }
                            }
                            _ => rsx! {
                                Entry { entry: reference }
                            },
                        }
                    }
                } else {
                    p { class: "p-2 text-lg text-red-500", "No results" }
                }
            }
        }
    }
}
