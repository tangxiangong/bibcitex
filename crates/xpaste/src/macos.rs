// Copyright (c) EcoPasteHub; modifications (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: Apache-2.0
// Derived from https://github.com/EcoPasteHub/EcoPaste
use enigo::{Direction, Enigo, Key, Keyboard};
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};
use std::{sync::Mutex, thread, time::Duration};

static TARGET: Mutex<Option<i32>> = Mutex::new(None);
static RECENT: Mutex<Option<i32>> = Mutex::new(None);
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn observe_foreground() {
    // The observer runs on a Rust worker, outside AppKit's main-loop pools.
    autoreleasepool(|_| observe_foreground_inner());
}
fn observe_foreground_inner() {
    if let Some(app) = NSWorkspace::sharedWorkspace().frontmostApplication() {
        let pid = app.processIdentifier();
        if pid != std::process::id() as i32
            && let Ok(mut recent) = RECENT.lock()
        {
            *recent = Some(pid);
        }
    }
}
pub fn capture_paste_target() -> Result<(), String> {
    observe_foreground();
    let recent = *RECENT.lock().map_err(|_| "Paste target lock poisoned")?;
    // The observer never changes this snapshot while a helper session is in progress.
    if recent.is_some() {
        *TARGET.lock().map_err(|_| "Paste target lock poisoned")? = recent;
    }
    Ok(())
}

pub fn focus_previous_window() -> Result<(), String> {
    autoreleasepool(|_| focus_previous_window_inner())
}
fn focus_previous_window_inner() -> Result<(), String> {
    if !unsafe { AXIsProcessTrusted() } {
        return Err("缺少辅助功能权限，无法控制其他应用".into());
    }
    let pid = TARGET
        .lock()
        .map_err(|_| "Paste target lock poisoned")?
        .ok_or("No previous window found")?;
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
        .ok_or("Previous application is no longer running")?;
    crate::validate_target_process(pid as u32, app.processIdentifier() as u32)?;
    if !app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows) {
        return Err("Target application refused activation".into());
    }
    let is_frontmost = || {
        NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .is_some_and(|front| front.processIdentifier() == pid)
    };
    for _ in 0..25 {
        if is_frontmost() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let mut enigo = Enigo::new(&enigo::Settings::default()).map_err(|e| e.to_string())?;
    if !is_frontmost() {
        return Err("Target application did not receive focus".into());
    }
    enigo
        .key(Key::Meta, Direction::Press)
        .map_err(|e| e.to_string())?;
    let result = enigo.key(Key::Unicode('v'), Direction::Click);
    let release = enigo.key(Key::Meta, Direction::Release);
    result.and(release).map_err(|e| e.to_string())
}
