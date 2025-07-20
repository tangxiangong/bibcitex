use dioxus::prelude::*;
use rfd::FileDialog;
use std::{collections::BTreeMap, path::PathBuf};

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

    let cursor_style = use_memo(move || {
        if save_available() {
            "pointer"
        } else {
            "not-allowed"
        }
    });

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0, 0, 0, 0.3); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: close_modal,

            div {
                style: "background: white; border-radius: 12px; box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2); width: 500px; max-height: 600px; overflow: hidden; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;",
                onclick: |e| e.stop_propagation(),

                // 对话框标题
                div { style: "padding: 20px 24px 16px; border-bottom: 1px solid #e5e5e7; display: flex; align-items: center; justify-content: space-between;",
                    h2 { style: "margin: 0; font-size: 17px; font-weight: 600; color: #1d1d1f;",
                        "添加文献库"
                    }
                    button {
                        style: "background: none; border: none; font-size: 18px; color: #86868b; cursor: pointer; padding: 4px; width: 24px; height: 24px; display: flex; align-items: center; justify-content: center; border-radius: 50%; transition: background-color 0.2s;",
                        onclick: close_modal,
                        "✕"
                    }
                }

                // 对话框内容
                div { style: "padding: 20px 24px; max-height: 400px; overflow-y: auto;",
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
                        r#type: "text",
                        value: "{path_string}",
                        readonly: true,
                        style: "background-color: #f0f0f0; cursor: not-allowed;",
                    }
                    button { onclick: select_file, "浏览" }
                }

                // 底部按钮区域
                div { style: "padding: 16px 24px 20px; border-top: 1px solid #e5e5e7; display: flex; justify-content: flex-end; gap: 12px;",
                    button {
                        style: "padding: 8px 16px; border: 1px solid #d2d2d7; background: white; color: #1d1d1f; border-radius: 6px; cursor: pointer; font-size: 14px; transition: background-color 0.2s;",
                        onclick: close_modal,
                        "取消"
                    }
                    button {
                        style: "padding: 8px 16px; border: none; background: #007aff; color: white; border-radius: 6px; cursor: {cursor_style}; font-size: 14px; transition: background-color 0.2s;",
                        onclick: close_modal,
                        disabled: !save_available(),
                        "保存"
                    }
                }
            }
        }
    }
}
