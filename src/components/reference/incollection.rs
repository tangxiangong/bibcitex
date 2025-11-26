use bibcitex_core::bib::Reference;
use dioxus::prelude::*;

use crate::{
    COPY_ICON, DETAILS_ICON, DRAWER_OPEN, DRAWER_REFERENCE, ERR_ICON, OK_ICON,
    components::ChunksComp,
};

#[component]
pub fn InCollection(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let mut copy_success = use_signal(|| true);
    let mut copied = use_signal(|| false);

    let copy_key = {
        let key_clone = key.clone();
        move |_| {
            copied.set(true);
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if clipboard.set_text(&key_clone).is_ok() {
                    copy_success.set(true);
                } else {
                    copy_success.set(false);
                }
            } else {
                copy_success.set(false);
            }

            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                copied.set(false);
            });
        }
    };
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    let open_drawer = {
        let entry_for_drawer = entry.clone();
        move |_| {
            *DRAWER_REFERENCE.write() = Some(entry_for_drawer.clone());
            *DRAWER_OPEN.write() = true;
        }
    };
    rsx! {
        div { class: "card-modern card-shine group hover:-translate-y-1 transition-all duration-300 m-4 border-l-4 border-l-fuchsia-500",
            div { class: "card-body p-5",
                // Header: Type + Title + Actions
                div { class: "flex justify-between items-start gap-4",
                    div { class: "flex-1",
                        div { class: "flex items-center gap-2 mb-2",
                            span { class: "badge badge-secondary badge-soft badge-sm font-bold",
                                "InCollection"
                            }
                            span { class: "text-xs font-mono opacity-50 select-all",
                                "{key}"
                            }
                        }
                        if let Some(title) = entry.title {
                            h3 { class: "text-xl font-bold leading-snug gradient-text",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg text-base-content/50 italic", "No title available" }
                        }
                    }
                    // Actions
                    div { class: "flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200",
                        button {
                            class: "btn btn-ghost btn-sm btn-circle tooltip tooltip-left",
                            "data-tip": "Copy Key",
                            onclick: copy_key,
                            if !copied() {
                                img {
                                    width: 18,
                                    src: COPY_ICON,
                                    class: "opacity-70",
                                }
                            } else {
                                if copy_success() {
                                    img {
                                        width: 18,
                                        src: OK_ICON,
                                        class: "text-success",
                                    }
                                } else {
                                    img {
                                        width: 18,
                                        src: ERR_ICON,
                                        class: "text-error",
                                    }
                                }
                            }
                        }
                        button {
                            class: "btn btn-ghost btn-sm btn-circle tooltip tooltip-left",
                            "data-tip": "Details",
                            onclick: open_drawer,
                            img {
                                width: 18,
                                src: DETAILS_ICON,
                                class: "opacity-70",
                            }
                        }
                    }
                }

                // Authors
                div { class: "mt-3 flex flex-wrap gap-2",
                    if let Some(authors) = entry.author {
                        for author in authors {
                            span { class: "badge badge-ghost hover:badge-secondary transition-colors cursor-default bg-base-200/50",
                                "{author}"
                            }
                        }
                    } else {
                        span { class: "text-sm text-base-content/50 italic", "Unknown Author" }
                    }
                }

                // Metadata Row
                div { class: "mt-4 flex flex-wrap items-center gap-4 text-sm text-base-content/70 border-t border-base-content/5 pt-3",
                    if let Some(booktitle) = &entry.book_title {
                        div { class: "flex items-center gap-1",
                            span { class: "font-semibold text-primary", "📘" }
                            span { class: "italic",
                                ChunksComp {
                                    chunks: booktitle.clone(),
                                    cite_key: format!("InCollection-BT-{key}"),
                                }
                            }
                        }
                    }
                    if let Some(publishers) = &entry.publisher {
                        for publisher in publishers {
                            div { class: "flex items-center gap-1",
                                span { class: "font-semibold text-primary", "🏢" }
                                span { "{publisher}" }
                            }
                        }
                    }
                    if let Some(year) = &entry.year {
                        div { class: "flex items-center gap-1",
                            span { class: "font-semibold text-secondary", "📅" }
                            span { "{year}" }
                        }
                    }
                    // Links
                    div { class: "flex-1" } // Spacer
                    if entry.doi.is_some() {
                        button {
                            class: "btn btn-xs btn-ghost gap-1 hover:text-secondary",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            "DOI"
                        }
                    }
                    if let Some(url) = entry.url {
                        button {
                            class: "btn btn-xs btn-ghost gap-1 hover:text-secondary",
                            onclick: move |_| {
                                let _ = opener::open_browser(&url);
                            },
                            "URL"
                        }
                    }
                    if let Some(file) = entry.file {
                        button {
                            class: "btn btn-xs btn-primary btn-soft gap-1",
                            onclick: move |_| {
                                let _ = opener::open(&file);
                            },
                            "PDF"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn InCollectionDrawer(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let bibtex = entry.source.split('\n').collect::<Vec<_>>();
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    rsx! {
        div { class: "space-y-2",
            div { class: "collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box",
                input { r#type: "checkbox", checked: true }
                div { class: "collapse-title font-medium", "Info" }
                div { class: "collapse-content",
                    table { class: "table table-sm",
                        tbody {
                            tr {
                                td { class: "text-right opacity-70 font-semibold",
                                    "Type"
                                }
                                td { "InBook" }
                            }
                            tr {
                                td { class: "text-right opacity-70 font-semibold",
                                    "Key"
                                }
                                td { "{key}" }
                            }
                            tr {
                                td { class: "text-right opacity-70 font-semibold",
                                    "Title"
                                }
                                if let Some(title) = entry.title {
                                    td {
                                        ChunksComp {
                                            chunks: title,
                                            cite_key: format!("InCollectionDrawer-{key}"),
                                        }
                                    }
                                } else {
                                    td { "" }
                                }
                            }
                            if let Some(authors) = entry.author {
                                for author in authors {
                                    tr {
                                        td { class: "text-right", "Author" }
                                        td { "{author}" }
                                    }
                                }
                            } else {
                                tr {
                                    td { class: "text-right", "Author" }
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Book Title" }
                                if let Some(book_title) = entry.book_title {
                                    td {
                                        ChunksComp {
                                            chunks: book_title,
                                            cite_key: format!("InBook-{key}"),
                                        }
                                    }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Series" }
                                if let Some(series) = entry.series {
                                    td { "{series}" }
                                } else {
                                    td { "" }
                                }
                            }
                            if let Some(publishers) = entry.publisher {
                                for publisher in publishers {
                                    tr {
                                        td { class: "text-right", "Publisher" }
                                        td { "{publisher}" }
                                    }
                                }
                            } else {
                                tr {
                                    td { class: "text-right", "Publisher" }
                                    td { "" }
                                }
                            }
                            if let Some(editors) = entry.editor {
                                for (editor , type_) in editors {
                                    tr {
                                        td { class: "text-right", "{type_}" }
                                        td { "{editor}" }
                                    }
                                }
                            }
                            tr {
                                td { class: "text-right", "Address" }
                                if let Some(address) = entry.address {
                                    td { "{address}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Volume" }
                                if let Some(volume) = entry.volume {
                                    td { "{volume}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Edition" }
                                if let Some(edition) = entry.edition {
                                    td { "{edition}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Pages" }
                                if let Some(book_pages) = entry.book_pages {
                                    td { "{book_pages}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Year" }
                                if let Some(year) = entry.year {
                                    td { "{year}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "ISBN" }
                                if let Some(isbn) = entry.isbn {
                                    td { "{isbn}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "DOI" }
                                if let Some(doi) = entry.doi {
                                    td { class: "break-all",
                                        button {
                                            class: "tooltip cursor-pointer text-left break-all",
                                            "data-tip": "在浏览器中打开",
                                            onclick: move |_| {
                                                let _ = opener::open_browser(&doi_url);
                                            },
                                            "{doi}"
                                        }
                                    }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "URL" }
                                if let Some(url) = entry.url {
                                    td { class: "break-all",
                                        button {
                                            class: "tooltip cursor-pointer text-left break-all",
                                            "data-tip": "在浏览器中打开",
                                            onclick: move |_| {
                                                let _ = opener::open_browser(&url);
                                            },
                                            "{url}"
                                        }
                                    }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "File" }
                                if let Some(file) = entry.file {
                                    td {
                                        button {
                                            class: "tooltip cursor-pointer text-left break-all",
                                            "data-tip": "打开",
                                            onclick: move |_| {
                                                let _ = opener::open(&file);
                                            },
                                            "{file}"
                                        }
                                    }
                                } else {
                                    td { "" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box",
                input { r#type: "checkbox" }
                div { class: "collapse-title font-medium", "Abstract" }
                div { class: "collapse-content",
                    if let Some(abstract_chunks) = entry.abstract_ {
                        ChunksComp {
                            chunks: abstract_chunks,
                            cite_key: format!("{key}-abstract"),
                        }
                    }
                }
            }
            div { class: "collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box",
                input { r#type: "checkbox" }
                div { class: "collapse-title font-medium", "Note" }
                div { class: "collapse-content",
                    if let Some(note) = entry.note {
                        ChunksComp { chunks: note, cite_key: format!("{key}-note") }
                    }
                }
            }
            div { class: "collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box",
                input { r#type: "checkbox" }
                div { class: "collapse-title font-medium", "BibTeX" }
                div { class: "collapse-content",
                    for line in bibtex {
                        p { class: "font-mono text-xs", "{line}" }
                    }
                }
            }
        }
    }
}
