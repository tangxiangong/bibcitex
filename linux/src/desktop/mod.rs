//! Linux tray and global shortcuts; xpaste owns cross-application input.
mod x11;
use crate::Command;
use futures_util::StreamExt;
use ksni::TrayMethods;

#[derive(Clone)]
pub struct Desktop {
    language: async_channel::Sender<bool>,
    paste: xpaste::linux::Paste,
}

impl Desktop {
    pub fn start(events: async_channel::Sender<Command>, chinese: bool) -> Result<Self, String> {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok_and(|display| !display.is_empty());
        let errors = events.clone();
        let paste = xpaste::linux::Paste::start("io.github.tangxiangong.bibcitex", move |error| {
            let _ = errors.try_send(Command::Error(error));
        })?;
        if !wayland {
            x11::start(events.clone())?;
        }
        let portal_registration = paste.portal_registration();
        let (language, receiver) = async_channel::unbounded();
        std::thread::spawn(move || {
            smol::block_on(async move {
                if let Err(error) = &portal_registration {
                    let _ = events.send(Command::Error(error.clone())).await;
                }
                if wayland && portal_registration.is_ok() {
                    let events = events.clone();
                    smol::spawn(async move {
                        if let Err(error) = shortcuts(events.clone()).await {
                            let _ = events
                                .send(Command::Error(format!("Global shortcut: {error}")))
                                .await;
                        }
                    })
                    .detach();
                }
                let tray = Tray {
                    events: events.clone(),
                    chinese,
                };
                let handle = match tray.spawn().await {
                    Ok(handle) => Some(handle),
                    Err(error) => {
                        let _ = events
                            .send(Command::Error(format!("System tray: {error}")))
                            .await;
                        None
                    }
                };
                while let Ok(chinese) = receiver.recv().await {
                    if let Some(handle) = &handle {
                        handle.update(|tray| tray.chinese = chinese).await;
                    }
                }
            });
        });
        Ok(Self { language, paste })
    }
    pub fn capture(&self, external: bool) -> Result<(), String> {
        self.paste.capture(external)
    }
    pub fn captured_application_name(&self) -> Option<String> {
        self.paste.captured_application_name()
    }
    pub fn language(&self, chinese: bool) {
        let _ = self.language.try_send(chinese);
    }
    pub async fn prepare(&self) -> Result<(), String> {
        self.paste.prepare().await
    }
    pub async fn paste(&self) -> Result<(), String> {
        self.paste.paste().await
    }
}

async fn shortcuts(events: async_channel::Sender<Command>) -> Result<(), String> {
    use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
    let portal = GlobalShortcuts::new().await.map_err(|e| e.to_string())?;
    let session = portal
        .create_session(Default::default())
        .await
        .map_err(|e| e.to_string())?;
    let result = async {
        let mut activated = portal
            .receive_activated()
            .await
            .map_err(|e| e.to_string())?;
        let response = portal
            .bind_shortcuts(
                &session,
                &[NewShortcut::new("helper", "BibCiTeX").preferred_trigger("CTRL+SHIFT+k")],
                None,
                Default::default(),
            )
            .await
            .map_err(|e| e.to_string())?
            .response()
            .map_err(|e| e.to_string())?;
        if !response.shortcuts().iter().any(|s| s.id() == "helper") {
            return Err("The desktop did not grant the BibCiTeX shortcut".into());
        }
        while let Some(event) = activated.next().await {
            if event.shortcut_id() == "helper" {
                let command = event
                    .options()
                    .get("activation_token")
                    .and_then(|value| <&str>::try_from(value).ok())
                    .map(|token| Command::HelperToken(token.to_owned()))
                    .unwrap_or(Command::Helper);
                events.send(command).await.map_err(|e| e.to_string())?;
            }
        }
        Err("The global shortcut session ended".into())
    }
    .await;
    let _ = session.close().await;
    result
}

struct Tray {
    events: async_channel::Sender<Command>,
    chinese: bool,
}
impl ksni::Tray for Tray {
    // Left click activates the workbench; the host renders Menu on right click.
    const MENU_ON_ACTIVATE: bool = false;
    fn id(&self) -> String {
        "io.github.tangxiangong.bibcitex".into()
    }
    fn title(&self) -> String {
        "BibCiTeX".into()
    }
    fn icon_name(&self) -> String {
        // Use the dedicated tray artwork, not the desktop application icon.
        String::new()
    }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let Ok(icon) = image::load_from_memory_with_format(
            include_bytes!("../../../assets/app-icons/tray.ico"),
            image::ImageFormat::Ico,
        ) else {
            return vec![];
        };
        let icon = icon
            .resize_exact(32, 32, image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        let mut data = Vec::with_capacity(32 * 32 * 4);
        for pixel in icon.pixels() {
            data.extend_from_slice(&[pixel[3], pixel[0], pixel[1], pixel[2]]);
        }
        vec![ksni::Icon {
            width: 32,
            height: 32,
            data,
        }]
    }
    fn activate(&mut self, x: i32, y: i32) {
        let _ = self.events.try_send(Command::TrayAt { x, y });
    }
    fn watcher_online(&self) {
        let _ = self.events.try_send(Command::TrayAvailable(true));
    }
    fn watcher_offline(&self, _: ksni::OfflineReason) -> bool {
        let _ = self.events.try_send(Command::TrayAvailable(false));
        true
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        [
            ("显示窗口", "Show window", Command::Main),
            ("快捷助手", "Quick helper", Command::Helper),
            ("检查更新", "Check for updates", Command::CheckUpdates),
            ("退出 BibCiTeX", "Quit BibCiTeX", Command::Quit),
        ]
        .into_iter()
        .map(|(zh, en, command)| {
            ksni::menu::StandardItem {
                label: if self.chinese { zh } else { en }.into(),
                activate: Box::new(move |tray: &mut Self| {
                    let _ = tray.events.try_send(command.clone());
                }),
                ..Default::default()
            }
            .into()
        })
        .enumerate()
        .flat_map(|(index, item)| {
            if index == 3 {
                vec![ksni::MenuItem::Separator, item]
            } else {
                vec![item]
            }
        })
        .collect()
    }
}
