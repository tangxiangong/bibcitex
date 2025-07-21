use crate::components::{AddBibliography, Bibliographies};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let show_modal = use_signal(|| false);
    rsx! {
        Bibliographies { show_modal }
        if show_modal() {
            AddBibliography { show: show_modal }
        }
    }
}
