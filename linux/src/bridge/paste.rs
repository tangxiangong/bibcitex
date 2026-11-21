//! One captured external application, and at most one live citation for it.
//! The JS-facing tickets keep old or repeated Enter events from injecting keys.

use bibcitex_service::SearchContext;
use nsh_core::prelude::*;
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
#[path = "../platform/paste_linux.rs"]
mod platform;

const STALE: &str = "Search results changed; search again before pasting";
const TICKET_LIFETIME: Duration = Duration::from_secs(300);

/// The application captured before showing the helper.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct CapturedTarget {
    /// Pass this opaque target to SearchSession.preparePaste.
    pub ticket: String,
    /// Display name when the desktop exposes the target identity.
    pub target_app_name: Option<String>,
}

/// A formatted citation ready to paste after hiding the helper.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct PastePlan {
    /// Single-use commit ticket bound to this selection and target.
    pub ticket: String,
    /// Citation text retained for manual copying.
    pub formatted_text: String,
}

/// Result of consuming a prepared paste ticket.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PropEnum)]
pub enum PasteStatus {
    /// The desktop accepted paste input for the captured target.
    Pasted,
    /// The citation can be copied manually with the same ticket.
    ManualCopy,
    /// The target or search results have changed.
    Stale,
}

/// Completion status for one paste attempt.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct PasteOutcome {
    /// Whether keys were sent or manual action is needed.
    pub status: PasteStatus,
    /// Localizable desktop error or stale-result reason.
    pub message: Option<String>,
}

fn new_ticket() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed).to_string()
}

#[derive(Clone)]
struct Selection {
    id: String,
    text: String,
    context: SearchContext,
    active: Arc<AtomicBool>,
    created: Instant,
}

impl Selection {
    fn live(&self) -> bool {
        self.active.load(Ordering::Acquire) && self.created.elapsed() < TICKET_LIFETIME
    }

    fn validate(&self) -> Result<(), String> {
        if self.live()
            && bibcitex_service::validate_search_context(self.context.clone())
                .map_err(|e| e.to_string())?
            && self.live()
        {
            Ok(())
        } else {
            Err(STALE.into())
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Preparing,
    Ready,
    Consumed,
    Fallback,
}

#[derive(Default)]
struct Tickets {
    target: Option<String>,
    capture_error: Option<String>,
    selection: Option<(Selection, Stage)>,
}

impl Tickets {
    fn clear(&mut self) {
        if let Some((selection, _)) = self.selection.take() {
            selection.active.store(false, Ordering::Release);
        }
    }

    fn select(
        &mut self,
        target: &str,
        text: String,
        context: SearchContext,
    ) -> Result<Selection, String> {
        if self.target.as_deref() != Some(target) || text.is_empty() {
            return Err(STALE.into());
        }
        self.clear();
        let selection = Selection {
            id: new_ticket(),
            text,
            context,
            active: Arc::new(AtomicBool::new(true)),
            created: Instant::now(),
        };
        self.selection = Some((selection.clone(), Stage::Preparing));
        Ok(selection)
    }

    fn transition(&mut self, id: &str, from: Stage, to: Stage) -> Result<Selection, String> {
        if let Some((selection, stage)) = self.selection.as_mut()
            && selection.id == id
            && selection.live()
            && *stage == from
        {
            *stage = to;
            if to == Stage::Consumed {
                // Reopening the helper must capture the current external app again.
                self.target = None;
            }
            return Ok(selection.clone());
        }
        Err(STALE.into())
    }
}

thread_local! {
    static TICKETS: RefCell<Tickets> = RefCell::default();
    static BUSY: Cell<bool> = const { Cell::new(false) };
}

// NSH tasks interleave on the engine thread. Keep target replacement and other
// writes out of an active OS transaction, without holding a lock across await.
struct Transaction;
impl Transaction {
    fn start() -> Result<Self, String> {
        if BUSY.replace(true) {
            Err("Another paste operation is in progress".into())
        } else {
            Ok(Self)
        }
    }
}
impl Drop for Transaction {
    fn drop(&mut self) {
        BUSY.set(false);
    }
}

pub(super) async fn capture_target() -> Result<CapturedTarget, String> {
    let _operation = Transaction::start()?;
    TICKETS.with_borrow_mut(|tickets| {
        tickets.clear();
        tickets.target = None;
    });
    #[cfg(target_os = "linux")]
    let result = platform::capture().await;
    #[cfg(not(target_os = "linux"))]
    let result = blocking(xpaste::capture_paste_target)
        .await
        .map_err(|e| e.to_string())?;
    // An unavailable target must still allow a search and a manual copy.
    let (name, error) = match result {
        Ok(name) => (name, None),
        Err(error) => (None, Some(error)),
    };
    let id = new_ticket();
    TICKETS.with_borrow_mut(|tickets| {
        tickets.target = Some(id.clone());
        tickets.capture_error = error;
    });
    Ok(CapturedTarget {
        ticket: id,
        target_app_name: name,
    })
}

pub(super) fn accessibility_status() -> bool {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "ApplicationServices", kind = "framework")]
        unsafe extern "C" {
            fn AXIsProcessTrusted() -> bool;
        }
        // SAFETY: this read-only system query takes no parameters or pointers.
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

async fn prepare_platform() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        super::desktop::linux_desktop()?.prepare().await
    }
    #[cfg(not(target_os = "linux"))]
    {
        accessibility_status()
            .then_some(())
            .ok_or_else(|| "缺少辅助功能权限，无法控制其他应用".to_owned())
    }
}

