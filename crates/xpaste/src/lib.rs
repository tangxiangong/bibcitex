//! Copyright (c) EcoPasteHub
//!
//! License: Apache-2.0
//!
//! Modified by tangxiangong (2025) for [bibcitex](https://github.com/tangxiangong/bibcitex).
//!
//! # Note
//!
//! This crate is forked from the [EcoPasteHub/EcoPaste](https://github.com/EcoPasteHub/EcoPaste), which is licensed under [Apache 2.0](https://github.com/EcoPasteHub/EcoPaste/blob/master/LICENSE).

use std::sync::Mutex;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

/// Shared validation used immediately before focus/key injection.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
fn validate_target_process(expected: u32, actual: u32) -> Result<(), String> {
    if expected == 0 || expected == std::process::id() || actual != expected {
        Err("Previous application is no longer a valid paste target".into())
    } else {
        Ok(())
    }
}

// Serialize clipboard and key injection as one transaction, without locking searches.
static PASTE: Mutex<()> = Mutex::new(());

pub fn capture_paste_target() -> Result<(), String> {
    transaction(platform::capture_paste_target)
}
/// Start once at app launch. The recent external target and active paste snapshot are separate.
pub fn observe_app() {
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| {
        platform::observe_foreground();
        std::thread::spawn(|| {
            loop {
                platform::observe_foreground();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    });
}
pub fn focus_previous_window() -> Result<(), String> {
    platform::focus_previous_window()
}
fn transaction<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    let _guard = PASTE.lock().map_err(|_| "Clipboard lock poisoned")?;
    operation()
}
pub fn copy(text: &str) -> Result<(), String> {
    transaction(|| copy_inner(text))
}
fn copy_inner(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .map_err(|e| e.to_string())?
        .set_text(text)
        .map_err(|e| e.to_string())
}
pub fn paste(text: &str) -> Result<(), String> {
    transaction(|| {
        copy_inner(text)?;
        focus_previous_window()
    })
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    pub fn observe_foreground() {}
    pub fn capture_paste_target() -> Result<(), String> {
        Err("Unsupported platform".into())
    }
    pub fn focus_previous_window() -> Result<(), String> {
        Err("Unsupported platform".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paste_target_rejects_self_missing_and_recycled_processes() {
        let own = std::process::id();
        let external = own + 1;
        assert!(validate_target_process(0, 0).is_err());
        assert!(validate_target_process(own, own).is_err());
        assert!(validate_target_process(external, own + 2).is_err());
        assert!(validate_target_process(external, external).is_ok());
    }
    #[test]
    fn target_capture_transaction_waits_for_in_flight_paste() {
        use std::sync::mpsc;
        use std::time::Duration;
        let (paste_started, started) = mpsc::channel();
        let (finish_paste, finish) = mpsc::channel();
        let (capture_waiting, waiting) = mpsc::channel();
        let (capture_completed, completed) = mpsc::channel();
        std::thread::scope(|scope| {
            let paste = scope.spawn(move || {
                transaction(|| {
                    paste_started.send(()).unwrap();
                    finish.recv().unwrap();
                    Ok(())
                })
            });
            started.recv().unwrap();
            let capture = scope.spawn(move || {
                capture_waiting.send(()).unwrap();
                transaction(|| {
                    capture_completed.send(()).unwrap();
                    Ok(())
                })
            });
            waiting.recv().unwrap();
            let capture_was_blocked = matches!(
                completed.recv_timeout(Duration::from_millis(50)),
                Err(mpsc::RecvTimeoutError::Timeout)
            );
            // Release the worker before asserting, so a regression cannot deadlock this test.
            finish_paste.send(()).unwrap();
            paste.join().unwrap().unwrap();
            capture.join().unwrap().unwrap();
            assert!(capture_was_blocked);
            completed.recv().unwrap();
        });
    }
}
