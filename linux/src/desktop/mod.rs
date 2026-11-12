//! Linux tray and global shortcuts; xpaste owns cross-application input.
mod x11;
use crate::Command;
use ashpd::desktop::{
    Session,
    global_shortcuts::{GlobalShortcuts, NewShortcut},
};
use futures_util::StreamExt;
use ksni::TrayMethods;
use std::sync::{Arc, Mutex};

pub(super) struct ShortcutRequest {
    value: String,
    response: async_channel::Sender<Result<ShortcutChange, String>>,
}

/// Retains the old X11 grab until the new value has been persisted and committed.
#[must_use]
pub struct ShortcutChange {
    pub value: String,
    decision: Option<async_channel::Sender<ShortcutDecision>>,
}

pub(super) enum ShortcutDecision {
    Commit(async_channel::Sender<()>),
    Cancel,
}

impl ShortcutChange {
    pub async fn commit(mut self) -> Result<(), String> {
        if let Some(decision) = self.decision.take() {
            let (response, result) = async_channel::bounded(1);
            decision
                .send(ShortcutDecision::Commit(response))
                .await
                .map_err(|_| "error.desktopUnavailable".to_owned())?;
            result
                .recv()
                .await
                .map_err(|_| "error.desktopUnavailable".to_owned())?;
        }
        Ok(())
    }
}

impl Drop for ShortcutChange {
    fn drop(&mut self) {
        if let Some(decision) = &self.decision {
            let _ = decision.try_send(ShortcutDecision::Cancel);
        }
    }
}

enum TrayRequest {
    Language(bool),
    Enabled(bool, async_channel::Sender<Result<(), String>>),
}

#[derive(Clone)]
pub struct Desktop {
    tray: async_channel::Sender<TrayRequest>,
    shortcuts: async_channel::Sender<ShortcutRequest>,
    shortcut_description: Arc<Mutex<String>>,
    pub wayland: bool,
    paste: xpaste::linux::Paste,
}

impl Desktop {
    pub fn start(
        events: async_channel::Sender<Command>,
        chinese: bool,
        tray_enabled: bool,
        shortcut: &str,
    ) -> Result<Self, String> {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok_and(|display| !display.is_empty());
        let errors = events.clone();
        let paste = xpaste::linux::Paste::start("io.github.tangxiangong.bibcitex", move |error| {
            let _ = errors.try_send(Command::Error(error));
        })?;
        let (shortcuts, shortcut_requests) = async_channel::unbounded();
        let shortcut_description = Arc::new(Mutex::new(String::new()));
        if !wayland
            && let Err(error) = x11::start(events.clone(), shortcut, shortcut_requests.clone())
        {
            let _ = events.try_send(Command::Error(error));
        }
        let portal_registration = paste.portal_registration();
        let (tray, receiver) = async_channel::unbounded();
        let description = shortcut_description.clone();
        std::thread::spawn(move || {
            smol::block_on(async move {
                if let Err(error) = &portal_registration {
                    let _ = events.send(Command::Error(error.clone())).await;
                }
                if wayland && portal_registration.is_ok() {
                    let events = events.clone();
                    smol::spawn(async move {
                        if let Err(error) =
                            portal_shortcuts(events.clone(), shortcut_requests, description).await
                        {
                            let _ = events
                                .send(Command::Error(format!("Global shortcut: {error}")))
                                .await;
                        }
                    })
                    .detach();
                } else {
                    drop(shortcut_requests);
                }
                let mut handle = None;
                let mut chinese = chinese;
                if tray_enabled {
                    match (Tray {
                        events: events.clone(),
                        chinese,
                    })
                    .spawn()
                    .await
                    {
                        Ok(tray) => handle = Some(tray),
                        Err(error) => {
                            let _ = events
                                .send(Command::Error(format!("System tray: {error}")))
                                .await;
                        }
                    }
                }
                while let Ok(request) = receiver.recv().await {
                    match request {
                        TrayRequest::Language(value) => {
                            chinese = value;
                            if let Some(handle) = &handle {
                                handle.update(|tray| tray.chinese = chinese).await;
                            }
                        }
                        TrayRequest::Enabled(enabled, response) => {
                            let result =
                                if enabled && handle.as_ref().is_none_or(ksni::Handle::is_closed) {
                                    match (Tray {
                                        events: events.clone(),
                                        chinese,
                                    })
                                    .spawn()
                                    .await
                                    {
                                        Ok(tray) => {
                                            handle = Some(tray);
                                            Ok(())
                                        }
                                        Err(error) => Err(error.to_string()),
                                    }
                                } else {
                                    if !enabled && let Some(tray) = handle.take() {
                                        tray.shutdown().await;
                                        let _ = events.send(Command::TrayAvailable(false)).await;
                                    }
                                    Ok(())
                                };
                            let _ = response.send(result).await;
                        }
                    }
                }
                if let Some(tray) = handle {
                    tray.shutdown().await;
                }
            });
        });
        Ok(Self {
            tray,
            shortcuts,
            shortcut_description,
            wayland,
            paste,
        })
    }
    pub fn capture(&self, external: bool) -> Result<(), String> {
        self.paste.capture(external)
    }
    pub fn captured_application_name(&self) -> Option<String> {
        self.paste.captured_application_name()
    }
    pub fn language(&self, chinese: bool) {
        let _ = self.tray.try_send(TrayRequest::Language(chinese));
    }
    pub async fn set_tray_enabled(&self, enabled: bool) -> Result<(), String> {
        let (response, result) = async_channel::bounded(1);
        self.tray
            .send(TrayRequest::Enabled(enabled, response))
            .await
            .map_err(|e| e.to_string())?;
        result.recv().await.map_err(|e| e.to_string())?
    }
    pub fn shortcut_description(&self) -> String {
        self.shortcut_description
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }
    pub async fn prepare_shortcut(&self, value: String) -> Result<ShortcutChange, String> {
        let (response, result) = async_channel::bounded(1);
        self.shortcuts
            .send(ShortcutRequest { value, response })
            .await
            .map_err(|_| "error.desktopUnavailable".to_owned())?;
        result
            .recv()
            .await
            .map_err(|_| "error.desktopUnavailable".to_owned())?
    }
    pub async fn prepare(&self) -> Result<(), String> {
        self.paste.prepare().await
    }
    pub async fn paste(&self) -> Result<(), String> {
        self.paste.paste().await
    }
}

