use crate::{CURRENT_REF, components::Entry};
use dioxus::prelude::*;

#[component]
pub fn References() -> Element {
    let refs = CURRENT_REF().unwrap();
    println!("biblen: {}", refs.len());
    rsx! {
        div {
            h2 { "References ({refs.len()})" }
            for reference in refs {
                Entry { entry: reference }
            }
        }
    }
}
