use crate::{
    LOGO,
    components::ChunksComp,
    views::{HELPER_BIB, HELPER_WINDOW_OPEN, set_helper_bib},
};
use arboard::Clipboard;
use bibcitex_core::{
    bib::{Reference, parse},
    search_references,
    utils::read_bibliography,
};
use biblatex::EntryType;
use dioxus::{desktop::use_window, prelude::*};

#[component]
pub fn BibItem(name: String, path: String, updated_at: String) -> Element {
    let path_clone = path.clone();
    let name_clone = name.clone();
    let updated_at_clone = updated_at.clone();
    let mut selected_bib = use_context::<Signal<Option<(String, String, String)>>>();
    let mut error_message = use_context::<Signal<Option<String>>>();

    let handle_click = move |_| {
        error_message.set(None);
        match parse(&path_clone) {
            Ok(bib) => {
                let refs = read_bibliography(bib);
                selected_bib.set(Some((
                    name_clone.clone(),
                    path_clone.clone(),
                    updated_at_clone.clone(),
                )));
                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                    set_helper_bib(Some(refs));
                });
            }
            Err(e) => {
                error_message.set(Some(e.to_string()));
            }
        }
    };
    rsx! {
        div { class: "item", onclick: handle_click,
            h3 { {name} }
            p { "{path} ({updated_at})" }
        }
    }
}

#[component]
pub fn SelectBib(bibs: Memo<Vec<(String, String, String)>>) -> Element {
    let error_message = use_context_provider(|| Signal::new(None::<String>));
    let mut selected_bib = use_context_provider(|| Signal::new(None::<(String, String, String)>));
    let mut selected_index = use_signal(|| None::<usize>);
    let mut content_height = use_context::<Signal<f64>>();

    let handle_keydown = move |evt: Event<KeyboardData>| {
        let bib_list = bibs();
        if !bib_list.is_empty() {
            match evt.key() {
                Key::Enter => {
                    if let Some(index) = selected_index() {
                        let (name, path, _) = &bib_list[index];
                        selected_bib.set(Some((name.clone(), path.clone(), "".to_string())));
                        // Load the bibliography here
                        if let Ok(parsed_bib) = parse(path) {
                            let refs = read_bibliography(parsed_bib);
                            set_helper_bib(Some(refs));
                        }
                    }
                }
                Key::ArrowDown => {
                    let max_index = if !bib_list.is_empty() {
                        bib_list.len() - 1
                    } else {
                        0
                    };
                    if let Some(index) = selected_index() {
                        let new_index = (index + 1).min(max_index);
                        selected_index.set(Some(new_index));
                    } else {
                        selected_index.set(Some(0));
                    }
                }
                Key::ArrowUp => {
                    if let Some(index) = selected_index() {
                        let new_index = if index > 0 { index - 1 } else { 0 };
                        selected_index.set(Some(new_index));
                    }
                }
                _ => {}
            }
        }
    };

    let mut container_mounted = use_signal(|| None::<MountedEvent>);

    // 动态计算内容高度并更新窗口大小
    use_effect(move || {
        let _bib_list = bibs();

        if let Some(mounted) = container_mounted() {
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                if let Ok(rect) = mounted.get_client_rect().await {
                    let measured_height = rect.height();
                    let max_height = 460.0;
                    let min_height = 160.0;
                    let final_height = measured_height.max(min_height).min(max_height);
                    content_height.set(final_height + 20.0);
                } else {
                    content_height.set(200.0);
                }
            });
        }
    });

    rsx! {
        div { class: "w-full h-auto bg-transparent", onkeydown: handle_keydown,

            // Floating container with backdrop blur like Spotlight
            div {
                class: "bg-base-100 rounded-xl shadow-2xl overflow-hidden",
                "data-select-container": "true",
                onmounted: move |event| {
                    container_mounted.set(Some(event));
                },
                // Header matching exact Spotlight style
                div { class: "flex items-center px-5 h-14 border-b border-base-300",
                    div { class: "text-lg text-base-content mr-3 font-medium", "BibCiTeX" }
                    div { class: "flex-1 text-lg text-base-content/60", "选择文献库..." }
                }

                div { class: "px-5 py-2 text-xs font-semibold text-base-content/60 uppercase tracking-wider",
                    "Bibliographies"
                }

                if bibs().is_empty() {
                    div { class: "px-5 py-10 text-center text-base-content/60 text-sm",
                        "没有可用的文献库"
                    }
                } else {
                    div { class: "max-h-[448px] overflow-y-auto",
                        for (index , (name , path , updated_at)) in bibs().into_iter().enumerate() {
                            div {
                                class: if selected_index() == Some(index) { "flex items-center px-5 h-14 bg-primary text-primary-content cursor-pointer transition-colors duration-100" } else { "flex items-center px-5 h-14 hover:bg-base-200 cursor-pointer transition-colors duration-100" },
                                onclick: move |_| {
                                    selected_bib.set(Some((name.clone(), path.clone(), updated_at.clone())));
                                    if let Ok(parsed_bib) = parse(&path) {
                                        let refs = read_bibliography(parsed_bib);
                                        set_helper_bib(Some(refs));
                                    }
                                },

                                // Content
                                div { class: "flex-1 min-w-0 mr-3",
                                    div { class: "text-sm font-medium text-base-content truncate",
                                        "{name}"
                                    }
                                    div { class: if selected_index() == Some(index) { "text-xs text-primary-content/70 truncate mt-0.5" } else { "text-xs text-base-content/60 truncate mt-0.5" },
                                        "{path}"
                                    }
                                }
                                div { class: "text-xs text-base-content/50 ml-3 flex-shrink-0",
                                    "{updated_at}"
                                }
                            }
                        }
                    }
                }
            }

            if let Some(error) = error_message() {
                div { class: "px-5 py-2 text-error text-sm", "{error}" }
            }
        }
    }
}

