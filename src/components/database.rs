use dioxus::prelude::*;
use rfd::FileDialog;
use std::{collections::BTreeMap, path::PathBuf};

static MODAL_CSS: Asset = asset!("/assets/styling/modal.css");

#[component]
pub fn Database(db: BTreeMap<String, PathBuf>) -> Element {
    let pairs = db
        .iter()
        .map(|(name, path)| (name.clone(), path.as_os_str().to_str().unwrap().to_string()))
        .collect::<Vec<_>>();
    rsx! {
        div {
            h2 { "文献列表" }
            for (name , path) in pairs {
                DatabaseItem { name, path }
            }
        }
    }
}

#[component]
pub fn DatabaseItem(name: String, path: String) -> Element {
    rsx! {
        div {
            h3 { {name} }
            p { {path} }
        }
    }
}

#[component]
pub fn AddItem(mut db: Signal<BTreeMap<String, PathBuf>>, mut show: Signal<bool>) -> Element {
    let mut name = use_signal(|| "".to_string());
    let mut path = use_signal(PathBuf::new);
    let mut add_path = use_signal(|| false);
    let save_available = use_memo(move || !name().is_empty() && add_path());
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
                    br {}
                    label { "路径" }
                    input {
                        id: "path-input",
                        r#type: "text",
                        value: "{path_string}",
                        readonly: true,
                    }
                    button { onclick: select_file, "浏览" }
                }

                // 底部按钮区域
                div { id: "footer",
                    button { id: "cancle-button", onclick: close_modal, "取消" }
                    button {
                        style: if save_available() { "#save-button-available" } else { "#save-button-unavailable" },
                        onclick: close_modal,
                        disabled: !save_available(),
                        "保存"
                    }
                }
            }
        }
    }
}
