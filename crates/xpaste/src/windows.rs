// Copyright (c) EcoPasteHub; modifications (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: Apache-2.0
// Derived from https://github.com/EcoPasteHub/EcoPaste
use ::windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindow, SW_RESTORE,
        SetForegroundWindow, ShowWindow,
    },
};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{sync::Mutex, thread, time::Duration};

// Keep the owning process as well as HWND to reject recycled handles.
static TARGET: Mutex<Option<(isize, u32)>> = Mutex::new(None);
static RECENT: Mutex<Option<(isize, u32)>> = Mutex::new(None);
fn process(hwnd: HWND) -> u32 {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    pid
}
pub fn observe_foreground() {
    let hwnd = unsafe { GetForegroundWindow() };
    let pid = process(hwnd);
    if !hwnd.0.is_null()
        && pid != 0
        && pid != std::process::id()
        && let Ok(mut recent) = RECENT.lock()
    {
        *recent = Some((hwnd.0 as isize, pid));
    }
}
pub fn capture_paste_target() -> Result<(), String> {
    observe_foreground();
    let recent = *RECENT.lock().map_err(|_| "Paste target lock poisoned")?;
    if recent.is_some() {
        *TARGET.lock().map_err(|_| "Paste target lock poisoned")? = recent;
    }
    Ok(())
}

pub fn focus_previous_window() -> Result<(), String> {
    let (raw, pid) = TARGET
        .lock()
        .map_err(|_| "Paste target lock poisoned")?
        .ok_or("No previous window found")?;
    let hwnd = HWND(raw as *mut _);
    unsafe {
        if !IsWindow(Some(hwnd)).as_bool() {
            return Err("Previous window is no longer valid".into());
        }
        crate::validate_target_process(pid, process(hwnd))?;
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        if !SetForegroundWindow(hwnd).as_bool() {
            return Err("Target window refused activation".into());
        }
        for _ in 0..25 {
            if GetForegroundWindow() == hwnd {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        if GetForegroundWindow() != hwnd || process(hwnd) != pid {
            return Err("Target window did not receive focus".into());
        }
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| e.to_string())?;
        let result = enigo.key(Key::Unicode('v'), Direction::Click);
        let release = enigo.key(Key::Control, Direction::Release);
        result.and(release).map_err(|e| e.to_string())
    }
}