pub(super) async fn prepare(
    target: String,
    text: String,
    context: SearchContext,
) -> Result<PastePlan, String> {
    let _operation = Transaction::start()?;
    let selection = TICKETS.with_borrow_mut(|tickets| tickets.select(&target, text, context))?;
    let permission = match TICKETS.with_borrow(|tickets| tickets.capture_error.clone()) {
        Some(error) => Err(error),
        None => prepare_platform().await,
    };
    let check = selection.clone();
    blocking(move || check.validate())
        .await
        .map_err(|e| e.to_string())??;
    if let Err(message) = permission {
        TICKETS.with_borrow_mut(|tickets| {
            tickets.transition(&selection.id, Stage::Preparing, Stage::Fallback)
        })?;
        super::events::paste_failed(selection.id, message.clone());
        return Err(message);
    }
    TICKETS.with_borrow_mut(|tickets| {
        tickets.transition(&selection.id, Stage::Preparing, Stage::Ready)
    })?;
    Ok(PastePlan {
        ticket: selection.id,
        formatted_text: selection.text,
    })
}

/// A helper query edit invalidates a pending action on its former snapshot.
pub(super) fn invalidate(context: &SearchContext) {
    TICKETS.with_borrow_mut(|tickets| {
        if tickets.selection.as_ref().is_some_and(|(entry, _)| {
            entry.context.query_version == context.query_version
                && entry.context.path == context.path
                && entry.context.file_version == context.file_version
                && entry.context.tag_revision == context.tag_revision
        }) {
            tickets.clear();
        }
    });
}

pub(super) fn cancel_paste(id: String) {
    TICKETS.with_borrow_mut(|tickets| {
        if tickets.target.as_ref() == Some(&id) {
            tickets.clear();
            tickets.target = None;
        } else if tickets.selection.as_ref().is_some_and(|(s, _)| s.id == id) {
            tickets.clear();
        }
    });
}

pub(super) async fn copy_text(text: String) -> Result<(), String> {
    let _operation = Transaction::start()?;
    write_clipboard(text).await
}

async fn write_clipboard(text: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    return platform::copy(text).await;
    #[cfg(not(target_os = "linux"))]
    blocking(move || xpaste::copy(&text))
        .await
        .map_err(|e| e.to_string())?
}

pub(super) async fn copy_fallback(ticket: String) -> Result<(), String> {
    let _operation = Transaction::start()?;
    let selection = TICKETS.with_borrow(|tickets| {
        tickets
            .selection
            .as_ref()
            .filter(|(s, stage)| s.id == ticket && s.live() && *stage == Stage::Fallback)
            .map(|(s, _)| s.clone())
            .ok_or_else(|| STALE.to_owned())
    })?;
    let check = selection.clone();
    blocking(move || check.validate())
        .await
        .map_err(|e| e.to_string())??;
    if !selection.live() {
        return Err(STALE.into());
    }
    write_clipboard(selection.text).await
}

pub(super) async fn commit_paste(ticket: String) -> Result<PasteOutcome, String> {
    let _operation = Transaction::start()?;
    let Ok(selection) = TICKETS
        .with_borrow_mut(|tickets| tickets.transition(&ticket, Stage::Ready, Stage::Consumed))
    else {
        return Ok(PasteOutcome {
            status: PasteStatus::Stale,
            message: Some(STALE.to_owned()),
        });
    };
    // The caller just hid the helper. Give the compositor time to return focus
    // before verifying the source and injecting into the captured application.
    timer(Duration::from_millis(180)).await;
    #[cfg(target_os = "linux")]
    let result = platform::paste(selection).await;
    #[cfg(not(target_os = "linux"))]
    let result = blocking(move || {
        selection.validate()?;
        xpaste::paste(&selection.text)
    })
    .await
    .map_err(|e| e.to_string())?;
    match result {
        Ok(()) => {
            cancel_paste(ticket);
            Ok(PasteOutcome {
                status: PasteStatus::Pasted,
                message: None,
            })
        }
        Err(message) if message == STALE => {
            cancel_paste(ticket);
            Ok(PasteOutcome {
                status: PasteStatus::Stale,
                message: Some(message),
            })
        }
        Err(message) => {
            TICKETS.with_borrow_mut(|tickets| {
                tickets.transition(&ticket, Stage::Consumed, Stage::Fallback)
            })?;
            super::events::paste_failed(ticket, message.clone());
            Ok(PasteOutcome {
                status: PasteStatus::ManualCopy,
                message: Some(message),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> SearchContext {
        SearchContext {
            path: "library.bib".into(),
            file_version: "1".into(),
            tag_revision: None,
            query_version: 42,
        }
    }
    #[test]
    fn replacing_selection_invalidates_old_ticket_even_with_same_text() {
        let mut tickets = Tickets {
            target: Some("target".into()),
            ..Default::default()
        };
        let old = tickets.select("target", "same".into(), context()).unwrap();
        let new = tickets.select("target", "same".into(), context()).unwrap();
        assert!(!old.live());
        assert_ne!(old.id, new.id);
        assert!(
            tickets
                .transition(&old.id, Stage::Preparing, Stage::Ready)
                .is_err()
        );
    }
    #[test]
    fn a_prepared_ticket_can_commit_only_once() {
        let mut tickets = Tickets {
            target: Some("target".into()),
            ..Default::default()
        };
        let selection = tickets.select("target", "text".into(), context()).unwrap();
        tickets
            .transition(&selection.id, Stage::Preparing, Stage::Ready)
            .unwrap();
        tickets
            .transition(&selection.id, Stage::Ready, Stage::Consumed)
            .unwrap();
        assert!(
            tickets
                .transition(&selection.id, Stage::Ready, Stage::Consumed)
                .is_err()
        );
        assert!(tickets.select("target", "again".into(), context()).is_err());
    }
}
