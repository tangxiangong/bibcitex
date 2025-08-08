use crate::{
    COPY_ICON, DETAILS_ICON, DRAWER_OPEN, DRAWER_REFERENCE, ERR_ICON, OK_ICON,
    components::InlineMath,
};
use bibcitex_core::{MSC_MAP, bib::Reference, parse_code};
use biblatex::{Chunk, EntryType};
use dioxus::prelude::*;

#[component]
pub fn ChunksComp(chunks: Vec<Chunk>, cite_key: String) -> Element {
    rsx! {
        for (i , chunk) in chunks.into_iter().enumerate() {
            match chunk {
                Chunk::Normal(txt) => rsx! {
                    span { key: "{cite_key}-{i}", "{txt}" }
                },
                Chunk::Verbatim(txt) => rsx! {
                    span { key: "{cite_key}-{i}", "{txt}" }
                },
                Chunk::Math(txt) => rsx! {
                    InlineMath { key: "{cite_key}-{i}", content: txt }
                },
            }
        }
    }
}

#[component]
pub fn ReferenceComponent(entry: Reference) -> Element {
    rsx! {
        match entry.type_ {
            EntryType::Article => rsx! {
                Article { entry }
            },
            EntryType::Book => rsx! {
                Book { entry }
            },
            EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => {
                rsx! {
                    Thesis { entry }
                }
            }
            EntryType::InProceedings => rsx! {
                InProceedings { entry }
            },
            EntryType::TechReport => rsx! {
                TechReport { entry }
            },
            EntryType::Misc => rsx! {
                Misc { entry }
            },
            _ => rsx! {
                Entry { entry }
            },
        }
    }
}

#[component]
pub fn ReferenceDrawer(entry: Reference) -> Element {
    match entry.type_.clone() {
        EntryType::Article => rsx! {
            ArticleDrawer { entry }
        },
        EntryType::Book => rsx! {
            BookDrawer { entry }
        },
        EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => rsx! {
            ThesisDrawer { entry }
        },
        EntryType::InProceedings => rsx! {
            InProceedingsDrawer { entry }
        },
        EntryType::TechReport => rsx! {
            TechReportDrawer { entry }
        },
        EntryType::Misc => rsx! {
            MiscDrawer { entry }
        },
        _ => rsx! {},
    }
}

