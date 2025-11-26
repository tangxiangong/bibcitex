use crate::{
    LOGO,
    components::{BibliographySelector, HelperComponent},
    views::{HELPER_BIB, HELPER_WINDOW, MAX_HEIGHT, MIN_HEIGHT, set_helper_bib},
};
use arboard::Clipboard;
use bibcitex_core::{
    bib::{Reference, parse},
    search_references,
    utils::read_bibliography,
};
use biblatex::EntryType;
use dioxus::{desktop::use_window, prelude::*};
use itertools::Itertools;
use xpaste::focus_previous_window;

/// 搜索输入框组件
#[component]
pub fn SearchInput(
    query: Signal<String>,
    is_selecting_bib: Signal<bool>,
    current_bib: Memo<Option<(String, Vec<Reference>)>>,
    #[allow(clippy::type_complexity)] bibs: Memo<
        Vec<(String, String, String, Option<String>, bool)>,
    >,
    input_ref: Signal<Option<MountedEvent>>,
    on_input: EventHandler<Event<FormData>>,
    on_keydown: EventHandler<Event<KeyboardData>>,
    on_bib_select_click: EventHandler<()>,
) -> Element {
    // 当切换到文献库选择模式时，重新获得焦点
    use_effect(move || {
        if is_selecting_bib() {
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(input) = input_ref() {
                    let _ = input.set_focus(true).await;
                }
            });
        }
    });

    rsx! {
        div { class: if (!query().is_empty() && !query().is_empty())
    || (is_selecting_bib() && !bibs().is_empty()) { "relative w-full h-14 bg-transparent rounded-t-lg" } else { "relative w-full h-14 bg-transparent rounded-lg" },
            input {
                class: if (!query().is_empty() && !query().is_empty())
    || (is_selecting_bib() && !bibs().is_empty()) { "w-full h-full pl-5 pr-12 text-lg text-base-content placeholder-base-content/50 font-normal bg-base-100 border-0 rounded-t-lg focus:outline-none" } else { "w-full h-full pl-5 pr-12 text-lg text-base-content placeholder-base-content/50 font-normal bg-base-100 border-0 rounded-lg focus:outline-none" },
                r#type: "text",
                placeholder: if is_selecting_bib() { "选择文献库..." } else { "搜索文献、作者、标题..." },
                value: "{query}",
                oninput: on_input,
                onkeydown: on_keydown,
                autofocus: true,
                onmounted: {
                    move |event| {
                        input_ref.set(Some(event));
                    }
                },
            }
            div { class: "absolute right-3 top-1/2 transform -translate-y-1/2 flex items-center",
                if let Some(bib) = current_bib() {
                    button {
                        class: "btn btn-soft btn-primary btn-sm mr-1",
                        onclick: move |_| on_bib_select_click.call(()),
                        "{bib.0}"
                    }
                } else {
                    button { class: "btn btn-outline btn-primary btn-sm mr-1", "选择文献库" }
                }
                img { class: "opacity-60", width: 50, src: LOGO }
            }
        }
    }
}

