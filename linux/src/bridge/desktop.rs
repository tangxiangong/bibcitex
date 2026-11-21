//! Process-level desktop services shared by all NSH windows.
use std::cell::Cell;

use nsh_core::prelude::*;

use super::{
    catalog::{Catalog, LibraryDto},
    events::{self, DesktopEvent},
    paste::{self, CapturedTarget, PasteOutcome},
};
use crate::preferences::{PreferencePatch, Preferences, PreferencesDto};

/// Desktop host family used to choose presentation and OS integrations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PropEnum)]
pub enum DesktopOs {
    /// The AppKit host.
    Macos,
    /// The Windows desktop host.
    Windows,
    /// The GTK host.
    Linux,
}

/// Identifies the process that owns global desktop integrations.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct DesktopPlatform {
    /// Operating system of this host.
    pub os: DesktopOs,
    /// Rust publishes the StatusNotifierItem on this host.
    pub tray_managed_by_rust: bool,
    /// Rust installs the portal or X11 shortcut on this host.
    pub shortcut_managed_by_rust: bool,
    /// The Wayland compositor owns shortcut editing through its settings portal.
    pub portal_shortcut: bool,
    /// Linux can composite per-pixel surface alpha. This flag does not report blur support.
    pub linux_translucency_supported: bool,
}

/// Services the client can use in its current desktop environment.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct DesktopCapabilities {
    /// A system tray backend is supported, including while its icon is disabled.
    pub tray: bool,
    /// A global shortcut backend can accept registration requests.
    pub shortcut: bool,
    /// A cross-application paste backend is available; each target still requires validation.
    pub paste: bool,
}

/// A coherent startup view of libraries, appearance, integration.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct Snapshot {
    /// Event sequence before reading the snapshot. Replay later notifications after loading it.
    pub revision: String,
    /// Registered bibliographies.
    pub libraries: Vec<LibraryDto>,
    /// Persisted settings and their independent revision.
    pub preferences: PreferencesDto,
    /// Desktop services available in this host.
    pub capabilities: DesktopCapabilities,
    /// Initial tray availability; on Linux a StatusNotifier watcher must be online.
    pub tray_available: bool,
    /// Host OS and ownership of global integrations.
    pub platform: DesktopPlatform,
    /// Human-readable binding selected by the Wayland global shortcut portal.
    pub shortcut_description: Option<String>,
}

/// One format in the citation settings list.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct CitationFormat {
    /// Stable identifier accepted by savePreferences.
    pub id: String,
    /// Original catalog label for localization.
    pub label: String,
    /// Presentation group: raw, latex or typst.
    pub group: String,
}

thread_local! {
    static STARTED: Cell<bool> = const {Cell::new(false)};
    static SAVING: Cell<bool> = const {Cell::new(false)};
    #[cfg(target_os="linux")]
    static LINUX: std::cell::RefCell<Option<crate::desktop::Desktop>> = const {std::cell::RefCell::new(None)};
}

#[cfg(target_os = "linux")]
pub(super) fn linux_desktop() -> Result<crate::desktop::Desktop, String> {
    LINUX
        .with_borrow(Clone::clone)
        .ok_or_else(|| "error.desktopUnavailable".into())
}

fn start_services(prefs: &Preferences) -> Result<bool, String> {
    #[cfg(not(target_os = "linux"))]
    let _ = prefs;
    if STARTED.get() {
        return Ok(true);
    }
    let (tx, rx) = async_channel::unbounded();
    if !Owner::detached().run(|| crate::platform::activation::start(&tx))? {
        nsh_core::element::desktop::quit();
        return Ok(false);
    }
    #[cfg(not(target_os = "linux"))]
    bibcitex_service::initialize();
    #[cfg(target_os = "linux")]
    match crate::desktop::Desktop::start(
        tx.clone(),
        super::support::resolved_language(&prefs.language) == "zh-Hans",
        prefs.tray_icon_enabled,
        &prefs.helper_shortcut,
    ) {
        Ok(desktop) => LINUX.with_borrow_mut(|slot| *slot = Some(desktop)),
        Err(error) => events::dispatch(crate::Command::Error(error)),
    };
    // The OS registrations and receiver belong to the process, including while
    // the root component is replaced by a hot reload.
    Owner::detached().run(|| {
        drop(spawn(async move {
            while let Ok(command) = rx.recv().await {
                events::dispatch(command);
            }
        }));
    });
    STARTED.set(true);
    Ok(true)
}

