use crate::{
    STATE,
    components::{SearchRef, SelectBib},
};
use arboard::Clipboard;
use bibcitex_core::bib::Reference;
use dioxus::{
    desktop::{
        Config, DesktopService, WindowBuilder,
        tao::dpi::LogicalSize,
        tao::event::{Event, WindowEvent},
        use_window, use_wry_event_handler,
    },
    events::Key,
    prelude::*,
};
use enigo::{Direction, Enigo, Key as EnigoKey, Keyboard};
use itertools::Itertools;
use std::{
    rc::Weak,
    sync::{Arc, Mutex},
};

static CSS: Asset = asset!("/assets/styling/helper.css");

// 全局状态跟踪Helper窗口是否打开
pub static HELPER_WINDOW_OPEN: GlobalSignal<Option<Weak<DesktopService>>> = Signal::global(|| None);

// 使用 Arc<Mutex<>> 来确保状态在不同 VirtualDom 实例间共享
static HELPER_BIB_STATE: std::sync::LazyLock<Arc<Mutex<Option<Vec<Reference>>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(None)));

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

pub fn open_spotlight_window() {
    // 检查是否已经有Helper窗口打开
    let should_close = {
        let window_signal = HELPER_WINDOW_OPEN();
        if let Some(helper_window_weak) = window_signal.as_ref() {
            if let Some(helper_window) = helper_window_weak.upgrade() {
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
        HELPER_WINDOW_OPEN.write().take();
        return;
    }

    let window = use_window();

    // Spotlight风格的窗口尺寸
    let window_width = 600.0;
    let min_window_height = 56.0; // 最小高度：输入框高度
    let max_window_height = 500.0; // 最大高度

    // 创建Spotlight风格的窗口配置（不指定位置，让系统居中）
    let window_builder = WindowBuilder::new()
        .with_title("BibCiteX Spotlight")
        .with_inner_size(LogicalSize::new(window_width, min_window_height))
        .with_min_inner_size(LogicalSize::new(window_width, min_window_height))
        .with_max_inner_size(LogicalSize::new(window_width, max_window_height))
        .with_focused(false)
        .with_decorations(false) // 移除窗口装饰
        .with_transparent(true) // 支持透明背景
        .with_always_on_top(true) // 保持在最上层
        .with_resizable(true); // 允许调整大小以显示搜索结果

    let config = Config::new().with_window(window_builder);

    // 创建新窗口并保存窗口句柄
    // 使用 new_window_with_vdom 而不是 new_window 来确保全局状态共享
    let helper_window = window.new_window(VirtualDom::new(Helper), config);
    *HELPER_WINDOW_OPEN.write() = Some(helper_window);
}

/// The actual Helper window content
#[component]
pub fn Helper() -> Element {
    let query = use_context_provider(|| Signal::new(String::new()));

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
        let has_results = !query().is_empty();
        if has_results || !has_bib() {
            // 有搜索结果时扩展窗口高度
            window.set_inner_size(LogicalSize::new(600.0, 300.0));
        } else {
            // 无搜索结果时缩小到最小高度
            window.set_inner_size(LogicalSize::new(600.0, 56.0));
        }
    });

    // 使用 use_wry_event_handler 直接监听 tao 窗口事件
    use_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::Focused(focused),
            ..
        } = event
        {
            if !focused {
                // 窗口失去焦点时自动关闭
                let window = use_window();
                window.close();
                // 清除窗口状态
                HELPER_WINDOW_OPEN.write().take();
            }
        }
    });

    // 窗口初始化时的清理逼辑
    use_effect(move || {
        // 窗口关闭时的清理逻辑将在上面的事件处理中执行
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
        document::Link { rel: "stylesheet", href: CSS }

        div {
            class: "helper-container",
            onkeydown: move |evt| {
                if evt.key() == Key::Escape {
                    let window = use_window();
                    window.close();
                    HELPER_WINDOW_OPEN.write().take();
                }
            },
            if !has_bib() {
                SelectBib { bibs }
            } else {
                SearchRef {}
            }
        }
    }
}
