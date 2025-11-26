use crate::{
    ADD_ICON, CURRENT_REF, DELETE_ICON, ERR_ICON, OK_ICON, STATE,
    route::Route,
    views::{get_helper_bib, set_helper_bib},
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
        div { class: "relative container mx-auto p-6",
            div { class: "flex items-center justify-between mb-8",
                div {
                    h2 { class: "text-3xl font-bold gradient-text", "Bibliographies" }
                    p { class: "text-base-content/60 text-sm mt-1", "管理你的文献库" }
                }
                button { class: "btn btn-modern gap-2", onclick: open_modal,
                    img {
                        width: 16,
                        src: ADD_ICON,
                        class: "invert brightness-0",
                    }
                    "新建文献库"
                }
            }
            BibliographyTable {}
            if show_error() {
                div { class: if is_fading_out() { "absolute top-2 right-2 w-1/3 z-50 animate-fade-out" } else { "absolute top-2 right-2 w-1/3 z-50 animate-fade-in" },
                    div {
                        role: "alert",
                        class: "alert alert-error shadow-lg backdrop-blur-md bg-error/10 border-error/20 flex justify-between items-center",
                        div { class: "flex items-center gap-2",
                            img { width: 20, src: ERR_ICON }
                            span { class: "font-medium", "{error_message().unwrap_or_default()}" }
                        }
                        div {
                            class: "radial-progress text-error text-xs",
                            style: "--value:{progress()}; --size:1.2rem; --thickness:2px;",
                            aria_valuenow: "{progress()}",
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
            "请选择 .bib 文件".to_string()
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
            .set_title("选择文献库文件")
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
        div { class: if show() { "modal modal-open backdrop-blur-sm" } else { "modal" },
            div { class: "modal-box w-1/2 max-w-2xl glass-panel shadow-2xl",
                h3 { class: "text-2xl font-bold mb-6 gradient-text", "新增文献库" }

                div { class: "form-control w-full mb-4",
                    label { class: "label",
                        span { class: "label-text font-medium", "文献库名称" }
                    }
                    label { class: "input input-bordered flex items-center gap-2 focus-within:input-primary transition-colors",
                        input {
                            class: "grow",
                            r#type: "text",
                            placeholder: "",
                            value: "{name}",
                            oninput: move |e| {
                                name.set(e.data.value());
                            },
                        }
                        if !name().is_empty() {
                            if name_is_valid() {
                                img {
                                    width: 20,
                                    src: OK_ICON,
                                    class: "opacity-50",
                                }
                            } else {
                                img { width: 20, src: ERR_ICON }
                            }
                        }
                    }
                    if !name().is_empty() && !name_is_valid() {
                        div { class: "label",
                            span { class: "label-text-alt text-error", "名称已存在" }
                        }
                    }
                }

                div { class: "form-control w-full mb-4",
                    label { class: "label",
                        span { class: "label-text font-medium", "文件路径" }
                    }
                    div { class: "join w-full",
                        input {
                            class: "input input-bordered join-item grow focus:outline-none cursor-default bg-base-200/50",
                            r#type: "text",
                            placeholder: "未选择文件",
                            value: "{path_abbr_string}",
                            readonly: true,
                            title: "{path_string}",
                        }
                        button { class: "btn join-item", onclick: select_file, "🔍" }
                    }
                }

                div { class: "form-control w-full mb-6",
                    label { class: "label",
                        span { class: "label-text font-medium", "描述" }
                        span { class: "label-text-alt text-base-content/50", "可选" }
                    }
                    textarea {
                        class: "textarea textarea-bordered h-24 focus:textarea-primary transition-colors resize-none",
                        placeholder: "请输入该文献库的简要说明...",
                        value: "{description}",
                        oninput: move |event| {
                            description.set(event.data.value());
                        },
                    }
                }

                if let Some(error) = error_message() {
                    div {
                        role: "alert",
                        class: "alert alert-error mb-4 shadow-sm",
                        img { width: 20, src: ERR_ICON }
                        span { "{error}" }
                    }
                }

                div { class: "modal-action mt-8",
                    button {
                        class: "btn btn-ghost hover:bg-base-content/10",
                        onclick: close_modal,
                        "取消"
                    }
                    button {
                        class: if save_available() { "btn btn-primary btn-modern px-8" } else { "btn btn-disabled px-8" },
                        onclick: save,
                        disabled: !save_available(),
                        "添加"
                    }
                }
            }
            div { class: "modal-backdrop bg-base-300/30", onclick: close_modal }
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
                    name.clone(),
                    info.path.as_os_str().to_str().unwrap().to_string(),
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
        if let Some((helper_bib_name, _)) = get_helper_bib()
            && helper_bib_name == bib_name
        {
            set_helper_bib(None);
        }
    };

    let mut open_bib_file = move |path: String| {
        let result = opener::open(&path);
        if let Err(e) = result {
            error_message.set(Some(e.to_string()));
        }
    };

    rsx! {
        div { class: "w-full",
            if pairs().is_empty() {
                div { class: "flex flex-col items-center justify-center h-64 text-base-content/50",
                    div { class: "text-4xl mb-4", "📚" }
                    p { class: "text-lg", "未找到文献库" }
                    p { class: "text-sm", "点击 + 按钮添加一个文献库" }
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 p-4",
                    for (name , name_clone , path , path_clone , path_clone_2 , updated_at , description , is_exist) in pairs() {
                        div { class: "card-modern card-shine group relative overflow-hidden flex flex-col h-full min-h-[200px] transition-all duration-500 hover:-translate-y-2 hover:shadow-primary/10 border-white/5",
                            // Decorative Background Elements
                            div { class: "absolute -top-20 -right-20 w-40 h-40 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-all duration-700 animate-blob" }
                            div { class: "absolute -bottom-20 -left-20 w-40 h-40 bg-secondary/5 rounded-full blur-3xl group-hover:bg-secondary/10 transition-all duration-700 animate-blob animation-delay-2000" }
                            div { class: "absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-full h-full bg-linear-to-br from-white/0 via-white/5 to-white/0 opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none" }

                            div { class: "card-body p-6 flex-1 relative z-10 backdrop-blur-[2px]",
                                // Header
                                div { class: "flex items-start justify-between mb-4",
                                    div { class: "flex items-center gap-3 overflow-hidden",
                                        div { class: "w-10 h-10 rounded-xl bg-linear-to-br from-primary/10 to-secondary/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-500 shadow-inner border border-white/10",
                                            span { class: "text-2xl", "📚" }
                                        }
                                        div {
                                            h3 {
                                                class: "text-xl font-bold gradient-text truncate leading-tight",
                                                title: "{name}",
                                                "{name}"
                                            }
                                            if is_exist {
                                                div { class: "flex items-center gap-1 mt-1",
                                                    div { class: "w-1.5 h-1.5 rounded-full bg-success animate-pulse" }
                                                    span { class: "text-[10px] uppercase tracking-wider font-bold text-success/80",
                                                        "可用"
                                                    }
                                                }
                                            } else {
                                                div { class: "flex items-center gap-1 mt-1",
                                                    div { class: "w-1.5 h-1.5 rounded-full bg-error animate-pulse" }
                                                    span { class: "text-[10px] uppercase tracking-wider font-bold text-error/80",
                                                        "缺失"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Content
                                div { class: "flex-1 pl-1",
                                    if let Some(desc) = description {
                                        p {
                                            class: "text-base-content/70 text-sm mb-6 line-clamp-2 font-light leading-relaxed",
                                            title: "{desc}",
                                            "{desc}"
                                        }
                                    } else {
                                        p { class: "text-base-content/30 text-sm mb-6 italic font-light",
                                            "暂无描述"
                                        }
                                    }

                                    div { class: "flex flex-col gap-3 text-xs text-base-content/60",
                                        div { class: "flex items-center gap-2 group/link",
                                            span { class: "opacity-50 group-hover/link:text-primary transition-colors",
                                                "📂"
                                            }
                                            a {
                                                class: "link link-hover truncate hover:text-primary transition-colors font-mono bg-base-200/50 px-2 py-1 rounded-md w-full text-left border border-transparent hover:border-primary/20 hover:bg-primary/5",
                                                onclick: move |_| open_bib_file(path_clone_2.clone()),
                                                title: "{path}",
                                                "{abbr_path(&path, 35)}"
                                            }
                                        }
                                        div { class: "flex items-center gap-2",
                                            span { class: "opacity-50", "🕒" }
                                            span { class: "font-mono opacity-80", "{updated_at}" }
                                        }
                                    }
                                }

                                // Actions overlay (visible on hover or always visible but styled)
                                div { class: "absolute bottom-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 translate-y-2 group-hover:translate-y-0 transition-all duration-300",
                                    button {
                                        class: "btn btn-sm btn-circle btn-ghost text-error hover:bg-error/10 tooltip tooltip-left shadow-sm border border-transparent hover:border-error/20",
                                        "data-tip": "删除",
                                        onclick: move |_| delete_bib(name_clone.clone()),
                                        img {
                                            src: DELETE_ICON,
                                            width: 14,
                                            class: "opacity-70",
                                        }
                                    }
                                    button {
                                        class: "btn btn-sm btn-primary shadow-lg shadow-primary/30 hover:shadow-primary/50 border-none animate-gradient-x bg-linear-to-r from-primary to-secondary text-white gap-2 px-4 rounded-full",
                                        onclick: move |_| open_bib(path_clone.clone()),
                                        span { "打开" }
                                        span { class: "group-hover:translate-x-1 transition-transform",
                                            "→"
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
