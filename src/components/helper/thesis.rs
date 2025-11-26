use bibcitex_core::bib::Reference;
use biblatex::EntryType;
use dioxus::prelude::*;

use crate::components::ChunksComp;

#[component]
pub fn ThesisHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;

    let school_address = {
        if let Some(school) = entry.school {
            if let Some(address) = entry.address {
                format!("{school} ({address})")
            } else {
                school
            }
        } else {
            "".to_string()
        }
    };
    let type_ = match entry.type_ {
        EntryType::Thesis => "Thesis",
        EntryType::MastersThesis => "Master Thesis",
        EntryType::PhdThesis => "PhD Thesis",
        _ => "Unknown",
    };
    rsx! {
        div { class: "w-full",
            div { class: "flex justify-between items-center",
                div { class: "flex items-center",
                    div { class: if entry.type_ == EntryType::MastersThesis { " badge badge-outline mr-2 text-pink-800 dark:text-pink-200" } else { "badge badge-outline mr-2 text-rose-800 dark:text-rose-200" },
                        "{type_}"
                    }
                    if let Some(title) = entry.title {
                        span { class: "text-gray-900 dark:text-gray-100 font-serif",
                            ChunksComp { chunks: title, cite_key: key.clone() }
                        }
                    } else {
                        span { class: "text-gray-900 dark:text-gray-100 font-serif",
                            "No title available"
                        }
                    }
                }
                div { class: "flex items-center shrink-0",
                    div { class: "text-gray-600 dark:text-gray-400 text-xs font-mono ml-2",
                        "{key}"
                    }
                }
            }
            p { class: "text-xs mt-2 break-all",
                if let Some(authors) = entry.author {
                    if authors.len() > 3 {
                        for author in authors.iter().take(3) {
                            span { class: "badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2",
                                "{author} "
                            }
                        }
                        span { class: "badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2",
                            "et al."
                        }
                    } else {
                        for author in authors {
                            span { class: "badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2",
                                "{author} "
                            }
                        }
                    }
                } else {
                    span { class: "badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2",
                        "Unknown"
                    }
                }
            }
            p { class: "text-xs mt-2 break-all",
                if !school_address.is_empty() {
                    span { class: "badge badge-outline text-purple-600 dark:text-purple-300 mr-2",
                        "{school_address}"
                    }
                } else {
                    span { class: "badge badge-outline text-purple-600 dark:text-purple-300 mr-2",
                        "Unknown"
                    }
                }
                if let Some(year) = &entry.year {
                    span { class: "badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2",
                        "{year}"
                    }
                } else {
                    span { class: "badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2",
                        "year"
                    }
                }
            }
        }
    }
}
