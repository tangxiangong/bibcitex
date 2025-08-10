use crate::platforms::MAIN_WINDOW_TITLE;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::{ffi::OsString, os::windows::ffi::OsStringExt, ptr, sync::Mutex, thread, time::Duration};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        EVENT_SYSTEM_FOREGROUND, GetWindowTextLengthW, GetWindowTextW, HWINEVENTHOOK,
        SetForegroundWindow, SetWinEventHook, WINEVENT_OUTOFCONTEXT,
    },
};

static PREVIOUS_WINDOW: Mutex<Option<isize>> = Mutex::new(None);

// 获取窗口标题
unsafe fn get_window_title(hwnd: HWND) -> String {
    let length = GetWindowTextLengthW(hwnd);

    if length == 0 {
        return String::new();
    }

    let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];

    GetWindowTextW(hwnd, buffer.as_mut_ptr(), length + 1);

    OsString::from_wide(&buffer[..length as usize])
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
        let window_title = get_window_title(hwnd);

        if window_title == MAIN_WINDOW_TITLE {
            return;
        }

        let mut previous_window = PREVIOUS_WINDOW.lock().unwrap();
        let _ = previous_window.insert(hwnd as isize);
    }
}

// 监听窗口切换
pub fn observe_app() {
    thread::spawn(|| {
        unsafe {
            // 设置事件钩子
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                0, // hmodWinEventProc
                Some(event_hook_callback),
                0, // idProcess
                0, // idThread
                WINEVENT_OUTOFCONTEXT,
            );

            if hook == 0 {
                log::error!("设置事件钩子失败");
                return;
            }

            // 保持线程运行以监听事件
            loop {
                thread::sleep(Duration::from_millis(100));
            }
        }
    });
}

fn get_previous_window() -> Option<isize> {
    *PREVIOUS_WINDOW.lock().unwrap()
}

// 聚焦上一个窗口
pub fn focus_previous_window() -> Result<(), Box<dyn std::error::Error>> {
    let hwnd_option = get_previous_window();

    if let Some(hwnd_value) = hwnd_option {
        unsafe {
            let hwnd = hwnd_value as HWND;

            if hwnd == 0 {
                return Err("Invalid window handle".into());
            }

            let success = SetForegroundWindow(hwnd);
            if success == 0 {
                return Err("Failed to set foreground window".into());
            }

            // 等待窗口激活
            thread::sleep(Duration::from_millis(100));

            // 执行粘贴操作
            let mut enigo = Enigo::new(&Settings::default())?;
            enigo.key(Key::Control, Press)?;
            enigo.key(Key::Unicode('v'), Click)?;
            enigo.key(Key::Control, Release)?;

            Ok(())
        }
    } else {
        Err("No previous window found".into())
    }
}
