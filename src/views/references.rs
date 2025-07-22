use crate::{CURRENT_REF, components::Entry};
use dioxus::prelude::*;

#[component]
pub fn References() -> Element {
    let refs = CURRENT_REF().unwrap();
    rsx! {
        div {
            h2 { "References ({refs.len()})" }
            for reference in refs {
                Entry { entry: reference }
            }
        }
    }
}