struct SaveInProgress;

impl SaveInProgress {
    fn enter() -> Result<Self, String> {
        if SAVING.replace(true) {
            Err("Another preferences update is in progress".into())
        } else {
            Ok(Self)
        }
    }
}

impl Drop for SaveInProgress {
    fn drop(&mut self) {
        SAVING.set(false);
    }
}

async fn load_preferences() -> Result<Preferences, String> {
    blocking(Preferences::load)
        .await
        .map_err(|e| e.to_string())?
}

async fn persist(mut prefs: Preferences, revision: u64) -> Result<Preferences, String> {
    blocking(move || {
        prefs.save_next(revision)?;
        Ok(prefs)
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn save(patch: PreferencePatch, expected_rev: String) -> Result<PreferencesDto, String> {
    let _write = SaveInProgress::enter()?;
    let expected: u64 = expected_rev
        .parse()
        .map_err(|_| "Invalid preferences revision")?;
    let before = load_preferences().await?;
    if before.revision != expected {
        return Err("Preferences changed; reload before saving".into());
    }
    let mut next = before.clone();
    #[cfg(target_os = "linux")]
    let (patch, configure_portal) = {
        let mut patch = patch;
        let configure = patch.helper_shortcut.is_some() && linux_desktop().is_ok_and(|d| d.wayland);
        if configure {
            patch.helper_shortcut = None;
        }
        (patch, configure)
    };
    patch.apply(&mut next)?;
    #[cfg(target_os = "linux")]
    let (desktop, shortcut, changed_tray) = {
        let changed_tray = before.tray_icon_enabled != next.tray_icon_enabled;
        let changed_key = configure_portal || before.helper_shortcut != next.helper_shortcut;
        let desktop = if changed_tray || changed_key {
            Some(linux_desktop()?)
        } else {
            linux_desktop().ok()
        };
        let mut shortcut = None;
        if let Some(d) = desktop.as_ref() {
            if changed_key {
                let prepared = d.prepare_shortcut(next.helper_shortcut.clone()).await?;
                // The Wayland portal controls its own trigger; the display label
                // comes back via ShortcutChanged and may differ from the request.
                if d.wayland {
                    next.helper_shortcut.clone_from(&before.helper_shortcut);
                } else {
                    next.helper_shortcut.clone_from(&prepared.value);
                }
                shortcut = Some(prepared);
            }
            if changed_tray {
                d.set_tray_enabled(next.tray_icon_enabled).await?;
            }
        }
        (desktop, shortcut, changed_tray)
    };
    let saved = persist(next, expected).await;
    #[cfg(target_os = "linux")]
    if saved.is_err()
        && changed_tray
        && let Some(d) = desktop.as_ref()
    {
        d.set_tray_enabled(before.tray_icon_enabled).await?;
    }
    let saved = saved?;
    #[cfg(target_os = "linux")]
    {
        if let Some(change) = shortcut
            && let Err(error) = change.commit().await
        {
            persist(before.clone(), saved.revision).await?;
            if changed_tray && let Some(d) = desktop.as_ref() {
                d.set_tray_enabled(before.tray_icon_enabled).await?;
            }
            events::emit("preferences");
            return Err(error);
        }
        if saved.language != before.language
            && let Some(d) = desktop.as_ref()
        {
            d.language(super::support::resolved_language(&saved.language) == "zh-Hans");
        }
    }
    events::emit("preferences");
    Ok(saved.into())
}

/// Shared access to desktop preferences, integration events, clipboard transactions.
#[class]
pub struct DesktopSession;

#[class]
impl DesktopSession {
    /// Creates a handle to the current process's desktop services.
    pub fn new() -> Self {
        Self
    }

    /// Starts desktop services once and reads the current state. Subscribe before awaiting it.
    pub async fn start(this: Instance<Self>) -> Result<Snapshot, String> {
        let _ = this;
        let revision = events::sequence();
        let prefs = load_preferences().await?;
        let primary = start_services(&prefs)?;
        let libraries = if primary {
            Catalog::libraries().await?
        } else {
            Vec::new()
        };
        #[cfg(target_os = "linux")]
        let (os, tray, backend) = (
            DesktopOs::Linux,
            events::tray_available(),
            linux_desktop().is_ok(),
        );
        #[cfg(target_os = "macos")]
        let (os, tray, backend) = (DesktopOs::Macos, prefs.tray_icon_enabled, true);
        #[cfg(target_os = "windows")]
        let (os, tray, backend) = (DesktopOs::Windows, prefs.tray_icon_enabled, true);
        #[cfg(target_os = "linux")]
        let (portal_shortcut, shortcut_description) = linux_desktop()
            .ok()
            .filter(|desktop| desktop.wayland)
            .map(|desktop| {
                (
                    true,
                    Some(desktop.shortcut_description()).filter(|value| !value.is_empty()),
                )
            })
            .unwrap_or((false, None));
        #[cfg(not(target_os = "linux"))]
        let (portal_shortcut, shortcut_description) = (false, None);
        let linux_translucency_supported =
            primary && crate::platform::translucency::supported().await;
        Ok(Snapshot {
            shortcut_description,
            revision,
            libraries,
            preferences: prefs.into(),
            capabilities: DesktopCapabilities {
                tray: backend,
                shortcut: backend,
                paste: backend,
            },
            tray_available: tray,
            platform: DesktopPlatform {
                os,
                tray_managed_by_rust: cfg!(target_os = "linux"),
                shortcut_managed_by_rust: cfg!(target_os = "linux"),
                portal_shortcut,
                linux_translucency_supported,
            },
        })
    }

    /// Subscribes on the engine thread. Calling the returned function stops delivery.
    pub fn on_change(&self, listener: Callback<DesktopEvent>) -> Stop {
        events::listen(listener)
    }

    /// The OS locale before NSH's app-language override, for the system preference.
    #[getter]
    pub fn system_locale(&self) -> String {
        sys_locale::get_locale().unwrap_or_else(|| "en".into())
    }

    /// Reads settings and the revision required to change them.
    pub async fn preferences(this: Instance<Self>) -> Result<PreferencesDto, String> {
        let _ = this;
        Ok(load_preferences().await?.into())
    }

    /// Applies only the requested settings after checking the expected preferences revision.
    pub async fn save_preferences(
        this: Instance<Self>,
        patch: PreferencePatch,
        expected_rev: String,
    ) -> Result<PreferencesDto, String> {
        let _ = this;
        save(patch, expected_rev).await
    }

    /// Lists all formats in their display order with their settings group.
    pub fn citation_formats(&self) -> Vec<CitationFormat> {
        bibcitex_service::CITATION_FORMATS
            .iter()
            .map(|&(id, label)| CitationFormat {
                id: id.into(),
                label: label.into(),
                group: if id == "raw" {
                    "raw"
                } else if id.starts_with("typst") {
                    "typst"
                } else {
                    "latex"
                }
                .into(),
            })
            .collect()
    }

    /// Captures the current external target before the helper takes focus.
    pub async fn capture_target(this: Instance<Self>) -> Result<CapturedTarget, String> {
        let _ = this;
        paste::capture_target().await
    }

    /// Finishes a prepared paste after hiding the helper.
    pub async fn commit_paste(
        this: Instance<Self>,
        ticket: String,
    ) -> Result<PasteOutcome, String> {
        let _ = this;
        paste::commit_paste(ticket).await
    }

    /// Cancels the target or pending paste associated with this ticket.
    pub fn cancel_paste(&self, ticket: String) {
        paste::cancel_paste(ticket);
    }

    /// Copies the exact citation retained by a failed paste operation.
    pub async fn copy_fallback(this: Instance<Self>, ticket: String) -> Result<(), String> {
        let _ = this;
        paste::copy_fallback(ticket).await
    }

    /// Copies user-selected text to the system clipboard.
    pub async fn copy_text(this: Instance<Self>, text: String) -> Result<(), String> {
        let _ = this;
        paste::copy_text(text).await
    }

    /// Reports whether the OS presently permits accessibility input for this app.
    pub fn accessibility_status(&self) -> bool {
        paste::accessibility_status()
    }

    /// Places the mapped tray window at its status-notifier anchor where the window system allows it.
    pub async fn place_tray(this: Instance<Self>, x: i32, y: i32) -> Result<bool, String> {
        let _ = this;
        crate::platform::tray_placement::place_tray(x, y).await
    }

    /// Consumes a compositor activation token for the requested visible surface.
    pub async fn activate_wayland(
        this: Instance<Self>,
        token: String,
        surface: String,
    ) -> Result<(), String> {
        let _ = this;
        crate::platform::activation::activate_wayland(&token, &surface).await
    }
}

impl Default for DesktopSession {
    fn default() -> Self {
        Self::new()
    }
}