// 搜索结果组件
#[component]
fn SearchResults(
    query: Signal<String>,
    result: Signal<Vec<Reference>>,
    keys: Memo<Vec<(String, EntryType)>>,
    selected_index: Signal<Option<usize>>,
    item_elements: Signal<std::collections::HashMap<usize, MountedEvent>>,
    scrollable_container: Signal<Option<MountedEvent>>,
    on_item_click: EventHandler<String>,
    on_container_mounted: EventHandler<MountedEvent>,
) -> Element {
    if result().is_empty() {
        rsx! {
            div { class: "shrink-0 px-5 py-10 text-center text-base-content/60 text-sm",
                "没有找到结果"
            }
        }
    } else {
        rsx! {
            div {
                class: "flex-1 overflow-y-auto",
                style: format!("scroll-behavior: smooth; max-height: {}px;", MAX_HEIGHT - MIN_HEIGHT),
                onmounted: on_container_mounted,
                for (index , ((cite_key , kind) , bib)) in keys().into_iter().zip(result()).enumerate() {
                    div {
                        key: "{index}",
                        "data-item-index": "{index}",
                        class: {
                            let (bg_color, hover_bg_color, border_color) = match kind {
                                EntryType::Article => {
                                    (
                                        "bg-blue-100 dark:bg-blue-900/30",
                                        "hover:bg-blue-100 dark:hover:bg-blue-900/30",
                                        "border-blue-500 dark:border-blue-400",
                                    )
                                }
                                EntryType::Book => {
                                    (
                                        "bg-emerald-100 dark:bg-emerald-900/30",
                                        "hover:bg-emerald-100 dark:hover:bg-emerald-900/30",
                                        "border-emerald-500 dark:border-emerald-400",
                                    )
                                }
                                EntryType::MastersThesis => {
                                    (
                                        "bg-pink-100 dark:bg-pink-900/30",
                                        "hover:bg-pink-100 dark:hover:bg-pink-900/30",
                                        "border-pink-500 dark:border-pink-400",
                                    )
                                }
                                EntryType::Thesis | EntryType::PhdThesis => {
                                    (
                                        "bg-rose-100 dark:bg-rose-900/30",
                                        "hover:bg-rose-100 dark:hover:bg-rose-900/30",
                                        "border-rose-500 dark:border-rose-400",
                                    )
                                }
                                EntryType::InProceedings => {
                                    (
                                        "bg-purple-100 dark:bg-purple-900/30",
                                        "hover:bg-purple-100 dark:hover:bg-purple-900/30",
                                        "border-purple-500 dark:border-purple-400",
                                    )
                                }
                                EntryType::TechReport => {
                                    (
                                        "bg-amber-100 dark:bg-amber-900/30",
                                        "hover:bg-amber-100 dark:hover:bg-amber-900/30",
                                        "border-amber-500 dark:border-amber-400",
                                    )
                                }
                                EntryType::Misc => {
                                    (
                                        "bg-gray-100 dark:bg-gray-900/30",
                                        "hover:bg-gray-100 dark:hover:bg-gray-900/30",
                                        "border-gray-500 dark:border-gray-400",
                                    )
                                }
                                EntryType::Booklet => {
                                    (
                                        "bg-cyan-100 dark:bg-cyan-900/30",
                                        "hover:bg-cyan-100 dark:hover:bg-cyan-900/30",
                                        "border-cyan-500 dark:border-cyan-400",
                                    )
                                }
                                EntryType::InBook => {
                                    (
                                        "bg-teal-100 dark:bg-teal-900/30",
                                        "hover:bg-teal-100 dark:hover:bg-teal-900/30",
                                        "border-teal-500 dark:border-teal-400",
                                    )
                                }
                                EntryType::InCollection => {
                                    (
                                        "bg-fuchsia-100 dark:bg-fuchsia-900/30",
                                        "hover:bg-fuchsia-100 dark:hover:bg-fuchsia-900/30",
                                        "border-fuchsia-500 dark:border-fuchsia-400",
                                    )
                                }
                                _ => {
                                    (
                                        "bg-blue-100 dark:bg-blue-900/30",
                                        "hover:bg-blue-100 dark:hover:bg-blue-900/30",
                                        "border-blue-500 dark:border-blue-400",
                                    )
                                }
                            };
                            if selected_index() == Some(index) {
                                format!(
                                    "block {} rounded-lg text-gray-900 dark:text-gray-100 cursor-pointer transition-colors duration-100 border border-2 {}",
                                    bg_color,
                                    border_color,
                                )
                            } else {
                                format!(
                                    "block {} cursor-pointer hover:rounded-lg transition-colors duration-100 hover:border hover:border-2 {}",
                                    hover_bg_color,
                                    border_color,
                                )
                            }
                        },
                        onmounted: {
                            let mut item_elements = item_elements;
                            move |event| {
                                item_elements.write().insert(index, event);
                            }
                        },
                        onclick: move |_| on_item_click.call(cite_key.clone()),
                        div { class: "flex px-3 py-3 min-h-14",
                            div { class: "flex-1 min-w-0",
                                HelperComponent { entry: bib }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Search() -> Element {
    let mut query = use_signal(String::new);
    let mut result = use_signal(Vec::<Reference>::new);
    let mut is_selecting_bib = use_signal(|| HELPER_BIB().is_none());
    #[allow(clippy::redundant_closure)]
    let current_bib = use_memo(|| HELPER_BIB());
    let mut content_height = use_context::<Signal<usize>>();
    let mut container_mounted = use_signal(|| None::<MountedEvent>);
    let mut scrollable_container = use_signal(|| None::<MountedEvent>);
    let input_ref = use_signal(|| None::<MountedEvent>);
    let mut error_message = use_signal(|| None::<String>);

    let item_elements = use_signal(std::collections::HashMap::<usize, MountedEvent>::new);
    let mut selected_index = use_signal(|| None::<usize>);
    let mut bib_selected_index = use_signal(|| None::<usize>);

    // 获取文献库列表
    let bibs = use_memo(|| {
        let state = crate::STATE.read();
        state
            .bibliographies
            .iter()
            .sorted_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at))
            .map(|(name, info)| {
                (
                    name.clone(),
                    info.path.as_os_str().to_str().unwrap().to_string(),
                    info.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    info.description.clone(),
                    info.path.exists(),
                )
            })
            .collect::<Vec<_>>()
    });

    // 监听 HELPER_BIB 变化，自动切换模式
    use_effect(move || {
        let has_bib = HELPER_BIB().is_some();
        is_selecting_bib.set(!has_bib);
    });

    let keys = use_memo(move || {
        result()
            .iter()
            .map(|item| (item.key(), item.type_.clone()))
            .collect::<Vec<_>>()
    });

    // 事件处理函数
    let handle_input = move |e: Event<FormData>| {
        if !is_selecting_bib() {
            let new_query = e.value();
            query.set(new_query.clone());

            if let Some((_, refs)) = current_bib() {
                let filtered_refs = search_references(&refs, &new_query);
                result.set(filtered_refs);
                selected_index.set(Some(0));
            }
        }
    };

    let handle_bib_select_click = {
        move |_: ()| {
            is_selecting_bib.set(true);
            query.set(String::new());
            result.set(Vec::new());
            selected_index.set(None);
            bib_selected_index.set(Some(0)); // 默认选中第一项
            error_message.set(None);

            // 立即尝试设置焦点
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                if let Some(input) = input_ref() {
                    let _ = input.set_focus(true).await;
                }
            });
        }
    };

    let handle_bib_click = move |(bib_name, bib_path): (String, String)| {
        match parse(&bib_path) {
            Ok(parse_bib) => {
                let refs = read_bibliography(parse_bib);
                set_helper_bib(Some((bib_name, refs)));
                is_selecting_bib.set(false);
                error_message.set(None);

                // 设置焦点到输入框
                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    if let Some(input) = input_ref() {
                        let _ = input.set_focus(true).await;
                    }
                });
            }
            Err(e) => {
                error_message.set(Some(e.to_string()));
            }
        }
    };

    let handle_item_click = move |cite_key: String| {
        HELPER_WINDOW.write().take();
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(cite_key).unwrap();
        let window = use_window();
        window.close();
        HELPER_WINDOW.write().take();
        let _ = focus_previous_window();
    };

    let handle_container_mounted = move |event: MountedEvent| {
        scrollable_container.set(Some(event));
    };

    let _search = move |e: Event<FormData>| {
        query.set(e.value());
        if let Some(bib) = current_bib() {
            let res = search_references(&bib.1, &query());
            result.set(res);
        } else {
            result.set(Vec::new());
        }
        // 重置选中索引
        selected_index.set(None);
    };
    let max_index = use_memo(move || {
        let len = result().len();
        if len > 0 { len - 1 } else { 0 }
    });
    let handle_keydown = move |evt: Event<KeyboardData>| {
        if is_selecting_bib() {
            // 文献库选择模式
            let bib_list = bibs();
            if !bib_list.is_empty() {
                match evt.key() {
                    Key::Enter => {
                        if let Some(index) = bib_selected_index() {
                            let (name, path, _, _, _) = &bib_list[index];
                            // 解析bib
                            match parse(path) {
                                Ok(parsed_bib) => {
                                    let refs = read_bibliography(parsed_bib);
                                    set_helper_bib(Some((name.clone(), refs)));
                                    is_selecting_bib.set(false);
                                    error_message.set(None);
                                }
                                Err(e) => {
                                    error_message.set(Some(e.to_string()));
                                }
                            }
                        }
                    }
                    Key::ArrowDown => {
                        let max_index = if !bib_list.is_empty() {
                            bib_list.len() - 1
                        } else {
                            0
                        };
                        if let Some(index) = bib_selected_index() {
                            let new_index = (index + 1).min(max_index);
                            bib_selected_index.set(Some(new_index));
                        } else {
                            bib_selected_index.set(Some(0));
                        }
                    }
                    Key::ArrowUp => {
                        if let Some(index) = bib_selected_index() {
                            let new_index = if index > 0 { index - 1 } else { 0 };
                            bib_selected_index.set(Some(new_index));
                        }
                    }
                    _ => {}
                }
            }
        } else if !query().is_empty() {
            // 搜索模式
            match evt.key() {
                Key::Enter => {
                    // TODO: 错误处理
                    if let Some(index) = selected_index() {
                        let text = result()[index].cite_key.clone();
                        let mut clipboard = Clipboard::new().unwrap();
                        clipboard.set_text(text.to_string()).unwrap();

                        let window = use_window();
                        window.close();
                        HELPER_WINDOW.write().take();
                        let _ = focus_previous_window();
                    }
                }
                Key::ArrowDown => {
                    evt.prevent_default(); // 阻止默认行为，防止光标移动
                    if let Some(index) = selected_index() {
                        let update_index = (index + 1).min(max_index());
                        selected_index.set(Some(update_index));
                    } else {
                        selected_index.set(Some(0));
                    }
                }
                Key::ArrowUp => {
                    evt.prevent_default(); // 阻止默认行为，防止光标移动
                    if let Some(index) = selected_index() {
                        let update_index = if index > 0 { index - 1 } else { 0 };
                        selected_index.set(Some(update_index));
                    }
                }
                _ => {}
            }
        }
    };

    // 跟随上下方向键滚动
    use_effect(move || {
        if let Some(index) = selected_index()
            && let Some(container) = scrollable_container()
        {
            spawn(async move {
                // 等待刷新
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let mut scroll_successful = false;
                let mut element_refs = Vec::new();
                let target_element_ref;
                {
                    let elements = item_elements.read();
                    for i in 0..index {
                        if let Some(element) = elements.get(&i) {
                            element_refs.push(element.clone());
                        }
                    }
                    target_element_ref = elements.get(&index).cloned();
                }

                let mut cumulative_height = 0.0;

                // 高度
                for element in element_refs {
                    if let Ok(rect) = element.get_client_rect().await {
                        cumulative_height += rect.size.height;
                    } else {
                        cumulative_height += 56.0;
                    }
                }
                let target_height = if let Some(target_element) = target_element_ref {
                    if let Ok(rect) = target_element.get_client_rect().await {
                        rect.size.height
                    } else {
                        56.0
                    }
                } else {
                    56.0
                };

                if let Ok(container_rect) = container.get_client_rect().await {
                    let container_height = container_rect.size.height;
                    let target_center = cumulative_height + (target_height / 2.0);
                    let container_center = container_height / 2.0;
                    let scroll_position = (target_center - container_center).max(0.0);

                    if container
                        .scroll(
                            dioxus::html::geometry::PixelsVector2D::new(0.0, scroll_position),
                            dioxus::html::ScrollBehavior::Smooth,
                        )
                        .await
                        .is_ok()
                    {
                        scroll_successful = true;
                    }
                }

                // 备选方案
                if !scroll_successful {
                    let eval_instance = document::eval(&format!(
                        r#"
                            setTimeout(() => {{
                                const item = document.querySelector('[data-item-index="{index}"]');
                                if (item) {{
                                    item.scrollIntoView({{
                                        behavior: 'smooth',
                                        block: 'nearest',
                                        inline: 'nearest'
                                    }});
                                }}
                            }}, 10);
                            "#
                    ));
                    let _ = eval_instance.await;
                }
            });
        }
    });

    // 动态计算内容高度并更新窗口大小
    use_effect(move || {
        let _ = query();
        let _ = result();
        let _ = is_selecting_bib(); // 监听模式切换
        let _ = error_message();

        if let Some(mounted) = container_mounted() {
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                if let Ok(rect) = mounted.get_client_rect().await {
                    let measured_height = rect.height();
                    let final_height = measured_height
                        .max(MIN_HEIGHT as f64)
                        .min(MAX_HEIGHT as f64)
                        .round() as usize;
                    content_height.set(final_height);
                } else {
                    content_height.set(140);
                }
            });
        }
    });

    // 当查询变化时重置滚动位置
    use_effect(move || {
        let _ = query();
        if let Some(container) = scrollable_container() {
            spawn(async move {
                // 短暂延迟确保DOM更新完成
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                let _ = container
                    .scroll(
                        dioxus::html::geometry::PixelsVector2D::new(0.0, 0.0),
                        dioxus::html::ScrollBehavior::Smooth,
                    )
                    .await;
            });
        }
    });

    rsx! {
        div {
            class: format!("bg-base-100 rounded-xl shadow-2xl flex flex-col h-[{}px]", MAX_HEIGHT),
            "data-content-container": "true",
            onmounted: move |event| {
                container_mounted.set(Some(event));
            },
            // 固定在顶部的搜索输入框
            div { class: "shrink-0",
                SearchInput {
                    query,
                    is_selecting_bib,
                    current_bib,
                    bibs,
                    input_ref,
                    on_input: handle_input,
                    on_keydown: handle_keydown,
                    on_bib_select_click: handle_bib_select_click,
                }
            }
            // 可滚动的内容区域
            div { class: "flex-1 overflow-hidden",
                if is_selecting_bib() {
                    BibliographySelector {
                        bibs,
                        bib_selected_index,
                        on_bib_click: handle_bib_click,
                        error_message,
                    }
                } else if !query().is_empty() {
                    SearchResults {
                        query,
                        result,
                        keys,
                        selected_index,
                        item_elements,
                        scrollable_container,
                        on_item_click: handle_item_click,
                        on_container_mounted: handle_container_mounted,
                    }
                }
            }
        }
    }
}
