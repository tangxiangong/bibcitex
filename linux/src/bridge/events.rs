//! Deliver native events on the QuickJS engine thread after native calls return.
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
};

use nsh_core::prelude::*;

use crate::Command;

/// Category of a notification sent by native services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum DesktopEventKind {
    /// The library registry or helper selection changed.
    Libraries,
    /// The shared tag catalog or its assignments changed.
    Tags,
    /// Persisted settings changed.
    Preferences,
    /// An external trigger requested a desktop action.
    Activate,
    /// A Linux StatusNotifier watcher appeared or disappeared.
    TrayAvailability,
    /// The global shortcut binding changed.
    ShortcutChanged,
    /// A paste failed and manual copy is available.
    PasteFailed,
    /// A desktop backend reported an error.
    DesktopError,
}

/// A window or application action requested by a desktop integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum DesktopAction {
    /// Show the main workbench.
    Main,
    /// Capture the target and show the compact helper.
    Helper,
    /// Toggle the tray workbench.
    Tray,
    /// Show preferences.
    Settings,
    /// Show the about panel.
    About,
    /// Terminate the application.
    Quit,
    /// Open settings and check for updates.
    CheckUpdates,
}

/// An ordered desktop notification. Activation and paste failure events carry their payload.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct DesktopEvent {
    /// Monotonic process-local sequence, encoded as decimal text.
    pub sequence: String,
    /// libraries, tags, preferences, activate, trayAvailability, shortcutChanged, pasteFailed or desktopError.
    pub kind: DesktopEventKind,
    /// Target surface or command for an activation event.
    pub surface: Option<DesktopAction>,
    /// Compositor activation token, when supplied by Wayland.
    pub token: Option<String>,
    /// Tray anchor's horizontal screen coordinate when supplied by the status notifier.
    pub x: Option<i32>,
    /// Tray anchor's vertical screen coordinate when supplied by the status notifier.
    pub y: Option<i32>,
    /// Whether the tray watcher is currently available.
    pub available: Option<bool>,
    /// Human-readable portal binding or native error message.
    pub message: Option<String>,
    /// Paste ticket retained for manual copying after failure.
    pub ticket: Option<String>,
}

impl DesktopEvent {
    fn of(kind: DesktopEventKind) -> Self {
        Self {
            sequence: String::new(),
            kind,
            surface: None,
            token: None,
            x: None,
            y: None,
            available: None,
            message: None,
            ticket: None,
        }
    }
}

#[derive(Default)]
struct EventBus {
    sequence: u64,
    next_listener: u64,
    listeners: BTreeMap<u64, Callback<DesktopEvent>>,
    pending: VecDeque<DesktopEvent>,
    tray_available: Option<bool>,
}

thread_local! {
    static BUS: RefCell<EventBus> = RefCell::default();
}

pub(super) fn sequence() -> String {
    BUS.with_borrow(|bus| bus.sequence.to_string())
}

#[cfg(target_os = "linux")]
pub(super) fn tray_available() -> bool {
    BUS.with_borrow(|bus| bus.tray_available.unwrap_or(false))
}

fn publish(mut event: DesktopEvent) {
    let listeners = BUS.with_borrow_mut(|bus| {
        bus.sequence += 1;
        event.sequence = bus.sequence.to_string();
        if event.kind == DesktopEventKind::TrayAvailability {
            bus.tray_available = event.available;
        }
        // Snapshot reads recover data invalidations. Imperative commands must
        // survive the interval between engine startup and the first subscription.
        if bus.listeners.is_empty()
            && matches!(
                event.kind,
                DesktopEventKind::Activate
                    | DesktopEventKind::PasteFailed
                    | DesktopEventKind::DesktopError
                    | DesktopEventKind::ShortcutChanged
            )
        {
            if bus.pending.len() == 64 {
                bus.pending.pop_front();
            }
            bus.pending.push_back(event.clone());
        }
        bus.listeners.values().cloned().collect::<Vec<_>>()
    });
    for listener in listeners {
        listener.post(event.clone());
    }
}

pub(crate) fn emit(kind: &str) {
    let kind = match kind {
        "libraries" => DesktopEventKind::Libraries,
        "tags" => DesktopEventKind::Tags,
        "preferences" => DesktopEventKind::Preferences,
        _ => {
            debug_assert!(false, "Unexpected state invalidation: {kind}");
            return;
        }
    };
    publish(DesktopEvent::of(kind));
}

pub(crate) fn paste_failed(ticket: String, message: String) {
    publish(DesktopEvent {
        ticket: Some(ticket),
        message: Some(message),
        ..DesktopEvent::of(DesktopEventKind::PasteFailed)
    });
}

pub(super) fn listen(listener: Callback<DesktopEvent>) -> Stop {
    let (id, pending) = BUS.with_borrow_mut(|bus| {
        bus.next_listener += 1;
        let id = bus.next_listener;
        bus.listeners.insert(id, listener.clone());
        (id, std::mem::take(&mut bus.pending))
    });
    for event in pending {
        listener.post(event);
    }
    Stop::new(move || {
        BUS.with_borrow_mut(|bus| {
            bus.listeners.remove(&id);
        })
    })
}

pub(super) fn dispatch(command: Command) {
    let mut event = DesktopEvent::of(DesktopEventKind::Activate);
    let surface = match command {
        Command::Main => DesktopAction::Main,
        Command::MainToken(token) => {
            event.token = Some(token);
            DesktopAction::Main
        }
        Command::Helper => DesktopAction::Helper,
        Command::HelperToken(token) => {
            event.token = Some(token);
            DesktopAction::Helper
        }
        Command::Tray => DesktopAction::Tray,
        Command::TrayAt { x, y } => {
            event.x = Some(x);
            event.y = Some(y);
            DesktopAction::Tray
        }
        Command::Settings => DesktopAction::Settings,
        Command::About => DesktopAction::About,
        Command::Quit => DesktopAction::Quit,
        Command::CheckUpdates => DesktopAction::CheckUpdates,
        Command::RefreshLibraries => return emit("libraries"),
        Command::Error(message) => {
            event.kind = DesktopEventKind::DesktopError;
            event.message = Some(message);
            return publish(event);
        }
        Command::TrayAvailable(available) => {
            event.kind = DesktopEventKind::TrayAvailability;
            event.available = Some(available);
            return publish(event);
        }
        Command::ShortcutChanged => {
            event.kind = DesktopEventKind::ShortcutChanged;
            #[cfg(target_os = "linux")]
            {
                event.message = super::desktop::linux_desktop()
                    .ok()
                    .map(|d| d.shortcut_description());
            }
            return publish(event);
        }
        Command::PasteFailed { error, key } => {
            event.kind = DesktopEventKind::PasteFailed;
            event.message = Some(error);
            event.ticket = Some(key);
            return publish(event);
        }
    };
    event.surface = Some(surface);
    publish(event);
}
