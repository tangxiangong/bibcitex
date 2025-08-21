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
        div { class: "rounded-lg p-6 mx-auto",
            div { class: "flex items-center justify-center mb-6",
                div { class: "mr-4",
                    img { width: 80, height: 80, src: LOGO }
                }
            }
            div { class: "text-center mb-4",
                h3 { "正在检查更新..." }
            }
            div { class: "mb-6 w-1/2 mx-auto",
                progress { class: "w-full progress" }
            }
            div { class: "flex justify-end",
                button { class: "btn btn-soft btn-error", onclick: cancle, "取消" }
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
        div { class: "rounded-lg p-6 mx-auto",
            div { class: "flex items-center justify-center mb-6",
                div { class: "mr-4",
                    img { width: 80, height: 80, src: LOGO }
                }
            }
            div { class: "text-center mb-4",
                h3 { class: "font-bold", "发现新的版本!" }
                p { class: "text-sm mt-2", "当前版本 v{current_version}" }
                p { class: "text-sm mt-2", "最新版本 v{latest_version}" }
                p { class: "text-sm mt-2", "更新大小 {format_bytes(download_size)}" }
            }

            div { class: "flex justify-end",
                button {
                    class: "btn btn-soft mr-2",
                    onclick: move |_| {
                        let mut status = use_context::<Signal<UpdaterStatus>>();
                        status.set(UpdaterStatus::Download);
                    },
                    "下载更新"
                }
                button {
                    class: "btn btn-soft btn-error",
                    onclick: move |_| show_window.set(false),
                    "关闭"
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
            format!("正在下载更新... {:.2}%", progress * 100.0)
        }
    });

    rsx! {
        div { class: "rounded-lg p-6 mx-auto",
            div { class: "flex items-center justify-center mb-6",
                div { class: "mr-4",
                    img { width: 80, height: 80, src: LOGO }
                }
            }
            div { class: "text-center mb-4",
                h3 { class: "mb-2", "{title}" }
                if !downloaded() {
                    div { class: "mb-6 w-1/2 mx-auto",
                        progress {
                            class: "w-full progress",
                            value: "{total_download}",
                            max: "{size}",
                        }
                    }
                    if download_failed() {
                        p { class: "text-sm mt-2 text-error", "{error_message().unwrap_or_default()}" }
                    }
                } else {
                    if !preinstall() {
                        p { "正在处理安装包..." }
                        if preinstall_failed() {
                            p { class: "text-sm mt-2 text-error",
                                "处理安装包失败 {error_message().unwrap_or_default()}"
                            }
                        }
                    } else {
                        p { "安装包处理完成, 请点击重新启动以安装" }
                    }
                }
            }

            if install_failed() {
                p { class: "text-center text-sm mt-2 text-error",
                    "安装失败 {error_message().unwrap_or_default()}"
                }
            }
            div { class: "flex justify-end",
                button {
                    class: if preinstall() { "mr-2 btn btn-soft btn-success" } else { "mr-2  btn btn-soft btn-disabled" },
                    disabled: !preinstall(),
                    onclick: relaunch,
                    "重新启动"
                }
                button { class: "btn btn-soft btn-error", onclick: cancle, "取消" }
            }
        }
    }
}

#[component]
pub fn UpToDate(show_window: Signal<bool>) -> Element {
    let current_version = env!("CARGO_PKG_VERSION");
    rsx! {
        div { class: "rounded-lg p-6 mx-auto",
            div { class: "flex items-center justify-center mb-6",
                div { class: "mr-4",
                    img { width: 80, height: 80, src: LOGO }
                }
            }
            div { class: "text-center mb-4",
                h3 { class: "font-bold", "您使用的就是最新版!" }
                p { class: "text-sm mt-2", "BibCiTeX v{current_version} 是当前的最新版本" }
            }

            div { class: "flex justify-end",
                button {
                    class: "btn btn-soft btn-error",
                    onclick: move |_| show_window.set(false),
                    "关闭"
                }
            }
        }
    }
}

#[component]
pub fn Failed(error_message: Signal<Option<String>>, show_window: Signal<bool>) -> Element {
    let error_msg = error_message().unwrap_or_else(|| "未知错误".into());

    rsx! {
        div { class: "rounded-lg p-6 mx-auto",
            div { class: "flex items-center justify-center mb-6",
                div { class: "mr-4",
                    img { width: 80, height: 80, src: LOGO }
                }
            }
            div { class: "text-center mb-4",
                h3 { class: "font-bold", "检查更新失败" }
                p { class: "text-sm mt-2 text-error", "{error_msg}" }
            }
            div { class: "flex justify-end",
                button {
                    class: "btn btn-soft mr-2",
                    onclick: move |_| {
                        let mut check_update = use_context::<Resource<()>>();
                        let mut status = use_context::<Signal<UpdaterStatus>>();
                        status.set(UpdaterStatus::Checking);
                        check_update.restart();
                    },
                    "重试"
                }
                button {
                    class: "btn btn-soft btn-error",
                    onclick: move |_| show_window.set(false),
                    "关闭"
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
