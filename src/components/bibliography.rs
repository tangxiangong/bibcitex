use crate::{
    ADD_ICON, CANCEL_ICON, CURRENT_REF, DELETE_ICON, ERR_ICON, OK_ICON, STATE, route::Route,
};
use bibcitex_core::{
    bib::parse,
    utils::{abbr_path, read_bibliography},
};
use dioxus::prelude::*;
use itertools::Itertools;
use rfd::FileDialog;
use std::{path::PathBuf, time::Duration};

#[component]
pub fn Bibliographies(mut show_modal: Signal<bool>) -> Element {
    let open_modal = move |_| {
        show_modal.set(true);
    };
    let mut error_message = use_context_provider(|| Signal::new(None::<String>));
    let mut is_fading_out = use_signal(|| false);
    let show_error = use_memo(move || error_message().is_some() || is_fading_out());
    let mut progress = use_signal(|| 100);
    use_effect(move || {
        if error_message().is_some() && !is_fading_out() {
            progress.set(100);
            spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    let current = progress();
                    if current == 0 {
                        // 开始渐出动画
                        is_fading_out.set(true);
                        // 等待渐出动画完成（300ms）
                        tokio::time::sleep(Duration::from_millis(300)).await;
                        // 清除状态
                        progress.set(100);
                        error_message.set(None);
                        is_fading_out.set(false);
                        break;
                    } else {
                        progress.set(current - 1);
                    }
                }
            });
        }
    });
    rsx! {
        div { class: "relative",
            h2 { class: "p-4 text-lg flex items-center",
                "Bibliographies"
                button {
                    class: "tooltip tooltip-right ml-2 cursor-pointer flex items-center",
                    "data-tip": "Add bibliography",
                    onclick: open_modal,
                    img { width: 16, src: ADD_ICON }
                }
            }
            BibliographyTable {}
            if show_error() {
                div { class: if is_fading_out() { "absolute top-2 right-2 w-1/3 z-10 bg-base-100 animate-fade-out indicator" } else { "absolute top-2 right-2 w-1/3 z-10 bg-base-100 animate-fade-in indicator" },
                    div { class: "indicator-item",
                        button {
                            class: "cursor-pointer rounded-full",
                            onclick: move |_| {
                                is_fading_out.set(true);
                            },
                            img { width: 10, src: ERR_ICON }
                        }
                    }
                    div {
                        role: "alert",
                        class: "alert alert-error alert-outline flex justify-between items-center transition-all duration-300 ease-in-out",
                        p { class: "flex break-all", "{error_message().unwrap_or_default()}" }
                        div {
                            class: "radial-progress flex flex-shrink-0",
                            style: "--value:{progress()}; --size:1.5rem; --thickness:2px;",
                            aria_valuenow: "{progress()}",
                            role: "progressbar",
                        }
                    }
                }
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
    let mut description = use_signal(String::new);
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
        let des = if description().is_empty() {
            None
        } else {
            Some(description())
        };
        if let Some(path) = path() {
            match state.add_update_bibliography(&name(), path, des) {
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
            div { class: "modal-box w-1/2",
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
                    button { class: "cursor-pointer", onclick: select_file, "选取文件" }
                }
                br {}
                label { class: "input",
                    "描述"
                    input {
                        class: "grow",
                        r#type: "text",
                        placeholder: "可选",
                        value: "{description}",
                        oninput: move |event| {
                            description.set(event.data.value());
                        },
                    }
                }
                if let Some(error) = error_message() {
                    div { role: "alert", class: "alert alert-error", "{error}" }
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

#[component]
pub fn BibliographyTable() -> Element {
    let mut error_message = use_context::<Signal<Option<String>>>();
    let navigator = use_navigator();
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
                    info.path.as_os_str().to_str().unwrap().to_string(),
                    info.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    info.description.clone(),
                    info.path.exists(),
                )
            })
            .collect::<Vec<_>>()
    });

    let mut open_bib = move |path: String| {
        error_message.set(None);
        match parse(&path) {
            Ok(bib) => {
                let refs = read_bibliography(bib);
                let mut current_ref = CURRENT_REF.write();
                *current_ref = Some(refs);
                navigator.push(Route::References {});
            }
            Err(e) => {
                error_message.set(Some(format!("解析文件失败: {e}")));
            }
        }
    };

    let mut delete_bib = move |bib_name: String| {
        let mut state = STATE.write();
        state.remove_bibliography(&bib_name);
        let result = state.update_file();
        if let Err(e) = result {
            error_message.set(Some(e.to_string()));
        }
    };

    let mut open_bib_file = move |path: String| {
        let result = opener::open(&path);
        if let Err(e) = result {
            error_message.set(Some(e.to_string()));
        }
    };

    rsx! {
        div {
            div { class: "overflow-x-auto rounded-box border border-base-content/5 bg-base-100",
                table { class: "table",
                    thead {
                        tr {
                            th { class: "text-center", "name" }
                            th { class: "text-center", "path" }
                            th { class: "text-center", "description" }
                            th { class: "text-center", "time" }
                            th { class: "text-center", "action" }
                        }
                    }
                    tbody {
                        for (name , path , path_clone , updated_at , description , is_exist) in pairs() {
                            tr {
                                td { class: "text-center break-all",
                                    if is_exist {
                                        div { class: "inline-grid *:[grid-area:1/1]",
                                            div { class: "status status-success animate-ping" }
                                            div { class: "status status-success" }
                                        }
                                    } else {
                                        div {
                                            class: "tooltip tooltip-error tooltip-right cursor-pointer",
                                            "data-tip": "文件不存在",
                                            div { class: "inline-grid *:[grid-area:1/1]",
                                                div { class: "status status-error animate-ping" }
                                                div { class: "status status-error" }
                                            }
                                        }
                                    }
                                    span { class: "ml-1", "{name}" }
                                }
                                td { class: "text-center break-all",
                                    a {
                                        class: "link tooltip",
                                        "data-tip": "以默认应用程序打开 {path}",
                                        onclick: move |_| open_bib_file(path.clone()),
                                        "{abbr_path(&path, 40)}"
                                    }
                                }
                                td { class: "text-center break-all",
                                    if let Some(description) = description {
                                        "{description}"
                                    } else {
                                        ""
                                    }
                                }
                                td { class: "text-center break-all", "{updated_at}" }
                                td { class: "text-center break-all",
                                    div { class: "flex w-full",
                                        div { class: "grid grow place-items-center",
                                            button {
                                                class: "cursor-pointer btn btn-sm btn-ghost",
                                                onclick: move |_| open_bib(path_clone.clone()),
                                                "View"
                                            }
                                        }
                                        div { class: "grid grow place-items-center",
                                            button {
                                                class: "tooltip cursor-pointer",
                                                "data-tip": "delete",
                                                onclick: move |_| delete_bib(name.clone()),
                                                img {
                                                    alt: "delete",
                                                    width: 20,
                                                    src: DELETE_ICON,
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
