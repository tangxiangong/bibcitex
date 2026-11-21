//! GTK owns the clipboard provider for the lifetime of the display. XTest and
//! the RemoteDesktop portal are handled by the existing Linux desktop service.
use super::{STALE, Selection};
use crate::bridge::desktop::linux_desktop;
use gtk4::{Window, gdk, glib, prelude::*};
use nsh_core::element::{blocking, run_on_main_thread, timer};
use std::{cell::Cell, rc::Rc, time::Duration};

fn clipboard() -> Result<gdk::Clipboard, String> {
    gdk::Display::default()
        .map(|display| display.clipboard())
        .ok_or_else(|| "Clipboard is unavailable".to_owned())
}

fn owned_window_focused() -> bool {
    Window::list_toplevels().iter().any(|widget| {
        widget
            .downcast_ref::<Window>()
            .is_some_and(|window| window.is_active())
    })
}

pub(super) async fn capture() -> Result<Option<String>, String> {
    let desktop = linux_desktop()?;
    let external = run_on_main_thread(|| !owned_window_focused())
        .await
        .map_err(|e| e.to_string())?;
    blocking(move || {
        desktop.capture(external)?;
        Ok(desktop.captured_application_name())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub(super) async fn copy(text: String) -> Result<(), String> {
    run_on_main_thread(move || {
        let provider = gdk::ContentProvider::for_value(&text.to_value());
        clipboard()?
            .set_content(Some(&provider))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

enum Snapshot {
    Text(String),
    Empty,
    Unavailable,
}

async fn snapshot(board: &gdk::Clipboard) -> Snapshot {
    let formats = board.formats();
    if formats.mime_types().is_empty() && formats.types().is_empty() {
        return Snapshot::Empty;
    }
    // A URI-list, HTML, image or arbitrary binary payload cannot be restored
    // by writing its text conversion. Preserve only an entirely plain-text offer.
    if formats.mime_types().iter().any(|mime| {
        let base = mime.split(';').next().unwrap_or_default();
        !matches!(base, "text/plain" | "UTF8_STRING" | "STRING" | "TEXT")
    }) || formats
        .types()
        .iter()
        .any(|kind| *kind != String::static_type())
    {
        return Snapshot::Unavailable;
    }
    match glib::future_with_timeout(Duration::from_secs(1), board.read_text_future()).await {
        Ok(Ok(Some(text))) => Snapshot::Text(text.into()),
        _ => Snapshot::Unavailable,
    }
}

fn marker(id: &str) -> String {
    format!("application/x-bibcitex-paste-{}-{id}", std::process::id())
}

async fn install(selection: &Selection, marker: String) -> Result<Snapshot, String> {
    let (reply, receipt) = async_channel::bounded(1);
    let selection = selection.clone();
    run_on_main_thread(move || {
        glib::MainContext::ref_thread_default().spawn_local(async move {
            let result = async {
                let board = clipboard()?;
                let changed = Rc::new(Cell::new(false));
                let flag = changed.clone();
                let watch = board.connect_changed(move |_| flag.set(true));
                let saved = snapshot(&board).await;
                board.disconnect(watch);
                if !selection.live() {
                    return Err(STALE.to_owned());
                }
                if owned_window_focused() {
                    return Err("No previous application is available for paste".to_owned());
                }
                let provider = gdk::ContentProvider::new_union(&[
                    gdk::ContentProvider::for_value(&selection.text.to_value()),
                    gdk::ContentProvider::for_bytes(
                        &marker,
                        &glib::Bytes::from_static(b"BibCiTeX"),
                    ),
                ]);
                board
                    .set_content(Some(&provider))
                    .map_err(|e| e.to_string())?;
                Ok(if changed.get() {
                    Snapshot::Unavailable
                } else {
                    saved
                })
            }
            .await;
            let _ = reply.try_send(result);
        });
    })
    .await
    .map_err(|e| e.to_string())?;
    receipt.recv().await.map_err(|e| e.to_string())?
}

async fn restore(marker: String, saved: Snapshot) {
    let _ = run_on_main_thread(move || {
        if let Ok(board) = clipboard()
            && board.is_local()
            && board.formats().contain_mime_type(&marker)
        {
            match saved {
                Snapshot::Text(text) => board.set_text(&text),
                Snapshot::Empty => {
                    let _ = board.set_content(None::<&gdk::ContentProvider>);
                }
                Snapshot::Unavailable => {}
            }
        }
    })
    .await;
}

pub(super) async fn paste(selection: Selection) -> Result<(), String> {
    let desktop = linux_desktop()?;
    let check = selection.clone();
    blocking(move || check.validate())
        .await
        .map_err(|e| e.to_string())??;
    let marker = marker(&selection.id);
    let saved = install(&selection, marker.clone()).await?;
    let check = selection.clone();
    let verified = blocking(move || check.validate())
        .await
        .map_err(|e| e.to_string())?;
    let verify_marker = marker.clone();
    let owned = run_on_main_thread(move || {
        clipboard().is_ok_and(|board| {
            board.is_local() && board.formats().contain_mime_type(&verify_marker)
        }) && !owned_window_focused()
    })
    .await
    .map_err(|e| e.to_string())?;
    if verified.is_err() || !owned || !selection.live() {
        restore(marker, saved).await;
        return Err(STALE.into());
    }
    // Failed injection deliberately leaves the citation available for manual paste.
    desktop.paste().await?;
    timer(Duration::from_millis(200)).await;
    restore(marker, saved).await;
    Ok(())
}
