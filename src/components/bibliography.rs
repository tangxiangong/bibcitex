use crate::{CURRENT_REF, STATE, route::Route};
use bibcitex_core::{
    bib::parse,
    utils::{abbr_path, read_bibliography},
};
use dioxus::prelude::*;
use itertools::Itertools;
use rfd::FileDialog;
use std::path::PathBuf;

static ADD_ICON: Asset = asset!("/assets/icons/add.svg");
static ERR_ICON: Asset = asset!("/assets/icons/error.svg");
static OK_ICON: Asset = asset!("/assets/icons/ok.svg");
static CANCEL_ICON: Asset = asset!("/assets/icons/cancel.svg");
static DELETE_ICON: Asset = asset!("/assets/icons/delete.svg");

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
            .sorted_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at))
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
            h2 { class: "p-4 text-lg",
                "Bibliographies"
                button {
                    class: "btn btn-sm btn-circle btn-ghost bg-base-100",
                    onclick: open_modal,
                    img { width: 20, src: ADD_ICON }
                }
            }
            for (name , path , updated_at) in pairs() {
                Bibliography { name, path, updated_at }
            }
        }
    }
}

#[component]
pub fn Bibliography(name: String, path: String, updated_at: String) -> Element {
    let mut error_message = use_signal(|| None::<String>);
    let navigator = use_navigator();
    let path_clone = path.clone();

    let handle_click = move |_| {
        error_message.set(None);
        match parse(&path_clone) {
            Ok(bib) => {
                let refs = read_bibliography(bib);
                let mut current_ref = CURRENT_REF.write();
                *current_ref = Some(refs);
                navigator.push(Route::References {});
            }
            Err(e) => {
                error_message.set(Some(format!("❌ 解析文件失败: {e}")));
            }
        }
    };

    let delete_bib = |bib_name: String| {
        let mut state = STATE.write();
        state.remove_bibliography(&bib_name);
        let _ = state.update_file();
    };

    rsx! {
        div {
            div { class: "card card-border bg-base-100 card-xs shadow-sm",
                div { class: "card-body",
                    h2 { class: "card-title", onclick: handle_click, "{name}" }
                    p { "{path} ({updated_at})" }
                    div { class: "card-actions justify-end",
                        button {
                            class: "btn btn-ghost btn-circle",
                            onclick: move |_| delete_bib(name.clone()),
                            img { width: 30, src: DELETE_ICON }
                        }
                    }
                }
            }
            if let Some(error) = error_message() {
                p { "{error}" }
            }
        }
    }
}

#[component]
pub fn AddBibliography(mut show: Signal<bool>) -> Element {
    let exist_names = use_memo(|| {
        STATE
            .read()
            .bibliographies
            .keys()
            .cloned()
            .collect::<Vec<_>>()
    });
    let mut name = use_signal(|| "".to_string());
    let mut path = use_signal(|| None::<PathBuf>);
    let mut add_path = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let name_is_valid = use_memo(move || !name().is_empty() && !exist_names().contains(&name()));
    let save_available = use_memo(move || add_path() && name_is_valid());
    let path_string = use_memo(move || {
        if let Some(path) = path() {
            path.as_os_str().to_str().unwrap().to_owned()
        } else {
            "未选择文件，点击选择".to_string()
        }
    });
    let path_abbr_string = use_memo(move || {
        if let Some(path) = path() {
            let path_str = path.as_os_str().to_str().unwrap().to_owned();
            abbr_path(&path_str, 40)
        } else {
            "".to_string()
        }
    });

    let select_file = move |_| {
        let file = FileDialog::new()
            .add_filter("bibtex", &["bib", "txt"])
            .set_title("选择文献文件")
            .pick_file();
        if let Some(file) = file {
            path.set(Some(file));
            add_path.set(true);
            error_message.set(None);
        }
    };

    let close_modal = move |_| {
        show.set(false);
    };

    let save = move |_| {
        let mut state = STATE.write();
        if let Some(path) = path() {
            match state.add_update_bibliography(&name(), path) {
                Ok(_) => {
                    if let Err(e) = state.update_file() {
                        error_message.set(Some(e.to_string()));
                    } else {
                        show.set(false);
                    }
                }
                Err(e) => error_message.set(Some(e.to_string())),
            }
        }
    };

    rsx! {
        div { class: if show() { "modal modal-open" } else { "modal" },
            div { class: "modal-box",
                h2 { class: "text-lg font-bold p-4", "添加文献库" }
                label { class: "input",
                    "名称"
                    input {
                        class: "grow",
                        r#type: "text",
                        value: "{name}",
                        oninput: move |e| {
                            name.set(e.data.value());
                        },
                    }
                    if name_is_valid() {
                        img { width: 20, src: OK_ICON }
                    } else {
                        img { width: 20, src: ERR_ICON }
                    }
                }

                br {}
                label {
                    class: format!(
                        "input tooltip tooltip-bottom {}",
                        if path().is_some() { "tooltip-success" } else { "tooltip-error" },
                    ),
                    "data-tip": "{path_string}",
                    "路径"
                    input {
                        class: "grow",
                        r#type: "text",
                        placeholder: "请选择文件",
                        value: "{path_abbr_string}",
                        readonly: true,
                    }
                    button { onclick: select_file, "选取文件" }
                }
                if let Some(error) = error_message() {
                    p { "❌{error}" }
                }

                div { class: "modal-action p-3",
                    button { class: "btn btn-soft btn-error", onclick: close_modal,
                        img { width: 20, src: CANCEL_ICON }
                        "取消"
                    }
                    button {
                        class: if save_available() { "btn btn-soft btn-success" } else { "btn btn-soft btn-disabled" },
                        onclick: save,
                        disabled: !save_available(),
                        img { width: 20, src: OK_ICON }
                        "保存"
                    }
                }
            }
            div { class: "modal-backdrop", onclick: close_modal }
        }
    }
}
