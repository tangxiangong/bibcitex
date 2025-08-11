// Copyright (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// This file contains code derived from EcoPaste
// Original source: https://github.com/EcoPasteHub/EcoPaste
// Copyright (c) EcoPasteHub
// Licensed under Apache-2.0

use crate::platforms::MAIN_WINDOW_TITLE;
use enigo::{Direction, Enigo, Key, Keyboard};
use std::{sync::Mutex, thread, time::Duration};
use x11::xlib::{
    self, Atom, Display, XCloseDisplay, XDefaultRootWindow, XFree, XGetInputFocus,
    XGetWindowProperty, XInternAtom, XNextEvent, XOpenDisplay, XRaiseWindow, XSelectInput,
    XSetInputFocus,
};

static PREVIOUS_WINDOW: Mutex<Option<u64>> = Mutex::new(None);

// 获取窗口标题
fn get_net_wm_name(display: *mut Display, window: u64) -> std::result::Result<String, String> {
    let mut actual_type: Atom = 0;
    let mut actual_format: i32 = 0;
    let mut nitems: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut prop: *mut u8 = std::ptr::null_mut();
    let net_wm_name_atom =
        unsafe { XInternAtom(display, c"_NET_WM_NAME".as_ptr() as _, xlib::False) };
    let result = unsafe {
        XGetWindowProperty(
            display,
            window,
            net_wm_name_atom,
            0,
            1024,
            xlib::False,
            xlib::AnyPropertyType as _,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop,
        )
    };
    if result == xlib::Success as i32 && !prop.is_null() {
        let name = unsafe {
            std::ffi::CStr::from_ptr(prop as *const std::ffi::c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { XFree(prop as *mut _) };
        Ok(name)
    } else {
        Err(format!("{}", window))
    }
}

// 监听窗口切换
pub fn observe_app() {
    std::thread::spawn(|| unsafe {
        let display = XOpenDisplay(std::ptr::null_mut());
        if display.is_null() {
            return;
        }

        let root_window = XDefaultRootWindow(display);
        XSelectInput(
            display,
            root_window,
            xlib::FocusChangeMask | xlib::PropertyChangeMask,
        );

        loop {
            let mut event = std::mem::zeroed();
            XNextEvent(display, &mut event);

            let mut window: u64 = 0;
            let mut revert_to_return: i32 = 0;
            XGetInputFocus(display, &mut window, &mut revert_to_return);

            if window == 1 {
                continue;
            }

            let wm_name = get_net_wm_name(display, window).unwrap_or_default();

            if wm_name.is_empty() || wm_name.eq(MAIN_WINDOW_TITLE) {
                continue;
            }

            let mut previous_window = PREVIOUS_WINDOW.lock().unwrap();
            let _ = previous_window.insert(window);
        }
    });
}

// 聚焦上一个窗口
pub fn focus_previous_window() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let display = XOpenDisplay(std::ptr::null_mut());
        if display.is_null() {
            return Err("Could not open display".into());
        }
        let window = match *PREVIOUS_WINDOW.lock().unwrap() {
            Some(window) => window,
            None => {
                return Err("Could not get active window".into());
            }
        };

        XRaiseWindow(display, window);
        XSetInputFocus(display, window, xlib::RevertToNone, xlib::CurrentTime);
        XCloseDisplay(display);
    }
    thread::sleep(Duration::from_millis(100));
    let mut enigo = Enigo::new(&enigo::Settings::default())?;
    enigo.key(Key::LShift, Direction::Press)?;
    enigo.key(Key::Unicode('v'), Direction::Click)?;
    enigo.key(Key::LShift, Direction::Release)?;
    Ok(())
}
