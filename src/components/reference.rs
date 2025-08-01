use crate::{COPY_ICON, ERR_ICON, OK_ICON, components::InlineMath};
use bibcitex_core::bib::Reference;
use biblatex::Chunk;
use dioxus::prelude::*;

#[component]
fn ChunksComp(chunks: Vec<Chunk>, cite_key: String) -> Element {
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
        if is_hovered() {
            Hover {
                entry,
                mouse_x: mouse_pos().0 - 30.0,
                mouse_y: mouse_pos().1,
            }
        }
    }
}

#[component]
pub fn Hover(entry: Reference, mouse_x: f64, mouse_y: f64) -> Element {
    let key = &entry.cite_key;

    rsx! {
        div {
            class: "hover",
            style: "left: {mouse_x + 1.0}px; top: {mouse_y + 10.0}px;",
            p {
                span { class: "hover-type", "{entry.type_.to_string()} " }
                " {key}"
            }
            p {
                if let Some(title) = &entry.title {
                    ChunksComp { chunks: title.clone(), cite_key: key.clone() }
                }

                if let Some(authors) = &entry.author {
                    for author in authors {
                        span { "{author}" }
                    }
                } else {
                    span { "No author found" }
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
    let open_doi = |doi: String| {
        let _ = opener::open_browser(&doi);
    };
    let open_url = |url: String| {
        let _ = opener::open_browser(&url);
    };
    let open_file = |file: String| {
        let _ = opener::open(&file);
    };
    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };
    rsx! {
        div { class: "bg-blue-100 border-blue-500 border-l-4 px-3 py-2 rounded-r",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-blue-800", "Article" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-grey-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div {
                        button {
                            class: "badge tooltip bg-blue-100 text-gray-400 text-sm font-mono  hover:text-gray-600 ml-1 cursor-pointer",
                            onclick: copy_key,
                            "data-tip": "点击以复制",
                            "{key}"
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
                    }
                }
                p {
                    if let Some(authors) = entry.author {
                        if authors.len() > 3 {
                            for author in authors.iter().take(3) {
                                span { class: "text-blue-700 font-semibold mr-2", "{author}" }
                            }
                            span { class: "font-semibold mr-1", " et al." }
                        } else {
                            for author in authors {
                                span { class: "text-blue-700 font-semibold bg-blue-100 mr-2",
                                    "{author} "
                                }
                            }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-1", "Unknown" }
                    }
                }
                p {
                    if let Some(journal) = &entry.journal {
                        span { class: "text-purple-600 mr-2", "{journal}" }
                    } else {
                        div { class: "badge badge-soft badge-primary mr-1", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        div { class: "text-emerald-700 mr-1", "year" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| open_doi(doi_url.clone()),
                            div { class: "text-cyan-600 mr-2", " DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| open_url(url.clone()),
                                div { class: "badge badge-soft badge-primary mr-1",
                                    "URL"
                                }
                            }
                        }
                    }
                    if let Some(file) = entry.file {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "打开文献",
                            onclick: move |_| open_file(file.clone()),
                            div { class: "badge badge-soft badge-primary mr-1", "PDF" }
                        }
                    }
                }
            }
        }
    }
}
