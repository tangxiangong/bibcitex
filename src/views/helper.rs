use dioxus::{
    desktop::{
        Config, WindowBuilder,
        tao::dpi::LogicalSize,
        tao::event::{Event, WindowEvent},
        use_window, use_wry_event_handler,
    },
    events::Key,
    prelude::*,
};

static CSS: Asset = asset!("/assets/styling/helper.css");

/// Helper function to open the spotlight window
pub fn open_spotlight_window() {
    let window = use_window();

    // Spotlight风格的窗口尺寸
    let window_width = 600.0;
    let window_height = 56.0; // 输入框高度: 24px + 上下 padding 16px*2 = 56px

    // 创建Spotlight风格的窗口配置（不指定位置，让系统居中）
    let window_builder = WindowBuilder::new()
        .with_title("BibCiteX Spotlight")
        .with_inner_size(LogicalSize::new(window_width, window_height))
        .with_decorations(false) // 移除窗口装饰
        .with_transparent(true) // 支持透明背景
        .with_always_on_top(true) // 保持在最上层
        .with_resizable(false);

    let config = Config::new().with_window(window_builder);

    let _ = window.new_window(VirtualDom::new(Helper), config);
}

/// The actual Helper window content
#[component]
pub fn Helper() -> Element {
    let mut search_query = use_signal(String::new);

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
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }

        div {
            class: "helper-container",
            onkeydown: move |evt| {
                if evt.key() == Key::Escape {
                    let window = use_window();
                    window.close();
                }
            },
            // 搜索输入框
            input {
                class: "helper-input",
                r#type: "text",
                placeholder: "搜索文献、作者、标题...",
                value: "{search_query}",
                oninput: move |evt| search_query.set(evt.value()),
                autofocus: true,
            }

            // 搜索结果区域 - 只在有输入时显示
            if !search_query().is_empty() {
                div { class: "helper-results",
                    // TODO: 这里将显示实际的搜索结果
                    div { class: "helper-no-results",
                        p { "搜索: \"{search_query()}\"" }
                        p { "（搜索功能正在开发中）" }
                    }
                }
            }
        }
    }
}
