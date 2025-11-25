use crate::{
    COPY_ICON, DETAILS_ICON, DRAWER_OPEN, DRAWER_REFERENCE, ERR_ICON, OK_ICON,
    components::ChunksComp,
};
use bibcitex_core::bib::Reference;
use dioxus::prelude::*;

/// ArXiv reference component.
///
/// # Example
///
/// ```tex
/// @misc{takemura2025errorestimatessemilagrangianschemes,
/// title={Error Estimates of Semi-Lagrangian Schemes for Diffusive Conservation Laws},
/// author={Haruki Takemura},
/// year={2025},
/// eprint={2508.03455},
/// archivePrefix={arXiv},
/// primaryClass={math.NA},
/// url={https://arxiv.org/abs/2508.03455},
/// }
/// ```
#[component]
pub fn ArXiv(entry: Reference) -> Element {
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

    let arxiv = match (entry.eprint.clone(), entry.arxiv_primary_class.clone()) {
        (Some(eprint), Some(primary_class)) => format!("arXiv:{eprint} [{primary_class}]"),
        (Some(eprint), None) => format!("arXiv:{eprint}"),
        (None, Some(primary_class)) => format!("arXiv [{primary_class}]"),
        (None, None) => "arXiv".to_string(),
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
        div { class: "card-modern card-shine group hover:-translate-y-1 transition-all duration-300 m-4 border-l-4 border-l-error",
            div { class: "card-body p-5",
                // Header: Type + Title + Actions
                div { class: "flex justify-between items-start gap-4",
                    div { class: "flex-1",
                        div { class: "flex items-center gap-2 mb-2",
                            span { class: "badge badge-error badge-soft badge-sm font-bold",
                                "ArXiv"
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
                            span { class: "badge badge-ghost hover:badge-error transition-colors cursor-default bg-base-200/50",
                                "{author}"
                            }
                        }
                    } else {
                        span { class: "text-sm text-base-content/50 italic", "Unknown Author" }
                    }
                }

                // Metadata Row
                div { class: "mt-4 flex flex-wrap items-center gap-4 text-sm text-base-content/70 border-t border-base-content/5 pt-3",
                    div { class: "flex items-center gap-1",
                        span { class: "font-semibold text-error", "📜" }
                        span { "{arxiv}" }
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
                            class: "btn btn-xs btn-ghost gap-1 hover:text-error",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            "DOI"
                        }
                    }
                    if let Some(url) = entry.url {
                        button {
                            class: "btn btn-xs btn-ghost gap-1 hover:text-error",
                            onclick: move |_| {
                                let _ = opener::open_browser(&url);
                            },
                            "URL"
                        }
                    }
                    if let Some(file) = entry.file {
                        button {
                            class: "btn btn-xs btn-error btn-soft gap-1",
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
pub fn ArXivDrawer(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let bibtex = entry.source.split('\n').collect::<Vec<_>>();
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    rsx! {
        div {
            div { class: "collapse collapse-arrow",
                input { r#type: "checkbox", checked: true }
                div { class: "collapse-title", "Info" }
                div { class: "collapse-content",
                    table { class: "table",
                        tbody {
                            tr {
                                td { class: "text-right", "Type" }
                                td { "Misc" }
                            }
                            tr {
                                td { class: "text-right", "Key" }
                                td { "{key}" }
                            }
                            tr {
                                td { class: "text-right", "Title" }
                                if let Some(title) = entry.title {
                                    td {
                                        ChunksComp {
                                            chunks: title,
                                            cite_key: format!("ArXivDrawer-{key}"),
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
                                td { class: "text-right", "Archive" }
                                td { "arXiv" }
                            }
                            tr {
                                td { class: "text-right", "ePrint" }
                                if let Some(eprint) = entry.eprint {
                                    td { "{eprint}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Primary Class" }
                                if let Some(primary_class) = entry.arxiv_primary_class {
                                    td { "{primary_class}" }
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
            div { class: "collapse collapse-arrow",
                input { r#type: "checkbox" }
                div { class: "collapse-title", "Abstract" }
                div { class: "collapse-content",
                    if let Some(abstract_chunks) = entry.abstract_ {
                        ChunksComp {
                            chunks: abstract_chunks,
                            cite_key: format!("{key}-abstract"),
                        }
                    }
                }
            }
            div { class: "collapse collapse-arrow",
                input { r#type: "checkbox" }
                div { class: "collapse-title", "Note" }
                div { class: "collapse-content",
                    if let Some(note) = entry.note {
                        ChunksComp { chunks: note, cite_key: format!("{key}-note") }
                    }
                }
            }
            div { class: "collapse collapse-arrow",
                input { r#type: "checkbox" }
                div { class: "collapse-title", "BibTeX" }
                div { class: "collapse-content",
                    for line in bibtex {
                        p { "{line}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Misc(entry: Reference) -> Element {
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

    let is_arxiv = if let Some(ref prefix) = entry.archive_prefix {
        prefix == "arXiv"
    } else {
        false
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
        if is_arxiv {
            ArXiv { entry }
        } else {
            div { class: "bg-gray-100 border-gray-500 card card-border border-2 m-2",
                div { class: "card-body",
                    div { class: "flex justify-between items-start",
                        div { class: "flex items-center",
                            div { class: "badge badge-outline mr-2 text-lg text-gray-800",
                                "Misc"
                            }
                            if let Some(title) = entry.title {
                                span { class: "badge badge-outline text-lg text-gray-900 font-serif",
                                    ChunksComp { chunks: title, cite_key: key.clone() }
                                }
                            } else {
                                span { class: "badge badge-outline text-lg text-gray-900 font-serif",
                                    "No title available"
                                }
                            }
                        }
                        div { class: "flex items-center shrink-0",
                            button {
                                class: "tooltip ml-2 mr-2 cursor-pointer",
                                onclick: copy_key,
                                "data-tip": "{key}",
                                if !copied() {
                                    img { width: 20, src: COPY_ICON }
                                } else {
                                    if copy_success() {
                                        img { width: 20, src: OK_ICON }
                                    } else {
                                        img { width: 20, src: ERR_ICON }
                                    }
                                }
                            }
                            button {
                                class: "tooltip cursor-pointer ml-2",
                                onclick: open_drawer,
                                "data-tip": "Details",
                                img { width: 20, src: DETAILS_ICON }
                            }
                        }
                    }
                    p {
                        if let Some(authors) = entry.author {
                            for author in authors {
                                span { class: "badge badge-outline text-blue-700 font-semibold mr-2",
                                    "{author} "
                                }
                            }
                        } else {
                            span { class: "badge badge-outline text-blue-700 font-semibold mr-2",
                                "Unknown"
                            }
                        }
                    }
                    p {
                        if let Some(archive) = entry.archive_prefix {
                            span { class: "badge badge-outline text-purple-700 mr-2",
                                "{archive}"
                            }
                        } else {
                            if let Some(how_published) = &entry.how_published {
                                span { class: "badge badge-outline text-purple-700 mr-2",
                                    "{how_published}"
                                }
                            }
                        }
                        if let Some(year) = &entry.year {
                            span { class: "badge badge-outline text-emerald-700 mr-2",
                                "{year}"
                            }
                        } else {
                            span { class: "badge badge-outline text-emerald-700 mr-2",
                                "year"
                            }
                        }
                        if let Some(doi) = &entry.doi {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&doi_url);
                                },
                                div { class: "badge badge-outline text-cyan-600 mr-2",
                                    "DOI: {doi}"
                                }
                            }
                        } else {
                            if let Some(url) = entry.url {
                                button {
                                    class: "tooltip cursor-pointer",
                                    "data-tip": "在浏览器中打开",
                                    onclick: move |_| {
                                        let _ = opener::open_browser(&url);
                                    },
                                    div { class: "badge badge-outline text-cyan-600 mr-2",
                                        "URL"
                                    }
                                }
                            }
                        }
                        if let Some(file) = entry.file {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "打开文献",
                                onclick: move |_| {
                                    let _ = opener::open(&file);
                                },
                                div { class: "badge badge-outline text-amber-700 mr-2",
                                    "PDF"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MiscDrawer(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let bibtex = entry.source.split('\n').collect::<Vec<_>>();
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    let is_arxiv = if let Some(ref prefix) = entry.archive_prefix {
        prefix == "arXiv"
    } else {
        false
    };

    rsx! {
        if is_arxiv {
            ArXivDrawer { entry }
        } else {
            div {
                div { class: "collapse collapse-arrow",
                    input { r#type: "checkbox", checked: true }
                    div { class: "collapse-title", "Info" }
                    div { class: "collapse-content",
                        table { class: "table",
                            tbody {
                                tr {
                                    td { class: "text-right", "Type" }
                                    td { "Misc" }
                                }
                                tr {
                                    td { class: "text-right", "Key" }
                                    td { "{key}" }
                                }
                                tr {
                                    td { class: "text-right", "Title" }
                                    if let Some(title) = entry.title {
                                        td {
                                            ChunksComp {
                                                chunks: title,
                                                cite_key: format!("MiscDrawer-{key}"),
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
                                    td { class: "text-right", "Archive" }
                                    if let Some(archive) = entry.archive_prefix {
                                        td { "{archive}" }
                                    } else {
                                        td { "" }
                                    }
                                }
                                tr {
                                    td { class: "text-right", "ePrint" }
                                    if let Some(eprint) = entry.eprint {
                                        td { "{eprint}" }
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
                div { class: "collapse collapse-arrow",
                    input { r#type: "checkbox" }
                    div { class: "collapse-title", "Abstract" }
                    div { class: "collapse-content",
                        if let Some(abstract_chunks) = entry.abstract_ {
                            ChunksComp {
                                chunks: abstract_chunks,
                                cite_key: format!("{key}-abstract"),
                            }
                        }
                    }
                }
                div { class: "collapse collapse-arrow",
                    input { r#type: "checkbox" }
                    div { class: "collapse-title", "Note" }
                    div { class: "collapse-content",
                        if let Some(note) = entry.note {
                            ChunksComp { chunks: note, cite_key: format!("{key}-note") }
                        }
                    }
                }
                div { class: "collapse collapse-arrow",
                    input { r#type: "checkbox" }
                    div { class: "collapse-title", "BibTeX" }
                    div { class: "collapse-content",
                        for line in bibtex {
                            p { "{line}" }
                        }
                    }
                }
            }
        }
    }
}
