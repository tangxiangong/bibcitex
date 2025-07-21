use crate::STATE;
use dioxus::prelude::*;
use itertools::Itertools;
use rfd::FileDialog;
use std::path::PathBuf;

static MODAL_CSS: Asset = asset!("/assets/styling/modal.css");

#[component]
pub fn Bibliographies(mut show_modal: Signal<bool>) -> Element {
    let open_modal = move |_| {
        show_modal.set(true);
    };
    let pairs = use_memo(|| {
        let state = STATE.read();
        state
            .bibliographies
            .iter()
            .sorted_by(|a, b| a.1.updated_at.cmp(&b.1.updated_at))
            .map(|(name, info)| {
                (
                    name.clone(),
                    info.path.as_os_str().to_str().unwrap().to_string(),
                    info.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                )
            })
            .collect::<Vec<_>>()
    });

    rsx! {
        div {
            div {
                h2 {
                    "文献列表"
                    button { onclick: open_modal, font_size: "16px", "+ 添加" }
                }
            }
            for (name , path , updated_at) in pairs() {
                BibliographyItem { name, path, updated_at }
            }
        }
    }
}

#[component]
pub fn BibliographyItem(name: String, path: String, updated_at: String) -> Element {
    rsx! {
        div {
            h3 { {name} }
            p { "{path} ({updated_at})" }
        }
    }
}

#[component]
pub fn AddBibliographyItem(mut show: Signal<bool>) -> Element {
    let exist_names = use_memo(|| {
        STATE
            .read()
            .bibliographies
            .keys()
            .cloned()
            .collect::<Vec<_>>()
    });
    let mut name = use_signal(|| "".to_string());
    let mut path = use_signal(PathBuf::new);
    let mut add_path = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let name_is_valid = use_memo(move || !name().is_empty() && !exist_names().contains(&name()));
    let save_available = use_memo(move || add_path() && name_is_valid());
    let path_string = use_memo(move || path().as_os_str().to_str().unwrap().to_owned());

    let select_file = move |_| {
        let file = FileDialog::new()
            .add_filter("bibtex", &["bib", "txt"])
            .set_title("选择文献文件")
            .pick_file();
        if let Some(file) = file {
            path.set(file);
            add_path.set(true)
        }
    };

    let close_modal = move |_| {
        show.set(false);
    };

    let save = move |_| {
        let mut state = STATE.write();
        match state.add_update_bibliography(&name(), path()) {
            Ok(_) => {
                if let Err(e) = state.update_file() {
                    error_message.set(Some(e.to_string()));
                } else {
                    show.set(false);
                }
            }
            Err(e) => error_message.set(Some(e.to_string())),
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: MODAL_CSS }
        div { id: "background", onclick: close_modal,

            div { id: "content", onclick: |e| e.stop_propagation(),

                // 对话框标题
                div { id: "header",
                    h2 { "添加文献库" }
                    button { onclick: close_modal, "✕" }
                }

                // 对话框内容
                div { id: "form",
                    label { "名称" }
                    input {
                        r#type: "text",
                        value: "{name}",
                        oninput: move |e| {
                            name.set(e.data.value());
                        },
                    }
                    if name_is_valid() {
                        span { "✅" }
                    } else {
                        span { "❌" }
                    }
                    br {}
                    label { "路径" }
                    input {
                        id: "path-input",
                        r#type: "text",
                        value: "{path_string}",
                        readonly: true,
                    }
                    button { onclick: select_file, "🔍" }
                }

                if let Some(error) = error_message() {
                    div { "❌{error}" }
                }

                // 底部按钮区域
                div { id: "footer",
                    button { id: "cancle-button", onclick: close_modal, "🚫取消" }
                    button {
                        style: if save_available() { "#save-button-available" } else { "#save-button-unavailable" },
                        onclick: save,
                        disabled: !save_available(),
                        "💾保存"
                    }
                }
            }
        }
    }
}
