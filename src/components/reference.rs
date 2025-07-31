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
    rsx! {
        div { class: "card card-border",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "badge badge-soft badge-primary mr-2 text-lg",
                            "Article"
                        }
                        if let Some(title) = entry.title {
                            span { class: "text-lg",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "badge badge-soft badge-primary text-lg ml-2",
                        "{key}"
                        button { onclick: copy_key,
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
                                div { class: "badge badge-soft badge-primary mr-1",
                                    "{author}"
                                }
                            }
                            div { class: "badge badge-soft badge-primary mr-1", " et al." }
                        } else {
                            for author in authors {
                                div { class: "badge badge-soft badge-primary mr-1",
                                    "{author}"
                                }
                            }
                        }
                    } else {
                        div { class: "badge badge-soft badge-primary mr-1", "Unknown" }
                    }
                }
                p {
                    if let Some(journal) = &entry.journal {
                        div { class: "badge badge-soft badge-primary mr-1", "{journal}" }
                    } else {
                        div { class: "badge badge-soft badge-primary mr-1", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        div { class: "badge badge-soft badge-primary mr-1", "{year}" }
                    } else {
                        div { class: "badge badge-soft badge-primary mr-1", "Unknown" }
                    }
                }
            }
        }
    }
}
