use crate::components::{AddBibliographyItem, Bibliographies};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let show_modal = use_signal(|| false);
    rsx! {
        Bibliographies { show_modal }
        if show_modal() {
            AddBibliographyItem { show: show_modal }
        }
    }
}
