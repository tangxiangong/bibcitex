use crate::{CANCEL_ICON, OK_ICON, UPDATE_MODAL_OPEN};
use dioxus::prelude::*;
use semver::Version;
use updater::{Updater, UpdaterBuilder};

// Update status enum
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum UpdateStatus {
    Checking,
    Available {
        updater: Updater,
        size: u64,
    },
    Downloading {
        downloaded: u64,
        total: u64,
        progress: f32,
    },
    Downloaded {
        updater: Updater,
        update_data: Vec<u8>,
    },
    Installing,
    UpToDate,
    Error(String),
}

async fn check_for_updates() -> Result<Option<(Updater, u64)>, String> {
    let current_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|e| format!("解析当前版本失败: {}", e))?;

    println!("[更新检查] 开始检查更新，当前版本: {}", current_version);

    let updater = UpdaterBuilder::new("BibCiteX", current_version, "tangxiangong", "bibcitex")
        .build()
        .map_err(|e| format!("创建更新器失败: {}", e))?;

    println!("[更新检查] 正在向 GitHub API 发送请求...");

    match updater.check().await {
        Ok(Some(updater)) => {
            let latest_version = updater.latest_version().unwrap();
            let size = updater.asset_size().unwrap();
            println!(
                "[更新检查] GitHub API 请求成功，发现远程版本: {}",
                latest_version
            );

            let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
                .map_err(|e| format!("解析当前版本失败: {}", e))?;
            if latest_version > current_version {
                println!(
                    "[更新检查] ✅ 发现新版本: {} > {}，大小: {:?}",
                    latest_version,
                    current_version,
                    format_bytes(size)
                );
                Ok(Some((updater, size)))
            } else {
                println!(
                    "[更新检查] ℹ️ 已是最新版本: {} <= {}",
                    latest_version, current_version
                );
                Ok(None)
            }
        }
        Ok(None) => {
            println!("[更新检查] ℹ️ GitHub API 请求成功，但未找到任何发布版本");
            Ok(None)
        }
        Err(e) => {
            println!("[更新检查] ❌ 请求失败: {}", e);
            Err(format!(
                "更新检查失败: {}\n\n可能的原因:\n• 网络连接问题\n• GitHub API 限制\n• 仓库不存在或无权限访问",
                e
            ))
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

#[component]
pub fn UpdateWindow() -> Element {
    let close_modal = move || {
        *UPDATE_MODAL_OPEN.write() = false;
    };

    let mut update_resource = use_resource(|| async { check_for_updates().await });
    let mut is_retrying = use_signal(|| false);

    let mut download_state = use_signal(|| None::<(u64, u64, f32)>);
    let mut download_data = use_signal(|| None::<(Updater, Vec<u8>)>);
    let mut error_message = use_signal(|| None::<String>);
    let mut total_downloaded = use_signal(|| 0u64);
    let mut installing_state = use_signal(|| false);

    let current_status = match update_resource.read().as_ref() {
        Some(Ok(Some((updater, size)))) => {
            // Reset retrying state when we get a successful result
            if *is_retrying.read() {
                is_retrying.set(false);
            }
            if *installing_state.read() {
                UpdateStatus::Installing
            } else if let Some((downloaded_update, data)) = download_data.read().as_ref() {
                UpdateStatus::Downloaded {
                    updater: downloaded_update.clone(),
                    update_data: data.clone(),
                }
            } else if let Some((downloaded, total, progress)) = download_state.read().as_ref() {
                UpdateStatus::Downloading {
                    downloaded: *downloaded,
                    total: *total,
                    progress: *progress,
                }
            } else {
                UpdateStatus::Available {
                    updater: updater.clone(),
                    size: *size,
                }
            }
        }
        Some(Ok(None)) => {
            // Reset retrying state when we get a successful result
            if *is_retrying.read() {
                is_retrying.set(false);
            }
            UpdateStatus::UpToDate
        }
        Some(Err(e)) => {
            // Don't reset retrying state immediately when we get an error result
            // Let the user see the loading animation
            UpdateStatus::Error(e.clone())
        }
        None => {
            if let Some(err) = error_message.read().as_ref() {
                UpdateStatus::Error(err.clone())
            } else {
                UpdateStatus::Checking
            }
        }
    };

    rsx! {
        h2 { class: "text-lg font-bold p-4", "软件更新" }
        div { class: "p-4",
            match current_status {
                UpdateStatus::Checking => rsx! {
                    div { class: "text-center space-y-6",
                        div { class: "flex justify-center mb-6",
                            div { class: "w-20 h-20 bg-gray-800 rounded-2xl flex items-center justify-center",
                                div { class: "w-12 h-8 bg-green-500 rounded flex items-center justify-center",
                                    div { class: "flex space-x-1",
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                    }
                                }
                            }
                        }
                        div { class: "flex justify-center mb-4",
                            div { class: "animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500" }
                        }
                        div { class: "space-y-2 mb-8",
                            h3 { class: "text-lg font-semibold text-gray-800 leading-tight", "正在检查更新..." }
                            div { class: "text-sm text-gray-600 space-y-1",
                                p { "• 当前版本: {env!(\"CARGO_PKG_VERSION\")}" }
                                p { "• 正在连接 GitHub API..." }
                                p { "• 检查仓库: tangxiangong/bibcitex" }
                            }
                        }
                    }
                },
                UpdateStatus::Available { updater, size } => rsx! {
                    div { class: "text-center space-y-6",
                        div { class: "flex justify-center mb-6",
                            div { class: "w-20 h-20 bg-gray-800 rounded-2xl flex items-center justify-center",
                                div { class: "w-12 h-8 bg-blue-500 rounded flex items-center justify-center",
                                    div { class: "flex space-x-1",
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                    }
                                }
                            }
                        }
                        div { class: "space-y-2 mb-8",
                            h3 { class: "text-lg font-semibold text-gray-800 leading-tight", "发现新版本" }
                            div { class: "text-sm text-gray-600 space-y-1",
                                p { "• 当前版本: {env!(\"CARGO_PKG_VERSION\")}" }
                                p { "• 最新版本: {updater.latest_version().unwrap()}" }
                                p { "• 下载大小: {format_bytes(size)}" }
                                p { "• 请求成功，找到更新版本" }
                            }
                        }
                    }
                    div { class: "modal-action p-3",
                        button { class: "btn btn-soft btn-error", onclick: move |_| close_modal(),
                            img { width: 20, src: CANCEL_ICON }
                            "稍后"
                        }
                        button {
                            class: "btn btn-soft btn-success",
                            onclick: move |_| {
                                let updater_clone = updater.clone();
                                // 重置下载状态
                                total_downloaded.set(0);
                                spawn(async move {
                                    match updater_clone
                                        .download(
                                            |chunk_size, total_size| {
                                                let current_downloaded = total_downloaded();
                                                let new_downloaded = current_downloaded + chunk_size as u64;
                                                total_downloaded.set(new_downloaded);



                                                let progress = (new_downloaded as f64 / total_size as f64)
                                                    as f32;
                                                download_state.set(Some((new_downloaded, total_size, progress)));
                                            },
                                            || {},
                                        )
                                        .await
                                    {
                                        Ok(data) => {
                                            download_data.set(Some((updater_clone.clone(), data)));
                                            download_state.set(None);
                                        }
                                        Err(e) => {
                                            error_message.set(Some(format!("Download failed: {}", e)));
                                            download_state.set(None);
                                        }
                                    }
                                });
                            },
                            img { width: 20, src: OK_ICON }
                            "立即更新"
                        }
                    }
                },
                UpdateStatus::Downloading { progress, downloaded, total } => {
                    rsx! {
                        div { class: "text-center space-y-6",
                            div { class: "flex justify-center mb-6",
                                div { class: "w-20 h-20 bg-gray-800 rounded-2xl flex items-center justify-center",
                                    div { class: "w-12 h-8 bg-blue-500 rounded flex items-center justify-center",
                                        div { class: "flex space-x-1",
                                            div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                            div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                        }
                                    }
                                }
                            }
                            div { class: "flex justify-center mb-4",
                                div { class: "animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500" }
                            }
                            div { class: "space-y-1 mb-6",
                                h3 { class: "text-lg font-semibold text-gray-800 leading-tight", "正在下载更新..." }
                                p { class: "text-sm text-gray-600",
                                    "{(progress * 100.0) as u32}% 完成 ({format_bytes(downloaded)} / {format_bytes(total)})"
                                }
                            }
                            div { class: "w-full bg-gray-200 rounded-full h-2 mb-8",
                                div {
                                    class: "bg-blue-500 h-full rounded-full transition-all duration-300",
                                    style: "width: {progress * 100.0}%",
                                }
                            }
                        }
                    }
                }
                UpdateStatus::Downloaded { updater, update_data } => {
                    let data_for_restart = update_data.clone();
                    rsx! {
                        div { class: "text-center space-y-6",
                            div { class: "flex justify-center mb-6",
                                div { class: "w-20 h-20 bg-gray-800 rounded-2xl flex items-center justify-center",
                                    div { class: "w-12 h-8 bg-green-500 rounded flex items-center justify-center",
                                        div { class: "flex space-x-1",
                                            div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                            div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                        }
                                    }
                                }
                            }
                            div { class: "space-y-1 mb-8",
                                h3 { class: "text-lg font-semibold text-gray-800 leading-tight",
                                    "Update downloaded successfully."
                                }
                                p { class: "text-sm text-gray-600", "Restart the application to complete the update." }
                            }
                        }
                        div { class: "modal-action p-3",
                            button { class: "btn btn-soft btn-error", onclick: move |_| close_modal(),
                                img { width: 20, src: CANCEL_ICON }
                                "稍后"
                            }
                            button {
                                class: "btn btn-soft btn-success",
                                onclick: move |_| {
                                    let data = data_for_restart.clone();
                                    let update_clone = updater.clone();
                                    spawn(async move {
                                        eprintln!("Installing update with {} bytes", data.len());

                                        installing_state.set(true);

                                        match update_clone.install(&data) {
                                            Ok(_) => {
                                                eprintln!("Update installed successfully, exiting application...");
                                                std::process::exit(0);
                                            }
                                            Err(e) => {
                                                eprintln!("Installation failed: {}", e);
                                                error_message.set(Some(format!("Installation failed: {}", e)));
                                                installing_state.set(false);
                                            }
                                        }
                                    });
                                },
                                img { width: 20, src: OK_ICON }
                                "立即重启"
                            }
                        }
                    }
                }
                UpdateStatus::Installing => rsx! {
                    div { class: "text-center space-y-6",
                        div { class: "flex justify-center mb-6",
                            div { class: "w-20 h-20 bg-gray-800 rounded-2xl flex items-center justify-center",
                                div { class: "w-12 h-8 bg-green-500 rounded flex items-center justify-center",
                                    div { class: "flex space-x-1",
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                    }
                                }
                            }
                        }
                        div { class: "space-y-1 mb-8",
                            h3 { class: "text-lg font-semibold text-gray-800 leading-tight", "Installing update..." }
                            p { class: "text-sm text-gray-600", "Please wait while the update is being installed." }
                        }
                        div { class: "flex justify-center",
                            div { class: "flex space-x-1",
                                div {
                                    class: "w-2 h-2 bg-blue-500 rounded-full animate-bounce",
                                    style: "animation-delay: 0ms",
                                }
                                div {
                                    class: "w-2 h-2 bg-blue-500 rounded-full animate-bounce",
                                    style: "animation-delay: 150ms",
                                }
                                div {
                                    class: "w-2 h-2 bg-blue-500 rounded-full animate-bounce",
                                    style: "animation-delay: 300ms",
                                }
                            }
                        }
                    }
                },
                UpdateStatus::UpToDate => rsx! {
                    div { class: "text-center space-y-6",
                        div { class: "flex justify-center mb-6",
                            div { class: "w-20 h-20 bg-gray-800 rounded-2xl flex items-center justify-center",
                                div { class: "w-12 h-8 bg-green-500 rounded flex items-center justify-center",
                                    div { class: "flex space-x-1",
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                    }
                                }
                            }
                        }
                        div { class: "space-y-2 mb-8",
                            h3 { class: "text-lg font-semibold text-gray-800 leading-tight", "已是最新版本" }
                            div { class: "text-sm text-gray-600 space-y-1",
                                p { "• 当前版本: {env!(\"CARGO_PKG_VERSION\")}" }
                                p { "• 检查完成，未发现新版本" }
                                p { "• 您正在使用最新版本" }
                            }
                        }
                        button {
                            class: "w-full bg-blue-500 hover:bg-blue-600 text-white font-medium py-3 px-6 rounded-lg transition-colors duration-200",
                            onclick: move |_| {
                                close_modal();
                            },
                            "OK"
                        }
                    }
                },
                UpdateStatus::Error(error_msg) => rsx! {
                    div { class: "text-center space-y-6",
                        div { class: "flex justify-center mb-6",
                            div { class: "w-20 h-20 bg-gray-800 rounded-2xl flex items-center justify-center",
                                div { class: "w-12 h-8 bg-red-500 rounded flex items-center justify-center",
                                    div { class: "flex space-x-1",
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                        div { class: "w-2 h-2 bg-gray-800 rounded-sm" }
                                    }
                                }
                            }
                        }
                        div { class: "space-y-2 mb-8",
                            h3 { class: "text-lg font-semibold text-gray-800 leading-tight", "检查更新失败" }
                            div { class: "text-sm text-gray-600 space-y-1",
                                p { "• 请求失败: {error_msg}" }
                                p { "• 请检查网络连接" }
                                p { "• 或稍后重试" }
                            }
                        }
                    }
                    div { class: "modal-action p-3",
                        button { class: "btn btn-soft btn-error", onclick: move |_| close_modal(),
                            img { width: 20, src: CANCEL_ICON }
                            "取消"
                        }
                        button {
                            class: if *is_retrying.read() { "btn btn-soft btn-success loading" } else { "btn btn-soft btn-success" },
                            disabled: *is_retrying.read(),
                            onclick: move |_| {
                                is_retrying.set(true);
                                error_message.set(None);
                                update_resource.restart();

                                spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                                    is_retrying.set(false);
                                });
                            },
                            if *is_retrying.read() {
                                span { class: "loading loading-spinner loading-sm" }
                                "重试中..."
                            } else {
                                img { width: 20, src: OK_ICON }
                                "重试"
                            }
                        }
                    }
                },
            }
        }
    }
}
