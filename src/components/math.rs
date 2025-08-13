use dioxus::prelude::*;
use katex_gdef_v8::{KatexOutput, Options, render_with_opts};

#[component]
pub fn InlineMath(content: String) -> Element {
    let opts = Options {
        output: KatexOutput::Mathml,
        ..Default::default()
    };
    let html = render_with_opts(&content, &opts, &mut Default::default());
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
    use katex_gdef_v8::{KatexOutput, Options, render_with_opts};
    #[test]
    fn test_katex() {
        let opts = Options {
            output: KatexOutput::Mathml,
            ..Default::default()
        };
        let html = render_with_opts(r#"E = mc^2"#, &opts, &mut Default::default());
        assert!(html.is_ok());
        println!("{}", html.unwrap());
    }
}
