// Copyright (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// This file contains code derived from EcoPaste
// Original source: https://github.com/EcoPasteHub/EcoPaste
// Copyright (c) EcoPasteHub
// Licensed under Apache-2.0

use crate::MAIN_WINDOW_TITLE;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{ffi::OsString, os::windows::ffi::OsStringExt, sync::Mutex, thread, time::Duration};
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook},
        WindowsAndMessaging::{
            EVENT_SYSTEM_FOREGROUND, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
            IsWindow, SetForegroundWindow, WINEVENT_OUTOFCONTEXT,
        },
    },
};

static PREVIOUS_WINDOW: Mutex<Option<isize>> = Mutex::new(None);
static CURRENT_WINDOW: Mutex<Option<isize>> = Mutex::new(None);

// 获取窗口标题
unsafe fn get_window_title(hwnd: HWND) -> String {
    let length = unsafe { GetWindowTextLengthW(hwnd) };

    if length == 0 {
        return String::new();
    }

    let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];

    let copied = unsafe { GetWindowTextW(hwnd, &mut buffer) };

    OsString::from_wide(&buffer[..copied as usize])
        .to_string_lossy()
        .into_owned()
}

// 定义事件钩子回调函数
unsafe extern "system" fn event_hook_callback(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    if event == EVENT_SYSTEM_FOREGROUND {
        let window_title = unsafe { get_window_title(hwnd) };

        // 获取当前窗口
        let mut current_window = CURRENT_WINDOW.lock().unwrap();
        let previous_hwnd = *current_window;

        // 如果新窗口是我们的主窗口，不更新previous_window
        if window_title == MAIN_WINDOW_TITLE {
            *current_window = Some(hwnd.0 as isize);
            return;
        }

        // 如果之前的窗口不是我们的主窗口，则更新previous_window
        if let Some(prev_hwnd) = previous_hwnd {
            let prev_title = unsafe { get_window_title(HWND(prev_hwnd as *mut _)) };
            if prev_title != MAIN_WINDOW_TITLE {
                let mut previous_window = PREVIOUS_WINDOW.lock().unwrap();
                *previous_window = Some(prev_hwnd);
            }
        }

        *current_window = Some(hwnd.0 as isize);
    }
}

// 初始化当前窗口信息
fn init_current_window() {
    unsafe {
        let current_hwnd = GetForegroundWindow();
        if !current_hwnd.0.is_null() {
            let mut current_window = CURRENT_WINDOW.lock().unwrap();
            *current_window = Some(current_hwnd.0 as isize);
        }
    }
}

// 监听窗口切换
pub fn observe_app() {
    // 初始化当前窗口
    init_current_window();

    unsafe {
        // 设置事件钩子
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(event_hook_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        if hook.0.is_null() {
            eprintln!("设置事件钩子失败");
        }
    }
}

// 获取最近的非BibCiTeX窗口
fn get_most_recent_window() -> Option<isize> {
    let previous = PREVIOUS_WINDOW.lock().unwrap();
    *previous
}

// 检查窗口是否仍然有效
unsafe fn is_window_valid(hwnd: HWND) -> bool {
    unsafe { IsWindow(Some(hwnd)).as_bool() }
}

// 聚焦上一个窗口
pub fn focus_previous_window() -> Result<(), Box<dyn std::error::Error>> {
    let hwnd_option = get_most_recent_window();

    if let Some(hwnd) = hwnd_option {
        unsafe {
            let hwnd = HWND(hwnd as *mut _);

            // 检查窗口是否仍然有效
            if !is_window_valid(hwnd) {
                return Err("Previous window is no longer valid".into());
            }

            // 获取窗口标题以进行调试
            let window_title = get_window_title(hwnd);

            // 尝试激活窗口
            let result = SetForegroundWindow(hwnd);
            if !result.as_bool() {
                return Err(format!("Failed to set foreground window: {window_title}").into());
            }

            // 等待窗口激活
            thread::sleep(Duration::from_millis(200));

            // 执行粘贴操作 (Ctrl+V)
            let mut enigo = Enigo::new(&Settings::default())?;
            enigo.key(Key::Control, Direction::Press)?;
            enigo.key(Key::Unicode('v'), Direction::Click)?;
            enigo.key(Key::Control, Direction::Release)?;

            Ok(())
        }
    } else {
        Err("No previous window found".into())
    }
}
