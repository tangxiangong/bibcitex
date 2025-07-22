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

    rsx! {
        document::Link { rel: "stylesheet", href: REFERENCE_CSS }
        div { class: "entry",
            p { "{key}" }
            if let Some(title) = &entry.title {
                ChunksComp { chunks: title.clone(), cite_key: key.clone() }
            } else {
                p { "No title found" }
            }
        }
    }
}
