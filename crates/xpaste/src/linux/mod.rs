//! Linux paste target tracking and authorized cross-application keyboard input.
//!
//! Clipboard ownership stays with the host's long-lived clipboard provider. Prepare
//! before hiding the host window, then paste after the compositor restores focus.
mod x11;

use ashpd::desktop::{
    Session,
    remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions},
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

type Reply = async_channel::Sender<Result<(), String>>;
enum Request {
    Prepare(Reply),
    Paste(Reply),
}

/// A shared Linux paste session, independent of the desktop UI and shortcuts.
#[derive(Clone)]
pub struct Paste {
    requests: async_channel::Sender<Request>,
    target: Option<Arc<Mutex<x11::Target>>>,
    external: Arc<AtomicBool>,
    portal_registration: Result<(), String>,
}

impl Paste {
    /// Observe X11 targets or register the host identity for Wayland portals.
    pub fn start(
        app_id: &str,
        report_error: impl Fn(String) + Send + 'static,
    ) -> Result<Self, String> {
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok_and(|display| !display.is_empty());
        let target = if wayland {
            None
        } else {
            Some(x11::start(report_error)?)
        };
        let portal_registration = if wayland {
            // A window app_id does not identify ASHPD's D-Bus connection. Register
            // before either GlobalShortcuts or RemoteDesktop is used.
            match app_id.parse::<ashpd::AppID>() {
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
        let registration = portal_registration.clone();
        std::thread::spawn(move || {
            smol::block_on(async move {
                let mut remote: Option<(RemoteDesktop, Session<RemoteDesktop>)> = None;
                while let Ok(request) = receiver.recv().await {
                    match request {
                        Request::Prepare(reply) => {
                            let result = if let Some(target) = &backend {
                                x11::validate(target)
                            } else if let Err(error) = &registration {
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
                if let Some((_, session)) = remote {
                    let _ = session.close().await;
                }
            });
        });
        Ok(Self {
            requests,
            target,
            external,
            portal_registration,
        })
    }

    /// Portal identity status, also used by the host's global shortcut setup.
    pub fn portal_registration(&self) -> Result<(), String> {
        self.portal_registration.clone()
    }

    /// Snapshot the recent external X11 target or arm Wayland paste after an external activation.
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

    /// The captured X11 application's display name, when available.
    pub fn captured_application_name(&self) -> Option<String> {
        self.target.as_ref()?.lock().ok()?.captured_name.clone()
    }

    /// Validate the target or request portal keyboard access before the host hides its window.
    pub async fn prepare(&self) -> Result<(), String> {
        let (send, receive) = async_channel::bounded(1);
        self.requests
            .send(Request::Prepare(send))
            .await
            .map_err(|e| e.to_string())?;
        receive.recv().await.map_err(|e| e.to_string())?
    }

    /// Restore and verify the X11 target, or inject through the authorized Wayland session.
    pub async fn paste(&self) -> Result<(), String> {
        let (send, receive) = async_channel::bounded(1);
        self.requests
            .send(Request::Paste(send))
            .await
            .map_err(|e| e.to_string())?;
        receive.recv().await.map_err(|e| e.to_string())?
    }
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
