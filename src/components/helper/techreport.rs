use bibcitex_core::bib::Reference;
use dioxus::prelude::*;

use crate::components::ChunksComp;

#[component]
pub fn TechReportHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;

    rsx! {
        div { class: "w-full",
            div { class: "flex justify-between items-center",
                div { class: "flex items-center",
                    div { class: "badge badge-outline mr-2 text-amber-800 dark:text-amber-200",
                        "TechReport"
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
                if let Some(institution) = &entry.institution {
                    span { class: "badge badge-outline text-purple-600 dark:text-purple-300 mr-2",
                        "{institution}"
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
