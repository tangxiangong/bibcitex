use crate::components::updater::*;
use dioxus::prelude::*;
use semver::Version;
use updater::{Updater, UpdaterBuilder};

#[derive(Debug, Clone)]
pub struct AvaliableUpdater {
    pub updater: Updater,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub enum UpdaterStatus {
    Checking,
    Available,
    Download,
    UpToDate,
    Failed,
}

pub async fn check_for_update() -> Result<Option<AvaliableUpdater>, String> {
    let current_version =
        Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| format!("解析当前版本失败: {e}"))?;
    let updater = UpdaterBuilder::new("BibCiteX", current_version, "tangxiangong", "bibcitex")
        .build()
        .map_err(|e| format!("创建更新器失败: {e}"))?;
    match updater.check().await {
        Ok(Some(updater)) => {
            let size = updater.asset_size().unwrap();
            Ok(Some(AvaliableUpdater { updater, size }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(format!("更新检查失败: {e}")),
    }
}

#[component]
pub fn UpdateWindow(show_window: Signal<bool>) -> Element {
    let check_update_resource = use_resource(|| async { check_for_update().await });
    let check_update = use_context_provider(|| check_update_resource);
    let mut updater_info = use_context_provider(|| Signal::new(None::<AvaliableUpdater>));
    let mut error_message = use_signal(|| None::<String>);
    let mut status = use_context_provider(|| Signal::new(UpdaterStatus::Checking));

    use_effect(move || match check_update() {
        Some(Ok(Some(avaliable_updater))) => {
            status.set(UpdaterStatus::Available);
            updater_info.set(Some(avaliable_updater));
        }
        Some(Ok(None)) => {
            status.set(UpdaterStatus::UpToDate);
        }
        Some(Err(e)) => {
            status.set(UpdaterStatus::Failed);
            error_message.set(Some(e));
        }
        None => {
            status.set(UpdaterStatus::Checking);
        }
    });

    rsx! {
        div { class: "p-4",
            match status() {
                UpdaterStatus::Checking => rsx! {
                    Checking { show_window }
                },
                UpdaterStatus::Available => rsx! {
                    Available { show_window }
                },
                UpdaterStatus::Download => rsx! {
                    Download { show_window }
                },
                UpdaterStatus::UpToDate => rsx! {
                    UpToDate { show_window }
                },
                UpdaterStatus::Failed => rsx! {
                    Failed { error_message, show_window }
                },
            }
        }
    }
}
