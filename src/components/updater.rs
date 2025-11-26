use crate::{
    LOGO,
    views::{AvaliableUpdater, UpdaterStatus},
};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn Checking(show_window: Signal<bool>) -> Element {
    let cancle = move |_| {
        let mut check_update = use_context::<Resource<()>>();
        check_update.cancel();
        show_window.set(false);
    };
    rsx! {
        div { class: "card-modern glass-panel p-8 mx-auto max-w-md",
            div { class: "flex flex-col items-center justify-center mb-6",
                img {
                    width: 80,
                    height: 80,
                    src: LOGO,
                    class: "mb-4 drop-shadow-lg",
                }
                h3 { class: "text-xl font-bold gradient-text animate-pulse", "正在检查更新..." }
            }
            div { class: "mb-8 w-full",
                progress { class: "progress progress-primary w-full h-2" }
            }
            div { class: "flex justify-center",
                button {
                    class: "btn btn-ghost text-base-content/70 hover:text-error",
                    onclick: cancle,
                    "取消"
                }
            }
        }
    }
}

#[component]
pub fn Available(show_window: Signal<bool>) -> Element {
    let updater_info = use_context::<Signal<Option<AvaliableUpdater>>>();
    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = updater_info
        .unwrap()
        .updater
        .latest_version()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let download_size = updater_info.unwrap().size;
    rsx! {
        div { class: "card-modern glass-panel p-8 mx-auto max-w-md",
            div { class: "flex flex-col items-center justify-center mb-6",
                img {
                    width: 80,
                    height: 80,
                    src: LOGO,
                    class: "mb-4 drop-shadow-lg",
                }
                h3 { class: "text-2xl font-bold gradient-text mb-2", "发现新版本!" }
                div { class: "badge badge-primary badge-soft", "v{latest_version}" }
            }

            div { class: "bg-base-200/50 rounded-lg p-4 mb-6 text-sm space-y-2",
                div { class: "flex justify-between",
                    span { class: "text-base-content/60", "当前版本" }
                    span { class: "font-mono", "v{current_version}" }
                }
                div { class: "flex justify-between",
                    span { class: "text-base-content/60", "更新大小" }
                    span { class: "font-mono", "{format_bytes(download_size)}" }
                }
            }

            div { class: "flex justify-end gap-3",
                button {
                    class: "btn btn-ghost text-base-content/70 hover:bg-base-200",
                    onclick: move |_| show_window.set(false),
                    "稍后"
                }
                button {
                    class: "btn btn-primary btn-soft shadow-lg hover:scale-105 transition-transform",
                    onclick: move |_| {
                        let mut status = use_context::<Signal<UpdaterStatus>>();
                        status.set(UpdaterStatus::Download);
                    },
                    "立即下载"
                }
            }
        }
    }
}

