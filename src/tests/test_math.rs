use crate::components::InlineMath;
use dioxus::prelude::*;

#[component]
pub(crate) fn TestMath(content: String) -> Element {
    rsx! {
        h2 { "行内数学公式渲染测试" }
        p {
            "质能方程："
            InlineMath { content }
        }
    }
}
