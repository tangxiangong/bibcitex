// Copyright (c) EcoPasteHub; modifications (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: Apache-2.0
// Derived from https://github.com/EcoPasteHub/EcoPaste
use ::windows::Win32::{
    Foundation::{CloseHandle, HWND},
    Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
    System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    },
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindow, SW_RESTORE,
        SetForegroundWindow, ShowWindow,
    },
};
use ::windows::core::{PCWSTR, PWSTR};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{sync::Mutex, thread, time::Duration};

// Keep the owning process as well as HWND to reject recycled handles.
static TARGET: Mutex<Option<(isize, u32)>> = Mutex::new(None);
static RECENT: Mutex<Option<(isize, u32)>> = Mutex::new(None);

pub fn paste_preserving_clipboard(text: &str) -> Result<(), String> {
    let Ok(mut clipboard) = crate::windows_clipboard::Pasteboard::new() else {
        // Preservation infrastructure must not prevent ordinary paste.
        crate::copy_inner(text)?;
        return focus_previous_window();
    };
    crate::clipboard::paste(&mut clipboard, text, focus_previous_window, thread::sleep)
}
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
pub fn capture_paste_target() -> Result<Option<String>, String> {
    observe_foreground();
    let recent = *RECENT.lock().map_err(|_| "Paste target lock poisoned")?;
    if recent.is_some() {
        *TARGET.lock().map_err(|_| "Paste target lock poisoned")? = recent;
    }
    Ok(recent.and_then(|(_, pid)| application_name(pid)))
}

fn application_name(pid: u32) -> Option<String> {
    // Metadata is optional: failure to read it must not prevent pasting.
    let mut path = vec![0u16; 32768];
    let mut length = path.len() as u32;
    // SAFETY: the process handle is closed after the query and the buffer length is valid.
    let queried = unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(path.as_mut_ptr()),
            &mut length,
        );
        let _ = CloseHandle(handle);
        result
    };
    queried.ok()?;
    path.truncate(length as usize);
    let fallback = std::path::Path::new(&String::from_utf16_lossy(&path))
        .file_stem()
        .map(|name| name.to_string_lossy().into_owned());
    path.push(0);
    file_description(&path).or(fallback)
}

fn file_description(path: &[u16]) -> Option<String> {
    // SAFETY: path is NUL-terminated. Version queries borrow the live, aligned data buffer.
    unsafe {
        let size = GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), None);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u32; (size as usize).div_ceil(4)];
        GetFileVersionInfoW(PCWSTR(path.as_ptr()), None, size, data.as_mut_ptr().cast()).ok()?;
        let query = |key: &str| -> Option<(*const u16, usize)> {
            let key: Vec<u16> = key.encode_utf16().chain(Some(0)).collect();
            let mut value = std::ptr::null_mut();
            let mut length = 0;
            if !VerQueryValueW(
                data.as_ptr().cast(),
                PCWSTR(key.as_ptr()),
                &mut value,
                &mut length,
            )
            .as_bool()
                || value.is_null()
                || length == 0
            {
                return None;
            }
            Some((value.cast(), length as usize))
        };
        let (translations, bytes) = query(r"\VarFileInfo\Translation")?;
        let translations = std::slice::from_raw_parts(translations, bytes / 2);
        for pair in translations.as_chunks::<2>().0 {
            for field in ["FileDescription", "ProductName"] {
                let key = format!(r"\StringFileInfo\{:04x}{:04x}\{field}", pair[0], pair[1]);
                if let Some((value, length)) = query(&key) {
                    let text = String::from_utf16_lossy(std::slice::from_raw_parts(value, length));
                    let text = text.trim_end_matches('\0').trim();
                    if !text.is_empty() {
                        return Some(text.to_owned());
                    }
                }
            }
        }
        None
    }
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