#[component]
pub fn Entry(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let mut copy_success = use_signal(|| true);
    let mut copied = use_signal(|| false);
    let mut is_hovered = use_signal(|| false);
    let mut mouse_pos = use_signal(|| (0.0, 0.0));
    let mouseover = move |e: Event<MouseData>| {
        is_hovered.set(true);
        mouse_pos.set((e.client_coordinates().x, e.client_coordinates().y));
    };

    let mouseout = move |_| {
        is_hovered.set(false);
    };

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

    rsx! {
        div { class: "entry",
            p { class: "entry-header",
                div {
                    span { class: "entry-type", "{entry.type_.to_string().to_uppercase()} " }
                    if let Some(journal) = &entry.journal {
                        span { class: "entry-journal", "{journal}" }
                    }
                }
                span {
                    class: "entry-key",
                    onmouseover: mouseover,
                    onmouseout: mouseout,
                    " {key}"
                    button { onclick: copy_key,
                        if !copied() {
                            "📋"
                        } else {
                            if copy_success() {
                                "✅"
                            } else {
                                "❌"
                            }
                        }
                    }
                }
            }
            if let Some(title) = &entry.title {
                p {
                    ChunksComp { chunks: title.clone(), cite_key: key.clone() }
                }
            }
            if let Some(authors) = &entry.author {
                p {
                    if authors.len() > 5 {
                        for author in authors.iter().take(5) {
                            span { class: "author-tag", "{author}" }
                        }
                        span { " et al." }
                    } else {
                        for author in authors {
                            span { class: "author-tag-alt", "{author}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Article(entry: Reference) -> Element {
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

    let msc_text = if let Some(ref raw) = entry.mrclass {
        if MSC_MAP.is_empty() {
            Vec::new()
        } else {
            let codes = parse_code(raw);
            let mut texts = Vec::with_capacity(codes.len() + 1);
            for code in codes {
                if let Some(text) = MSC_MAP.get(&code) {
                    texts.push(text.clone());
                }
            }
            texts
        }
    } else {
        Vec::new()
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
        div { class: "bg-blue-100 border-blue-500 border-l-4",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-blue-800", "Article" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-gray-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
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
                            span { class: "text-blue-700 font-semibold mr-2", "{author} " }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-2", "Unknown" }
                    }
                }
                p {
                    if let Some(journal) = &entry.journal {
                        span { class: "text-purple-600 mr-2", "{journal}" }
                    } else {
                        span { class: "text-purple-600 mr-2", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        span { class: "text-emerald-700 mr-2", "year" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
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
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }

                    if let Some(mrclass) = entry.mrclass {
                        div { class: "tooltip",
                            div { class: "tooltip-content",
                                if msc_text.is_empty() {
                                    p { "The MR Class code is not available" }
                                } else {
                                    for text in msc_text {
                                        p { class: "text-left", "{text}" }
                                    }
                                }
                            }
                            div { class: "text-red-500 cursor-pointer", "{mrclass}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ArticleDrawer(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let pages_string = if let Some(ref pages) = entry.pages {
        if pages.start == pages.end {
            pages.start.to_string()
        } else {
            format!("{}-{}", pages.start, pages.end)
        }
    } else {
        "".to_string()
    };
    let bibtex = entry.source.split('\n').collect::<Vec<_>>();
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    let msc_text = if let Some(ref raw) = entry.mrclass {
        if MSC_MAP.is_empty() {
            Vec::new()
        } else {
            let codes = parse_code(raw);
            let mut texts = Vec::with_capacity(codes.len() + 1);
            for code in codes {
                if let Some(text) = MSC_MAP.get(&code) {
                    texts.push(text.clone());
                }
            }
            texts
        }
    } else {
        Vec::new()
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
                                td { "Journal Article" }
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
                                            cite_key: key.clone(),
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
                            if let Some(full_journal) = entry.full_journal {
                                tr {
                                    td { class: "text-right", "Journal" }
                                    td { "{full_journal}" }
                                }
                                tr {
                                    td { class: "text-right", "Journal Abbr" }
                                    if let Some(journal) = entry.journal {
                                        td { "{journal}" }
                                    } else {
                                        td { "" }
                                    }
                                }
                            } else {
                                tr {
                                    td { class: "text-right", "Journal" }
                                    if let Some(journal) = entry.journal {
                                        td { "{journal}" }
                                    } else {
                                        td { "" }
                                    }
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
                                td { class: "text-right", "Number" }
                                if let Some(number) = entry.number {
                                    td { "{number}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Pages" }
                                if entry.pages.is_some() {
                                    td { "{pages_string}" }
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
                                td { class: "text-right", "MR Class" }
                                if let Some(mrclass) = entry.mrclass {
                                    tr {
                                        td {
                                            div { class: "tooltip",
                                                div { class: "tooltip-content",
                                                    if msc_text.is_empty() {
                                                        p { "The MR Class code is not available" }
                                                    } else {
                                                        for text in msc_text {
                                                            p { class: "text-left",
                                                                "{text}"
                                                            }
                                                        }
                                                    }
                                                }
                                                div { class: "break-all", "{mrclass}" }
                                            }
                                        }
                                    }
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
pub fn Book(entry: Reference) -> Element {
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

    let msc_text = if let Some(ref raw) = entry.mrclass {
        if MSC_MAP.is_empty() {
            Vec::new()
        } else {
            let codes = parse_code(raw);
            let mut texts = Vec::with_capacity(codes.len() + 1);
            for code in codes {
                if let Some(text) = MSC_MAP.get(&code) {
                    texts.push(text.clone());
                }
            }
            texts
        }
    } else {
        Vec::new()
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
        div { class: "bg-emerald-100 border-emerald-500 border-l-4",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-emerald-800", "Book" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-gray-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
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
                            span { class: "text-blue-700 font-semibold mr-2", "{author} " }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-2", "Unknown" }
                    }
                }
                p {
                    if let Some(publishers) = &entry.publisher {
                        for publisher in publishers {
                            span { class: "text-purple-600 mr-2", "{publisher}" }
                        }
                    } else {
                        span { class: "text-purple-600 mr-2", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        span { class: "text-emerald-700 mr-2", "year" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
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
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }

                    if let Some(mrclass) = entry.mrclass {
                        div { class: "tooltip",
                            div { class: "tooltip-content",
                                if msc_text.is_empty() {
                                    p { "The MR Class code is not available" }
                                } else {
                                    for text in msc_text {
                                        p { class: "text-left", "{text}" }
                                    }
                                }
                            }
                            div { class: "text-red-500 cursor-pointer", "{mrclass}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BookDrawer(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let bibtex = entry.source.split('\n').collect::<Vec<_>>();
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    let msc_text = if let Some(ref raw) = entry.mrclass {
        if MSC_MAP.is_empty() {
            Vec::new()
        } else {
            let codes = parse_code(raw);
            let mut texts = Vec::with_capacity(codes.len() + 1);
            for code in codes {
                if let Some(text) = MSC_MAP.get(&code) {
                    texts.push(text.clone());
                }
            }
            texts
        }
    } else {
        Vec::new()
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
                                td { "Book" }
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
                                            cite_key: key.clone(),
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
                                td { class: "text-right", "MR Class" }
                                if let Some(mrclass) = entry.mrclass {
                                    td {
                                        div { class: "tooltip",
                                            div { class: "tooltip-content",
                                                if msc_text.is_empty() {
                                                    p { "The MR Class code is not available" }
                                                } else {
                                                    for text in msc_text {
                                                        p { class: "text-left", "{text}" }
                                                    }
                                                }
                                            }
                                            div { class: "break-all", "{mrclass}" }
                                        }
                                    }
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
pub fn Thesis(entry: Reference) -> Element {
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
        div { class: if entry.type_ == EntryType::MastersThesis { "bg-pink-100 border-pink-500 border-l-4" } else { "bg-rose-100 border-rose-500 border-l-4" },
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: if entry.type_ == EntryType::MastersThesis { "mr-2 text-lg text-pink-800" } else { "mr-2 text-lg text-rose-800" },
                            "{type_}"
                        }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-gray-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
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
                            span { class: "text-blue-700 font-semibold mr-2", "{author}" }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-2", "Unknown" }
                    }
                }
                p {
                    if !school_address.is_empty() {
                        span { class: "text-purple-600 mr-2", "{school_address}" }
                    } else {
                        span { class: "text-purple-600 mr-2", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        span { class: "text-emerald-700 mr-2", "year" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
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
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ThesisDrawer(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let bibtex = entry.source.split('\n').collect::<Vec<_>>();
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    let type_ = match entry.type_ {
        EntryType::Thesis => "Thesis",
        EntryType::MastersThesis => "Master Thesis",
        EntryType::PhdThesis => "PhD Thesis",
        _ => "Unknown",
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
                                td { "{type_}" }
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
                                            cite_key: key.clone(),
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
                                td { class: "text-right", "School" }
                                if let Some(school) = entry.school {
                                    td { "{school}" }
                                } else {
                                    td { "" }
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
pub fn InProceedings(entry: Reference) -> Element {
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
    let date = if let Some(year) = entry.year {
        if let Some(month) = entry.month {
            format!("{year}-{month}")
        } else {
            year.to_string()
        }
    } else {
        "".to_string()
    };
    rsx! {
        div { class: "bg-purple-100 border-purple-500 border-l-4",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-purple-800", "InProceedings" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-gray-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
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
                            span { class: "text-blue-700 font-semibold mr-2", "{author} " }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-2", "Unknown" }
                    }
                }
                p {
                    if let Some(booktitle) = entry.book_title {
                        span { class: "text-purple-600 mr-2",
                            ChunksComp {
                                chunks: booktitle,
                                cite_key: format!("booktitle_{key}"),
                            }
                        }
                    } else {
                        span { class: "text-purple-600 mr-2", "Unknown" }
                    }
                    if !date.is_empty() {
                        span { class: "text-emerald-700 mr-2", "{date}" }
                    } else {
                        span { class: "text-emerald-700 mr-2", "date" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
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
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InProceedingsDrawer(entry: Reference) -> Element {
    let key = &entry.cite_key;
    let pages_string = if let Some(ref pages) = entry.pages {
        if pages.start == pages.end {
            pages.start.to_string()
        } else {
            format!("{}-{}", pages.start, pages.end)
        }
    } else {
        "".to_string()
    };
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
                                td { "InProceedings" }
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
                                            cite_key: key.clone(),
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
                                if let Some(booktitle) = entry.book_title {
                                    td {
                                        ChunksComp {
                                            chunks: booktitle,
                                            cite_key: format!("booktitle-drawer-{key}"),
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
                            tr {
                                if let Some(editors) = entry.editor {
                                    for (editor , type_) in editors {
                                        td { class: "text-right", "{type_}" }
                                        td { "{editor}" }
                                    }
                                } else {
                                    td { class: "text-right", "Editor" }
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
                            if let Some(organizations) = entry.organization {
                                for organization in organizations {
                                    tr {
                                        td { class: "text-right", "Organization" }
                                        td { "{organization}" }
                                    }
                                }
                            } else {
                                tr {
                                    td { class: "text-right", "Organization" }
                                    td { "" }
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
                                td { class: "text-right", "Number" }
                                if let Some(number) = entry.number {
                                    td { "{number}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Pages" }
                                if entry.pages.is_some() {
                                    td { "{pages_string}" }
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
                                td { class: "text-right", "Month" }
                                if let Some(month) = entry.month {
                                    td { "{month}" }
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
pub fn TechReport(entry: Reference) -> Element {
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
        div { class: "bg-amber-100 border-amber-500 border-l-4",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-amber-800", "TechReport" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-gray-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
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
                            span { class: "text-blue-700 font-semibold mr-2", "{author} " }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-2", "Unknown" }
                    }
                }
                p {
                    if let Some(institution) = &entry.institution {
                        span { class: "text-purple-600 mr-2", "{institution}" }
                    } else {
                        span { class: "text-purple-600 mr-2", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        span { class: "text-emerald-700 mr-2", "year" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
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
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TechReportDrawer(entry: Reference) -> Element {
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
                                td { "Journal Article" }
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
                                            cite_key: key.clone(),
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
                                td { class: "text-right", "Number" }
                                if let Some(number) = entry.number {
                                    td { "{number}" }
                                } else {
                                    td { "" }
                                }
                            }
                            tr {
                                td { class: "text-right", "Institution" }
                                if let Some(institution) = entry.institution {
                                    td { "{institution}" }
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
        div { class: "bg-gray-100 border-gray-500 border-l-4",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-gray-800", "Misc" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-gray-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
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
                            span { class: "text-blue-700 font-semibold mr-2", "{author} " }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-2", "Unknown" }
                    }
                }
                p {
                    span { class: "text-purple-600 mr-2", "{arxiv}" }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        span { class: "text-emerald-700 mr-2", "year" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
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
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ArXivDrawer(entry: Reference) -> Element {
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
                                            cite_key: key.clone(),
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
            div { class: "bg-gray-100 border-gray-500 border-l-4",
                div { class: "card-body",
                    div { class: "flex justify-between items-start",
                        div { class: "flex items-start",
                            div { class: "mr-2 text-lg text-gray-800", "Misc" }
                            if let Some(title) = entry.title {
                                span { class: "text-lg text-gray-900 font-serif",
                                    ChunksComp { chunks: title, cite_key: key.clone() }
                                }
                            } else {
                                span { class: "text-lg", "No title available" }
                            }
                        }
                        div { class: "flex items-center flex-shrink-0",
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
                                span { class: "text-blue-700 font-semibold mr-2", "{author} " }
                            }
                        } else {
                            span { class: "text-blue-700 font-semibold mr-2", "Unknown" }
                        }
                    }
                    p {
                        if let Some(archive) = entry.archive_prefix {
                            span { class: "text-purple-700 mr-2", "{archive}" }
                        } else {
                            if let Some(how_published) = &entry.how_published {
                                span { class: "text-purple-700 mr-2", "{how_published}" }
                            }
                        }
                        if let Some(year) = &entry.year {
                            span { class: "text-emerald-700 mr-2", "{year}" }
                        } else {
                            span { class: "text-emerald-700 mr-2", "year" }
                        }
                        if let Some(doi) = &entry.doi {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&doi_url);
                                },
                                div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                            }
                        } else {
                            if let Some(url) = entry.url {
                                button {
                                    class: "tooltip cursor-pointer",
                                    "data-tip": "在浏览器中打开",
                                    onclick: move |_| {
                                        let _ = opener::open_browser(&url);
                                    },
                                    div { class: "text-cyan-600 mr-2", "URL" }
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
                                div { class: "text-amber-700 mr-2", "PDF" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MiscDrawer(entry: Reference) -> Element {
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
                                                cite_key: key.clone(),
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
