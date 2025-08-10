#![allow(dead_code)]
use enigo::{Direction, Enigo, Key as EnigoKey, Keyboard};
use objc2::{
    rc::Retained,
    runtime::{AnyObject, Sel},
};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_foundation::{NSDictionary, NSNotification, NSString};
use std::process::Command;
use std::{sync::Mutex, thread, time::Duration};

static PREVIOUS_WINDOW: Mutex<Option<i32>> = Mutex::new(None);
static MAIN_WINDOW_TITLE: &str = "BibCiTeX";

extern "C" fn application_did_activate(
    _self: &AnyObject,
    _cmd: Sel,
    notification: &NSNotification,
) {
    unsafe {
        let user_info: Option<Retained<NSDictionary>> = notification.userInfo();

        if user_info.is_none() {
            return;
        }

        let app_dict = user_info.as_ref().unwrap();
        let app_key = NSString::from_str("NSWorkspaceApplicationKey");
        let app: Option<Retained<NSRunningApplication>> = app_dict
            .objectForKey(&app_key)
            .and_then(|obj| obj.downcast().ok());

        if let Some(app) = app {
            let localized_name = app.localizedName();
            let name = localized_name
                .map(|n| n.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            if name == MAIN_WINDOW_TITLE {
                return;
            }

            let process_id = app.processIdentifier();

            let mut previous_window = PREVIOUS_WINDOW.lock().unwrap();
            let _ = previous_window.insert(process_id);
        }
    }
}

static LAST_ACTIVE_APP: Mutex<Option<i32>> = Mutex::new(None);
static CURRENT_APP_PID: Mutex<Option<i32>> = Mutex::new(None);
static CURRENT_APP_BUNDLE_ID: Mutex<Option<String>> = Mutex::new(None);

fn init_current_app_info() {
    let current_pid = std::process::id() as i32;
    unsafe {
        if let Some(current_app) =
            NSRunningApplication::runningApplicationWithProcessIdentifier(current_pid)
        {
            let bundle_id = current_app.bundleIdentifier().map(|id| id.to_string());

            *CURRENT_APP_PID.lock().unwrap() = Some(current_pid);
            *CURRENT_APP_BUNDLE_ID.lock().unwrap() = bundle_id.clone();
        }
    }
}

// 检查是否有辅助功能权限
fn check_accessibility_permissions() -> bool {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get name of first process")
        .output();

    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

// 检查给定的应用是否是当前 Dioxus 应用的窗口
fn is_current_app_window(app: &NSRunningApplication) -> bool {
    let current_pid = CURRENT_APP_PID.lock().unwrap();
    let current_bundle_id = CURRENT_APP_BUNDLE_ID.lock().unwrap();

    unsafe {
        let app_pid = app.processIdentifier();
        let app_bundle_id = app.bundleIdentifier().map(|id| id.to_string());

        if let Some(cur_pid) = *current_pid
            && app_pid == cur_pid
        {
            return true;
        }
        if let (Some(cur_bundle), Some(app_bundle)) = (current_bundle_id.as_ref(), &app_bundle_id)
            && cur_bundle == app_bundle
        {
            return true;
        }
        let app_name = app.localizedName().unwrap_or_default().to_string();
        if app_name == MAIN_WINDOW_TITLE {
            return true;
        }

        false
    }
}

pub fn observe_app() {
    // 初始化当前应用信息
    init_current_app_info();

    thread::spawn(|| {
        loop {
            unsafe {
                let workspace = NSWorkspace::sharedWorkspace();
                if let Some(active_app) = workspace.frontmostApplication() {
                    let pid = active_app.processIdentifier();
                    let _app_name = active_app.localizedName().unwrap_or_default().to_string();

                    let mut last_pid = LAST_ACTIVE_APP.lock().unwrap();

                    if *last_pid != Some(pid) {
                        // 如果当前应用不是当前 Dioxus 应用，更新 PREVIOUS_WINDOW
                        if !is_current_app_window(&active_app) {
                            let mut previous = PREVIOUS_WINDOW.lock().unwrap();
                            *previous = Some(pid);
                        }

                        *last_pid = Some(pid);
                    }
                }
            }
            thread::sleep(Duration::from_millis(200)); // 减少到200ms提高响应性
        }
    });
}

// 获取最近的非BibCiTeX窗口
fn get_most_recent_window() -> Option<i32> {
    let previous = PREVIOUS_WINDOW.lock().unwrap();
    *previous
}

pub fn focus_previous_window() -> Result<(), Box<dyn std::error::Error>> {
    // 检查辅助功能权限
    if !check_accessibility_permissions() {
        return Err("缺少辅助功能权限，无法控制其他应用".into());
    }

    let pid_option = get_most_recent_window();

    if let Some(pid) = pid_option {
        unsafe {
            if let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) {
                let app_name = app
                    .localizedName()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                // 尝试多种激活方式
                let success1 = app.activateWithOptions(
                    objc2_app_kit::NSApplicationActivationOptions::ActivateAllWindows,
                );

                let success2 = true;

                if success1 || success2 {
                    // 增加等待时间确保应用完全激活
                    thread::sleep(Duration::from_millis(100));

                    // 模拟 Cmd+V 粘贴操作
                    let mut enigo = Enigo::new(&enigo::Settings::default())?;
                    enigo.key(EnigoKey::Meta, Direction::Press)?;
                    enigo.key(EnigoKey::Unicode('v'), Direction::Click)?;
                    enigo.key(EnigoKey::Meta, Direction::Release)?;

                    Ok(())
                } else {
                    let error_msg = format!("所有激活方式都失败了: {}", app_name);
                    Err(error_msg.into())
                }
            } else {
                let error_msg = format!("Application not found for PID: {}", pid);
                Err(error_msg.into())
            }
        }
    } else {
        let error_msg = "No previous window found";
        Err(error_msg.into())
    }
}
