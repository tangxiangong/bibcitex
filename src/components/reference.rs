use crate::components::InlineMath;
use bibcitex_core::bib::Reference;
use biblatex::Chunk;
use dioxus::prelude::*;

static REFERENCE_CSS: Asset = asset!("/assets/styling/reference.css");

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
            p {
                span { style: "color: red", "{entry.type_.to_string()} " }
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
            if let Some(title) = &entry.title {
                ChunksComp { chunks: title.clone(), cite_key: key.clone() }
            } else {
                p { "No title found" }
            }
        }
    }
}
