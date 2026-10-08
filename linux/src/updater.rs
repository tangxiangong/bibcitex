use crate::preferences::Preferences;
use bibcitex_linux::updates::{self, Release};
use gpui_kit::*;
use std::time::Duration;

pub struct Updater {
    pub busy: bool,
    pub checked: bool,
    pub installed: bool,
    pub release: Option<Release>,
    pub error: Option<String>,
    pub channel: String,
    task: Option<Task<()>>,
}
impl Updater {
    pub fn start(preferences: Entity<Preferences>, cx: &mut App) -> Entity<Self> {
        let updater = cx.new(|_| Self {
            busy: false,
            checked: false,
            installed: false,
            release: None,
            error: None,
            channel: String::new(),
            task: None,
        });
        let weak = updater.downgrade();
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_secs(10))
                .await;
            loop {
                if weak
                    .update(cx, |updater, cx| updater.check(preferences.clone(), cx))
                    .is_err()
                {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_secs(12 * 60 * 60))
                    .await;
            }
        })
        .detach();
        updater
    }
    pub fn check(&mut self, preferences: Entity<Preferences>, cx: &mut Context<Self>) {
        if self.busy || self.installed {
            return;
        }
        self.busy = true;
        self.error = None;
        self.release = None;
        self.checked = false;
        self.channel = preferences.read(cx).update_channel.clone();
        let channel = self.channel.clone();
        let job = cx
            .background_executor()
            .spawn(async move { updates::check(env!("CARGO_PKG_VERSION"), &channel) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                if this.channel != preferences.read(cx).update_channel {
                    this.error = Some("更新设置已更改，请重新检查更新".into());
                    cx.notify();
                    return;
                }
                this.checked = true;
                match result {
                    Ok(release) => this.release = release,
                    Err(error) => this.error = Some(error),
                }
                if this.release.is_some()
                    && preferences.read(cx).automatic_updates
                    && std::env::current_exe().is_ok_and(|exe| !updates::managed_install(&exe))
                {
                    this.install(cx);
                }
                cx.notify();
            });
        }));
        cx.notify();
    }
    pub fn install(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let Some(release) = self.release.clone() else {
            return;
        };
        self.busy = true;
        self.error = None;
        let job = cx
            .background_executor()
            .spawn(async move { updates::install(&release) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(updates::InstallOutcome::Replaced(path)) => {
                        cx.set_restart_path(path);
                        this.installed = true;
                    }
                    Ok(updates::InstallOutcome::Package(path)) => cx.open_with_system(&path),
                    Err(error) => this.error = Some(error),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }
}
