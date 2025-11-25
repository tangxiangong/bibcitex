use crate::components::InlineMath;
use biblatex::Chunk;
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
