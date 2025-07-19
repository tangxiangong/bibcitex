mod test_math;
pub(crate) use test_math::*;

use crate::{LOGO, MAIN_CSS, TAILWIND_CSS};
use dioxus::prelude::*;

#[component]
pub(crate) fn TestApp() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        img { src: LOGO, width: "100px" }
        TestMath { content: "E = mc^2" }
    }
}