#[component]
pub fn Download(show_window: Signal<bool>) -> Element {
    let updater_info = use_context::<Signal<Option<AvaliableUpdater>>>();
    let updater = Arc::new(updater_info.unwrap().updater);
    let updater_clone = updater.clone();
    let updater_for_launch = updater.clone();
    let size = updater_info.unwrap().size;
    let mut total_download = use_signal(|| 0u64);
    let mut downloaded = use_signal(|| false);
    let download_progress = use_memo(move || total_download() as f64 / size as f64);
    let mut error_message = use_signal(|| None::<String>);
    let mut download_buffer = use_signal(Vec::new);
    let mut download_failed = use_signal(|| false);
    let mut preinstall_failed = use_signal(|| false);
    let mut preinstall = use_signal(|| false);
    let mut install_failed = use_signal(|| false);
    let mut download = use_future(move || {
        let updater_clone = updater_clone.clone();
        async move {
            match updater_clone
                .download(|chunk_size| {
                    total_download.set(total_download() + chunk_size as u64);
                })
                .await
            {
                Ok(data) => {
                    download_buffer.set(data);
                    downloaded.set(true);
                }
                Err(e) => {
                    download_failed.set(true);
                    error_message.set(Some(format!("下载失败: {e}")));
                }
            }
        }
    });

    use_effect(move || {
        let updater = updater.clone();
        if downloaded() {
            match updater.install(download_buffer()) {
                Ok(_) => {
                    preinstall.set(true);
                }
                Err(e) => {
                    preinstall_failed.set(true);
                    error_message.set(Some(format!("处理安装包失败: {e}")));
                }
            }
        }
    });

    let cancle = move |_| {
        download.cancel();
        show_window.set(false);
    };

    let relaunch = {
        let updater = updater_for_launch.clone();
        move |_| match updater.relaunch() {
            Ok(_) => {
                show_window.set(false);
            }
            Err(e) => {
                install_failed.set(true);
                error_message.set(Some(e.to_string()));
            }
        }
    };

    let title = use_memo(move || {
        let progress = download_progress();
        if downloaded() {
            "下载完成".to_string()
        } else if download_failed() {
            "下载失败".to_string()
        } else {
            format!("正在下载更新... {:.0}%", progress * 100.0)
        }
    });

    rsx! {
        div { class: "card-modern glass-panel p-8 mx-auto max-w-md",
            div { class: "flex flex-col items-center justify-center mb-6",
                img {
                    width: 80,
                    height: 80,
                    src: LOGO,
                    class: "mb-4 drop-shadow-lg",
                }
                h3 { class: "text-xl font-bold gradient-text", "{title}" }
            }

            div { class: "mb-6 w-full space-y-2",
                if !downloaded() {
                    progress {
                        class: "progress progress-primary w-full h-3",
                        value: "{total_download}",
                        max: "{size}",
                    }
                    div { class: "flex justify-between text-xs text-base-content/50 px-1",
                        span { "{format_bytes(total_download())} / {format_bytes(size)}" }
                        span { "{ (download_progress() * 100.0) as u64 }%" }
                    }
                    if download_failed() {
                        div { class: "alert alert-error text-sm mt-4 shadow-lg",
                            span { "{error_message().unwrap_or_default()}" }
                        }
                    }
                } else {
                    div { class: "flex flex-col items-center gap-4 py-4",
                        if !preinstall() {
                            span { class: "loading loading-spinner loading-md text-primary" }
                            p { class: "text-base-content/70", "正在准备安装..." }
                            if preinstall_failed() {
                                div { class: "alert alert-error text-sm shadow-lg",
                                    span { "准备安装失败: {error_message().unwrap_or_default()}" }
                                }
                            }
                        } else {
                            div { class: "text-success text-5xl mb-2 animate-bounce",
                                "✓"
                            }
                            p { class: "text-lg font-medium", "下载完成" }
                            p { class: "text-sm text-base-content/60",
                                "准备就绪，请重启应用以完成更新"
                            }
                        }
                    }
                }
            }

            if install_failed() {
                div { class: "alert alert-error text-sm mb-4 shadow-lg",
                    span { "安装失败: {error_message().unwrap_or_default()}" }
                }
            }

            div { class: "flex justify-center gap-3",
                if downloaded() && preinstall() {
                    button {
                        class: "btn btn-success btn-soft shadow-lg hover:scale-105 transition-transform w-full",
                        onclick: relaunch,
                        "立即重启"
                    }
                } else {
                    button {
                        class: "btn btn-ghost text-base-content/70 hover:text-error",
                        onclick: cancle,
                        "取消下载"
                    }
                }
            }
        }
    }
}

#[component]
pub fn UpToDate(show_window: Signal<bool>) -> Element {
    let current_version = env!("CARGO_PKG_VERSION");
    rsx! {
        div { class: "card-modern glass-panel p-8 mx-auto max-w-md text-center",
            div { class: "flex justify-center mb-6",
                img {
                    width: 80,
                    height: 80,
                    src: LOGO,
                    class: "drop-shadow-lg",
                }
            }
            h3 { class: "text-2xl font-bold gradient-text mb-2", "已是最新版本" }
            p { class: "text-base-content/60 mb-8", "BibCiTeX v{current_version} 目前是最新的" }

            button {
                class: "btn btn-primary btn-soft w-full",
                onclick: move |_| show_window.set(false),
                "太棒了"
            }
        }
    }
}

#[component]
pub fn Failed(error_message: Signal<Option<String>>, show_window: Signal<bool>) -> Element {
    let error_msg = error_message().unwrap_or_else(|| "未知错误".into());

    rsx! {
        div { class: "card-modern glass-panel p-8 mx-auto max-w-md text-center border-t-4 border-error",
            div { class: "flex justify-center mb-6",
                img {
                    width: 80,
                    height: 80,
                    src: LOGO,
                    class: "grayscale opacity-50",
                }
            }
            h3 { class: "text-xl font-bold text-error mb-4", "检查更新失败" }
            div { class: "bg-error/10 text-error p-4 rounded-lg mb-8 text-sm text-left",
                "{error_msg}"
            }

            div { class: "flex justify-end gap-3",
                button {
                    class: "btn btn-ghost hover:bg-base-200",
                    onclick: move |_| show_window.set(false),
                    "关闭"
                }
                button {
                    class: "btn btn-error btn-soft shadow-lg",
                    onclick: move |_| {
                        let mut check_update = use_context::<Resource<()>>();
                        let mut status = use_context::<Signal<UpdaterStatus>>();
                        status.set(UpdaterStatus::Checking);
                        check_update.restart();
                    },
                    "重试"
                }
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}
