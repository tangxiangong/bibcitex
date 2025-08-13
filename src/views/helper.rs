use crate::{
    STATE, TAILWINDCSS,
    components::{Search, Select},
};
use arboard::Clipboard;
use bibcitex_core::bib::Reference;
use dioxus::{
    desktop::{
        Config, DesktopService, LogicalSize, WindowBuilder, WindowEvent, tao::event::Event,
        use_window, use_wry_event_handler,
    },
    prelude::*,
};
use enigo::{Direction, Enigo, Key as EnigoKey, Keyboard};
use itertools::Itertools;
use std::{
    rc::{Rc, Weak},
    sync::{Arc, LazyLock, Mutex},
};

pub static WIDTH: usize = 700;
pub static MIN_HEIGHT: usize = 60;
pub static MAX_HEIGHT: usize = 600;

// 全局状态跟踪Helper窗口是否打开
pub static HELPER_WINDOW: GlobalSignal<Option<Weak<DesktopService>>> = Signal::global(|| None);

// 使用 Arc<Mutex<>> 来确保状态在不同 VirtualDom 实例间共享
static HELPER_BIB_STATE: LazyLock<Arc<Mutex<Option<Vec<Reference>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

pub static HELPER_BIB: GlobalSignal<Option<Vec<Reference>>> =
    Signal::global(|| HELPER_BIB_STATE.lock().unwrap().clone());

// 辅助函数来设置和获取 HELPER_BIB 状态
pub fn set_helper_bib(refs: Option<Vec<Reference>>) {
    *HELPER_BIB_STATE.lock().unwrap() = refs.clone();
    *HELPER_BIB.write() = refs;
}

pub fn get_helper_bib() -> Option<Vec<Reference>> {
    HELPER_BIB_STATE.lock().unwrap().clone()
}

#[allow(dead_code)]
pub(crate) fn paste_to_active_app(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut enigo = Enigo::new(&enigo::Settings::default())?;

    #[cfg(target_os = "macos")]
    {
        enigo.key(EnigoKey::Meta, Direction::Press)?;
        enigo.key(EnigoKey::Unicode('v'), Direction::Click)?;
        enigo.key(EnigoKey::Meta, Direction::Release)?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(EnigoKey::Control, Direction::Press)?;
        enigo.key(EnigoKey::Unicode('v'), Direction::Click)?;
        enigo.key(EnigoKey::Control, Direction::Release)?;
    }

    Ok(())
}

pub async fn open_spotlight_window() {
    // 检查是否已经有Helper窗口打开
    let should_close = {
        let window_signal = HELPER_WINDOW.read();
        if let Some(weak_window) = window_signal.as_ref() {
            if let Some(helper_window) = weak_window.upgrade() {
                helper_window.close();
                true
            } else {
                true
            }
        } else {
            false
        }
    };

    if should_close {
        HELPER_WINDOW.write().take();
        return;
    }

    let window = use_window();

    // 创建Spotlight风格的窗口配置（不指定位置，让系统居中）
    let window_builder = WindowBuilder::new()
        .with_title("BibCiteX Spotlight")
        .with_inner_size(LogicalSize::new(WIDTH as f64, MIN_HEIGHT as f64))
        .with_min_inner_size(LogicalSize::new(WIDTH as f64, MIN_HEIGHT as f64))
        .with_max_inner_size(LogicalSize::new(WIDTH as f64, MAX_HEIGHT as f64))
        .with_focused(true)
        .with_decorations(false) // 移除窗口装饰
        .with_transparent(true) // 支持透明背景
        .with_always_on_top(true) // 保持在最上层
        .with_resizable(true); // 允许调整大小以显示搜索结果

    let helper_html = r#"<!doctype html>
<html data-theme="nord" style="background: transparent;">
    <head>
        <title>BibCiteX Helper</title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />
    </head>
    <body style="background: transparent; margin: 0; padding: 0;">
        <div id="main" style="background: transparent;"></div>
    </body>
</html>"#;

    let config = Config::new()
        .with_window(window_builder)
        .with_custom_index(helper_html.to_string());

    // 创建新窗口并保存窗口句柄
    let helper_window = window.new_window(VirtualDom::new(Helper), config).await;
    *HELPER_WINDOW.write() = Some(Rc::downgrade(&helper_window));
}

/// The actual Helper window content
#[component]
pub fn Helper() -> Element {
    let content_height = use_context_provider(|| Signal::new(MIN_HEIGHT)); // 提供内容高度信号

    // 在组件初始化时从持久化状态恢复 HELPER_BIB
    use_effect(move || {
        let stored_bib = get_helper_bib();
        if stored_bib.is_some() && HELPER_BIB().is_none() {
            *HELPER_BIB.write() = stored_bib;
        }
    });

    let has_bib = use_memo(|| HELPER_BIB().is_some());

    // 动态调整窗口大小
    let window = use_window();
    use_effect(move || {
        let current_height = content_height();
        // 确保高度在合理范围内
        let adjusted_height = current_height.max(MIN_HEIGHT).min(MAX_HEIGHT) + 4;

        window.set_inner_size(LogicalSize::new(WIDTH as f64, adjusted_height as f64));
    });

    // 使用 use_wry_event_handler 直接监听 tao 窗口事件
    use_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::Focused(focused),
            ..
        } = event
            && !focused
        {
            // 窗口失去焦点时自动关闭
            let window = use_window();
            window.close();
            // 清除窗口状态
            HELPER_WINDOW.write().take();
        }
    });

    let bibs = use_memo(|| {
        let state = STATE.read();
        state
            .bibliographies
            .iter()
            .sorted_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at))
            .map(|(name, info)| {
                (
                    name.clone(),
                    info.path.as_os_str().to_str().unwrap().to_string(),
                    info.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                )
            })
            .collect::<Vec<_>>()
    });

    rsx! {
        document::Stylesheet { href: TAILWINDCSS }

        div {
            class: "w-full h-auto bg-transparent",
            onkeydown: move |evt| {
                if evt.key() == Key::Escape {
                    let window = use_window();
                    window.close();
                    HELPER_WINDOW.write().take();
                }
            },
            if !has_bib() {
                Select { bibs }
            } else {
                Search {}
            }
        }
    }
}
