//! Linux tray, global shortcuts, and authorized cross-application input.
mod x11;
use crate::Command;
use ashpd::desktop::{
    Session,
    remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions},
};
use futures_util::StreamExt;
use ksni::TrayMethods;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

type Reply = async_channel::Sender<Result<(), String>>;
enum Request {
    Prepare(Reply),
    Paste(Reply),
    Language(bool),
}
#[derive(Clone)]
pub struct Desktop {
    requests: async_channel::Sender<Request>,
    target: Option<Arc<Mutex<x11::Target>>>,
    external: Arc<AtomicBool>,
}

impl Desktop {
    pub fn start(events: async_channel::Sender<Command>, chinese: bool) -> Result<Self, String> {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok_and(|display| !display.is_empty());
        let target = if wayland {
            None
        } else {
            Some(x11::start(events.clone())?)
        };
        let portal_registration = if wayland {
            // GPUI's window app_id does not identify ASHPD's D-Bus connection.
            // Register before either GlobalShortcuts or RemoteDesktop is used.
            match "io.github.tangxiangong.bibcitex".parse::<ashpd::AppID>() {
                Ok(app_id) => smol::block_on(ashpd::register_host_app(app_id))
                    .map_err(|error| format!("Portal application registration: {error}")),
                Err(error) => Err(error.to_string()),
            }
        } else {
            Ok(())
        };
        let (requests, receiver) = async_channel::unbounded();
        let backend = target.clone();
        let external = Arc::new(AtomicBool::new(false));
        let allowed = external.clone();
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
                let mut remote: Option<(RemoteDesktop, Session<RemoteDesktop>)> = None;
                while let Ok(request) = receiver.recv().await {
                    match request {
                        Request::Language(chinese) => {
                            if let Some(handle) = &handle {
                                handle.update(|tray| tray.chinese = chinese).await;
                            }
                        }
                        Request::Prepare(reply) => {
                            let result = if let Some(target) = &backend {
                                x11::validate(target)
                            } else if let Err(error) = &portal_registration {
                                Err(error.clone())
                            } else if !allowed.load(Ordering::Acquire) {
                                Err("No previous application is available for paste".into())
                            } else if remote.is_some() {
                                Ok(())
                            } else {
                                match remote_session().await {
                                    Ok(session) => {
                                        remote = Some(session);
                                        Ok(())
                                    }
                                    Err(error) => Err(error),
                                }
                            };
                            let _ = reply.send(result).await;
                        }
                        Request::Paste(reply) => {
                            let result = if let Some(target) = &backend {
                                x11::paste(target)
                            } else if !allowed.swap(false, Ordering::AcqRel) {
                                Err("No previous application is available for paste".into())
                            } else if let Some((proxy, session)) = &remote {
                                inject(proxy, session).await
                            } else {
                                Err("Remote desktop permission is required".into())
                            };
                            if result.is_err()
                                && let Some((_, session)) = remote.take()
                            {
                                // Dropping ashpd's proxy does not close the portal session.
                                let _ = session.close().await;
                            }
                            let _ = reply.send(result).await;
                        }
                    }
                }
            });
        });
        Ok(Self {
            requests,
            target,
            external,
        })
    }
    pub fn capture(&self, external: bool) -> Result<(), String> {
        self.external.store(external, Ordering::Release);
        if let Some(target) = &self.target {
            x11::capture(target)
        } else if external {
            Ok(())
        } else {
            Err("No previous application is available for paste".into())
        }
    }
    pub fn language(&self, chinese: bool) {
        let _ = self.requests.try_send(Request::Language(chinese));
    }
    pub async fn prepare(&self) -> Result<(), String> {
        let (send, receive) = async_channel::bounded(1);
        self.requests
            .send(Request::Prepare(send))
            .await
            .map_err(|e| e.to_string())?;
        receive.recv().await.map_err(|e| e.to_string())?
    }
    pub async fn paste(&self) -> Result<(), String> {
        let (send, receive) = async_channel::bounded(1);
        self.requests
            .send(Request::Paste(send))
            .await
            .map_err(|e| e.to_string())?;
        receive.recv().await.map_err(|e| e.to_string())?
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

async fn remote_session() -> Result<(RemoteDesktop, Session<RemoteDesktop>), String> {
    let proxy = RemoteDesktop::new().await.map_err(|e| e.to_string())?;
    let session = proxy
        .create_session(Default::default())
        .await
        .map_err(|e| e.to_string())?;
    let result = async {
        proxy
            .select_devices(
                &session,
                SelectDevicesOptions::default().set_devices(Some(DeviceType::Keyboard.into())),
            )
            .await
            .map_err(|e| e.to_string())?
            .response()
            .map_err(|e| e.to_string())?;
        let response = proxy
            .start(&session, None, Default::default())
            .await
            .map_err(|e| e.to_string())?
            .response()
            .map_err(|e| e.to_string())?;
        if !response.devices().contains(DeviceType::Keyboard) {
            return Err("Keyboard access was not granted".into());
        }
        Ok::<_, String>(())
    }
    .await;
    if let Err(error) = result {
        let _ = session.close().await;
        return Err(error);
    }
    Ok((proxy, session))
}
async fn inject(proxy: &RemoteDesktop, session: &Session<RemoteDesktop>) -> Result<(), String> {
    // Keysyms let the compositor resolve the active keyboard layout.
    proxy
        .notify_keyboard_keysym(session, 0xffe3, KeyState::Pressed, Default::default())
        .await
        .map_err(|e| e.to_string())?;
    let press = proxy
        .notify_keyboard_keysym(session, 0x76, KeyState::Pressed, Default::default())
        .await;
    let release_v = proxy
        .notify_keyboard_keysym(session, 0x76, KeyState::Released, Default::default())
        .await;
    let release_ctrl = proxy
        .notify_keyboard_keysym(session, 0xffe3, KeyState::Released, Default::default())
        .await;
    press
        .and(release_v)
        .and(release_ctrl)
        .map_err(|e| e.to_string())
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
