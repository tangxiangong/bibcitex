use bibcitex_core::bib::Reference;
use dioxus::prelude::*;

use crate::components::ChunksComp;

#[component]
pub fn BookletHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;
    rsx! {
        div { class: "w-full",
            div { class: "flex justify-between items-center",
                div { class: "flex items-center gap-2",
                    div { class: "badge badge-info badge-soft badge-sm font-bold", "Booklet" }
                    if let Some(title) = entry.title {
                        span { class: "text-gray-900 dark:text-gray-100 font-serif font-medium",
                            ChunksComp { chunks: title, cite_key: key.clone() }
                        }
                    } else {
                        span { class: "text-gray-900 dark:text-gray-100 font-serif italic",
                            "No title available"
                        }
                    }
                }
                div { class: "flex items-center shrink-0",
                    div { class: "text-xs font-mono opacity-50 ml-2", "{key}" }
                }
            }
            div { class: "mt-1 flex flex-wrap gap-1",
                if let Some(authors) = entry.author {
                    if authors.len() > 3 {
                        for author in authors.iter().take(3) {
                            span { class: "badge badge-ghost badge-xs hover:badge-info transition-colors cursor-default",
                                "{author}"
                            }
                        }
                        span { class: "badge badge-ghost badge-xs hover:badge-info transition-colors cursor-default",
                            "et al."
                        }
                    } else {
                        for author in authors {
                            span { class: "badge badge-ghost badge-xs hover:badge-info transition-colors cursor-default",
                                "{author}"
                            }
                        }
                    }
                } else {
                    span { class: "text-xs text-base-content/50 italic", "Unknown Author" }
                }
            }
            div { class: "mt-1 flex flex-wrap items-center gap-2 text-xs",
                if let Some(how_published) = &entry.how_published {
                    span { class: "flex items-center gap-1",
                        span { "📢 {how_published}" }
                    }
                }
                if let Some(year) = &entry.year {
                    span { class: "flex items-center gap-1 text-secondary",
                        span { "📅 {year}" }
                    }
                }
            }
        }
    }
}
