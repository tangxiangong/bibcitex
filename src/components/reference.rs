use crate::components::InlineMath;
use bibcitex_core::bib::Reference;
use biblatex::Chunk;
use dioxus::prelude::*;

static REFERENCE_CSS: Asset = asset!("/assets/styling/reference.css");
static HOVER_CSS: Asset = asset!("/assets/styling/hover.css");

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
        document::Link { rel: "stylesheet", href: REFERENCE_CSS }
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
        document::Link { rel: "stylesheet", href: HOVER_CSS }
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
