use dioxus::prelude::*;
use katex::{OptsBuilder, OutputType, render_with_opts};

#[component]
pub fn InlineMath(content: String) -> Element {
    let opts = OptsBuilder::default()
        .output_type(OutputType::Mathml)
        .build()
        .unwrap();
    let html = render_with_opts(&content, &opts);
    let is_success = html.is_ok();
    let data = if let Ok(html) = html { html } else { content };
    rsx! {
        span {
            if !is_success {
                {data}
            } else {
                span { dangerous_inner_html: data }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_katex() {
        let opts = katex::OptsBuilder::default()
            .output_type(katex::OutputType::Mathml)
            .build()
            .unwrap();
        let html = katex::render_with_opts(r#"\LaTeX"#, &opts);
        assert!(html.is_ok());
        println!("{}", html.unwrap());
    }
}