async fn portal_shortcuts(
    events: async_channel::Sender<Command>,
    requests: async_channel::Receiver<ShortcutRequest>,
    description: Arc<Mutex<String>>,
) -> Result<(), String> {
    use futures_util::future::{Either, select};
    let portal = GlobalShortcuts::new().await.map_err(|e| e.to_string())?;
    let mut session = None;
    let result = async {
        let mut activated = portal
            .receive_activated()
            .await
            .map_err(|e| e.to_string())?;
        let mut changed = portal
            .receive_shortcuts_changed()
            .await
            .map_err(|e| e.to_string())?;
        match bind_portal_shortcut(&portal).await {
            Ok((bound, label)) => {
                session = Some(bound);
                shortcut_description(&description, &label, &events).await;
            }
            Err(error) => {
                let _ = events
                    .send(Command::Error(format!("Global shortcut: {error}")))
                    .await;
            }
        }
        loop {
            let request = requests.recv();
            let activation = activated.next();
            let change = changed.next();
            futures_util::pin_mut!(request, activation, change);
            match select(request, select(activation, change)).await {
                Either::Left((Ok(request), _)) => {
                    let result = match &session {
                        Some(_) if portal.version() < 2 => {
                            Err("error.helperShortcutPortalVersion".into())
                        }
                        Some(session) => portal
                            .configure_shortcuts(session, None, Default::default())
                            .await
                            .map_err(|e| e.to_string()),
                        None => match bind_portal_shortcut(&portal).await {
                            Ok((bound, label)) => {
                                session = Some(bound);
                                shortcut_description(&description, &label, &events).await;
                                Ok(())
                            }
                            Err(error) => Err(error),
                        },
                    };
                    let _ = request
                        .response
                        .send(result.map(|_| ShortcutChange {
                            value: String::new(),
                            decision: None,
                        }))
                        .await;
                }
                Either::Right((Either::Left((Some(event), _)), _))
                    if session.is_some() && event.shortcut_id() == "helper" =>
                {
                    let command = event
                        .options()
                        .get("activation_token")
                        .and_then(|value| <&str>::try_from(value).ok())
                        .map(|token| Command::HelperToken(token.to_owned()))
                        .unwrap_or(Command::Helper);
                    events.send(command).await.map_err(|e| e.to_string())?;
                }
                Either::Right((Either::Right((Some(event), _)), _)) if session.is_some() => {
                    let label = event
                        .shortcuts()
                        .iter()
                        .find(|s| s.id() == "helper")
                        .map(|s| s.trigger_description())
                        .unwrap_or("");
                    shortcut_description(&description, label, &events).await;
                }
                Either::Left((Err(_), _))
                | Either::Right((Either::Left((None, _)) | Either::Right((None, _)), _)) => break,
                _ => {}
            }
        }
        Err("The global shortcut session ended".into())
    }
    .await;
    if let Some(session) = session {
        let _ = session.close().await;
    }
    result
}

async fn bind_portal_shortcut(
    portal: &GlobalShortcuts,
) -> Result<(Session<GlobalShortcuts>, String), String> {
    let session = portal
        .create_session(Default::default())
        .await
        .map_err(|e| e.to_string())?;
    let result = async {
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
        response
            .shortcuts()
            .iter()
            .find(|s| s.id() == "helper")
            .map(|s| s.trigger_description().to_owned())
            .ok_or_else(|| "The desktop did not grant the BibCiTeX shortcut".to_owned())
    }
    .await;
    match result {
        Ok(label) => Ok((session, label)),
        Err(error) => {
            let _ = session.close().await;
            Err(error)
        }
    }
}

async fn shortcut_description(
    description: &Mutex<String>,
    label: &str,
    events: &async_channel::Sender<Command>,
) {
    if let Ok(mut value) = description.lock() {
        *value = label.into();
    }
    let _ = events.send(Command::ShortcutChanged).await;
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