#[component]
pub fn SearchRef() -> Element {
    let mut query = use_signal(String::new);
    let mut result = use_signal(Vec::<Reference>::new);
    let current_bib = HELPER_BIB().unwrap();
    let mut content_height = use_context::<Signal<f64>>();
    let mut container_mounted = use_signal(|| None::<MountedEvent>);
    let mut scrollable_container = use_signal(|| None::<MountedEvent>);
    let item_elements = use_signal(std::collections::HashMap::<usize, MountedEvent>::new);
    let search = move |e: Event<FormData>| {
        query.set(e.value());
        let res = search_references(&current_bib, &query());
        result.set(res);
    };
    let mut selected_index = use_signal(|| None::<usize>);
    let max_index = use_memo(move || {
        let len = result().len();
        if len > 0 { len - 1 } else { 0 }
    });
    let handle_keydown = move |evt: Event<KeyboardData>| {
        if !query().is_empty() {
            match evt.key() {
                Key::Enter => {
                    if let Some(index) = selected_index() {
                        let text = result()[index].cite_key.clone();
                        let window = use_window();
                        window.close();
                        HELPER_WINDOW_OPEN.write().take();
                        let mut clipboard = Clipboard::new().unwrap();
                        clipboard.set_text(text.to_string()).unwrap();
                        // spawn(async move {
                        //     tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        //     if let Err(e) = paste_to_active_app(&text) {
                        //         eprintln!("跨应用粘贴失败: {e}");
                        //     }
                        // });
                    }
                }
                Key::ArrowDown => {
                    if let Some(index) = selected_index() {
                        let update_index = (index + 1).min(max_index());
                        selected_index.set(Some(update_index));
                    } else {
                        selected_index.set(Some(0));
                    }
                }
                Key::ArrowUp => {
                    if let Some(index) = selected_index() {
                        let update_index = if index > 0 { index - 1 } else { 0 };
                        selected_index.set(Some(update_index));
                    }
                }
                _ => {}
            }
        }
    };

    // Scroll the selected item into view using pure Dioxus Rust API
    use_effect(move || {
        if let Some(index) = selected_index() {
            if let Some(container) = scrollable_container() {
                spawn(async move {
                    // Wait a short time for the DOM to update
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                    let mut scroll_successful = false;

                    // Calculate cumulative height using Dioxus API
                    // Collect all element references before async operations
                    let mut element_refs = Vec::new();
                    let target_element_ref;
                    {
                        let elements = item_elements.read();

                        // Collect elements before target index
                        for i in 0..index {
                            if let Some(element) = elements.get(&i) {
                                element_refs.push(element.clone());
                            }
                        }

                        // Get target element reference
                        target_element_ref = elements.get(&index).cloned();
                    }

                    let mut cumulative_height = 0.0;

                    // Get heights of all elements before target index using get_client_rect
                    for element in element_refs {
                        if let Ok(rect) = element.get_client_rect().await {
                            cumulative_height += rect.size.height;
                        } else {
                            // Use fallback height if get_client_rect fails
                            cumulative_height += 56.0;
                        }
                    }

                    // Get target element height for centering
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

                        // Calculate scroll position to center the target element
                        let target_center = cumulative_height + (target_height / 2.0);
                        let container_center = container_height / 2.0;
                        let scroll_position = (target_center - container_center).max(0.0);

                        // Use Dioxus native scroll with calculated position
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

                    // If Dioxus native scroll fails, fallback to JavaScript scrollIntoView
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
        }
    });

    // 动态计算内容高度并更新窗口大小
    use_effect(move || {
        let _query_val = query();
        let _result_val = result();

        if let Some(mounted) = container_mounted() {
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                if let Ok(rect) = mounted.get_client_rect().await {
                    let measured_height = rect.height();
                    let max_height = 460.0;
                    let min_height = 80.0;
                    let final_height = measured_height.max(min_height).min(max_height);
                    content_height.set(final_height + 20.0);
                } else {
                    content_height.set(140.0);
                }
            });
        }
    });

    rsx! {
        div { class: "w-full h-auto bg-transparent",
            div {
                class: "bg-base-100 rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[460px]",
                "data-content-container": "true",
                onmounted: move |event| {
                    container_mounted.set(Some(event));
                },
                div {
                    class: "flex-shrink-0 overflow-hidden no-scroll",
                    style: "overscroll-behavior: none;",
                    onwheel: move |evt| {
                        evt.prevent_default();
                        evt.stop_propagation();
                    },
                    onscroll: move |evt| {
                        evt.prevent_default();
                        evt.stop_propagation();
                    },
                    div { class: if !query().is_empty() && !result().is_empty() { "relative w-full h-14 bg-transparent rounded-t-lg" } else { "relative w-full h-14 bg-transparent rounded-lg" },
                        input {
                            class: if !query().is_empty() && !result().is_empty() { "w-full h-full pl-5 pr-12 text-lg text-base-content placeholder-base-content/50 font-normal bg-base-100 border-0 rounded-t-lg focus:outline-none" } else { "w-full h-full pl-5 pr-12 text-lg text-base-content placeholder-base-content/50 font-normal bg-base-100 border-0 rounded-lg focus:outline-none" },
                            r#type: "text",
                            placeholder: "搜索文献、作者、标题...",
                            value: "{query}",
                            oninput: search,
                            onkeydown: handle_keydown,
                            autofocus: true,
                        }
                        div { class: "absolute right-3 top-1/2 transform -translate-y-1/2 flex items-center",
                            img { class: "opacity-60", width: 50, src: LOGO }
                        }
                    }
                }
                if !query().is_empty() {
                    // Scrollable results area
                    if result().is_empty() {
                        div { class: "flex-shrink-0 px-5 py-10 text-center text-base-content/60 text-sm",
                            "没有找到结果"
                        }
                    } else {
                        div {
                            class: "flex-1 overflow-y-auto",
                            style: "scroll-behavior: smooth; max-height: 400px;",
                            onmounted: move |event| {
                                scrollable_container.set(Some(event));
                            },
                            for (index , bib) in result().into_iter().enumerate() {
                                div {
                                    key: "{index}",
                                    "data-item-index": "{index}",
                                    class: if selected_index() == Some(index) { "block bg-success rounded-lg text-primary-content cursor-pointer transition-colors duration-100" } else { "block hover:bg-base-200 cursor-pointer transition-colors duration-100" },
                                    onmounted: {
                                        let mut item_elements = item_elements;
                                        move |event| {
                                            item_elements.write().insert(index, event);
                                        }
                                    },
                                    onclick: move |_| {
                                        let text = bib.cite_key.clone();
                                        let window = use_window();
                                        window.close();
                                        HELPER_WINDOW_OPEN.write().take();
                                        let mut clipboard = Clipboard::new().unwrap();
                                        clipboard.set_text(text.to_string()).unwrap();
                                    },

                                    div { class: "flex px-3 py-3 min-h-[56px]",
                                        div { class: "flex-1 min-w-0",
                                            match bib.type_ {
                                                EntryType::Article => rsx! {
                                                    ArticleHelper { entry: bib.clone() }
                                                },
                                                EntryType::Book => rsx! {
                                                    BookHelper { entry: bib.clone() }
                                                },
                                                EntryType::Thesis | EntryType::MastersThesis | EntryType::PhdThesis => {
                                                    rsx! {
                                                        ThesisHelper { entry: bib.clone() }
                                                    }
                                                }
                                                _ => rsx! {
                                                    ArticleHelper { entry: bib.clone() }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ArticleHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;
    rsx! {
        div {
            div { class: "flex justify-between items-start",
                div { class: "flex items-start",
                    div { class: "mr-2 text-blue-800", "Article" }
                    if let Some(title) = entry.title {
                        span { class: " text-gray-900 font-serif",
                            ChunksComp { chunks: title, cite_key: key.clone() }
                        }
                    } else {
                        span { "No title available" }
                    }
                }
                div { class: "flex items-center flex-shrink-0",
                    div { class: "text-gray-600 text-xs font-mono ml-2", "{key}" }
                }
            }
            p { class: "text-xs",
                if let Some(authors) = entry.author {
                    for author in authors {
                        span { class: "text-blue-700 font-semibold mr-2", "{author}" }
                    }
                } else {
                    span { class: "text-blue-700 font-semibold mr-1", "Unknown" }
                }
                if let Some(journal) = &entry.journal {
                    span { class: "text-purple-600 mr-2", "{journal}" }
                } else {
                    span { class: "text-purple-600 mr-2", "journal" }
                }
                if let Some(year) = &entry.year {
                    span { class: "text-emerald-700 mr-2", "{year}" }
                } else {
                    span { class: "text-emerald-700 mr-1", "year" }
                }
            }
        }
    }
}

#[component]
pub fn BookHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;
    rsx! {
        div {
            div { class: "flex justify-between items-start",
                div { class: "flex items-start",
                    div { class: "mr-2 text-emerald-800", "Book" }
                    if let Some(title) = entry.title {
                        span { class: " text-gray-900 font-serif",
                            ChunksComp { chunks: title, cite_key: key.clone() }
                        }
                    } else {
                        span { "No title available" }
                    }
                }
                div { class: "flex items-center flex-shrink-0",
                    div { class: "text-gray-600 text-xs font-mono ml-2", "{key}" }
                }
            }
            p { class: "text-xs",
                if let Some(authors) = entry.author {
                    for author in authors {
                        span { class: "text-blue-700 font-semibold mr-2", "{author} " }
                    }
                } else {
                    span { class: "text-blue-700 font-semibold mr-1", "Unknown" }
                }
                if let Some(publishers) = &entry.publisher {
                    for publisher in publishers {
                        span { class: "text-purple-600 mr-2", "{publisher}" }
                    }
                } else {
                    span { class: "text-purple-600 mr-2", "Unknown" }
                }
                if let Some(year) = &entry.year {
                    span { class: "text-emerald-700 mr-2", "{year}" }
                } else {
                    span { class: "text-emerald-700 mr-1", "year" }
                }
            }
        }
    }
}

#[component]
pub fn ThesisHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;

    let school_address = {
        if let Some(school) = entry.school {
            if let Some(address) = entry.address {
                format!("{school} ({address})")
            } else {
                school
            }
        } else {
            "".to_string()
        }
    };
    let type_ = match entry.type_ {
        EntryType::Thesis => "Thesis",
        EntryType::MastersThesis => "Master Thesis",
        EntryType::PhdThesis => "PhD Thesis",
        _ => "Unknown",
    };
    rsx! {
        div {
            div { class: "flex justify-between items-start",
                div { class: "flex items-start",
                    div { class: if entry.type_ == EntryType::MastersThesis { "mr-2 text-pink-800" } else { "mr-2 text-rose-800" },
                        "{type_}"
                    }
                    if let Some(title) = entry.title {
                        span { class: " text-gray-900 font-serif",
                            ChunksComp { chunks: title, cite_key: key.clone() }
                        }
                    } else {
                        span { "No title available" }
                    }
                }
                div { class: "flex items-center flex-shrink-0",
                    div { class: "text-gray-600 text-xs font-mono ml-2", "{key}" }
                }
                p { class: "text-xs",
                    if let Some(authors) = entry.author {
                        for author in authors {
                            span { class: if entry.type_ == EntryType::MastersThesis { "text-blue-700 font-semibold mr-2" } else { "text-blue-700 font-semibold mr-2" },
                                "{author} "
                            }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-1", "Unknown" }
                    }
                    if !school_address.is_empty() {
                        span { class: "text-purple-600 mr-2", "{school_address}" }
                    } else {
                        span { class: "text-purple-600 mr-2", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        span { class: "text-emerald-700 mr-1", "year" }
                    }
                }
            }
        }
    }
}
