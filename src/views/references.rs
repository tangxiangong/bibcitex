use dioxus::prelude::*;

#[component]
pub fn References(bib: biblatex::Bibliography) -> Element {
    rsx! {
        div {
            h2 { "References" }
        }
    }
}
