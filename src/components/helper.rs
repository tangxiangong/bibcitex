use crate::{
    components::ChunksComp,
    views::{HELPER_BIB, HELPER_WINDOW_OPEN, set_helper_bib},
};
use arboard::Clipboard;
use bibcitex_core::{
    MSC_MAP,
    bib::{Reference, parse},
    parse_code, search_references,
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

    rsx! {
        div { class: "w-full h-auto bg-transparent", onkeydown: handle_keydown,

            // Floating container with backdrop blur like Spotlight
            div { class: "bg-base-100/90 backdrop-blur-xl rounded-xl shadow-2xl overflow-hidden",
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
    let mut query = use_context::<Signal<String>>();
    let mut result = use_signal(Vec::<Reference>::new);
    let current_bib = HELPER_BIB().unwrap();
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

    // Pure Rust scrolling implementation
    let scroll_top = use_signal(|| 0.0);
    // Calculate real heights based on actual CSS:
    // .helper-item: padding: 8px (top) + 8px (bottom) = 16px
    // .helper-item: margin: 2px (top) + 2px (bottom) = 4px
    // .entry: margin: 10px (top) + 10px (bottom) = 20px
    // Estimated text content height: ~20px
    // Total: 16 + 4 + 20 + 20 + 20 = 80px per item
    let item_height = 80.0;

    use_effect(move || {
        if let Some(index) = selected_index() {
            // Simply scroll to the selected item position, no assumptions
            let target_scroll = index as f64 * item_height;
            // Use spawn to break the reactive cycle
            let mut scroll_top = scroll_top;
            spawn(async move {
                scroll_top.set(target_scroll);
            });
        }
    });

    rsx! {
        div { class: "w-full h-auto bg-transparent",
            if !query().is_empty() {
                // Complete connected container - input fixed at top, results scroll below
                div { class: "bg-base-100/90 backdrop-blur-xl rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[460px]",
                    // Fixed input at top
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
                        input {
                            class: "w-full h-14 px-5 text-lg text-base-content placeholder-base-content/50 font-normal bg-transparent border-0 focus:outline-none",
                            r#type: "text",
                            placeholder: "搜索文献、作者、标题...",
                            value: "{query}",
                            oninput: search,
                            onkeydown: handle_keydown,
                            autofocus: true,
                        }
                    }

                    // Results header
                    div {
                        class: "flex-shrink-0 px-5 py-2 text-xs font-semibold text-base-content/60 uppercase tracking-wider border-t border-base-300 overflow-hidden no-scroll",
                        style: "overscroll-behavior: none;",
                        onwheel: move |evt| {
                            evt.prevent_default();
                            evt.stop_propagation();
                        },
                        onscroll: move |evt| {
                            evt.prevent_default();
                            evt.stop_propagation();
                        },
                        "Results"
                    }

                    // Scrollable results area
                    if result().is_empty() {
                        div { class: "flex-shrink-0 px-5 py-10 text-center text-base-content/60 text-sm",
                            "没有找到结果"
                        }
                    } else {
                        div { class: "flex-1 overflow-y-auto",
                            for (index , bib) in result().into_iter().enumerate() {
                                div {
                                    id: format!("helper-item-{}", index),
                                    class: if selected_index() == Some(index) { "block bg-primary text-primary-content cursor-pointer transition-colors duration-100" } else { "block hover:bg-base-200 cursor-pointer transition-colors duration-100" },
                                    onclick: move |_| {
                                        let text = bib.cite_key.clone();
                                        let window = use_window();
                                        window.close();
                                        HELPER_WINDOW_OPEN.write().take();
                                        let mut clipboard = Clipboard::new().unwrap();
                                        clipboard.set_text(text.to_string()).unwrap();
                                    },

                                    div { class: "flex items-start px-5 py-3 min-h-[56px]",
                                        // Content area with ArticleHelper components
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
            } else {
                // Standalone input when no query
                div {
                    class: "bg-base-100/90 backdrop-blur-xl rounded-xl shadow-2xl overflow-hidden no-scroll",
                    style: "overscroll-behavior: none;",
                    onwheel: move |evt| {
                        evt.prevent_default();
                        evt.stop_propagation();
                    },
                    onscroll: move |evt| {
                        evt.prevent_default();
                        evt.stop_propagation();
                    },
                    input {
                        class: "w-full h-14 px-5 text-lg text-base-content placeholder-base-content/50 font-normal bg-transparent border-0 focus:outline-none",
                        r#type: "text",
                        placeholder: "搜索文献、作者、标题...",
                        value: "{query}",
                        oninput: search,
                        onkeydown: handle_keydown,
                        autofocus: true,
                    }
                }
            }
        }
    }
}

#[component]
pub fn ArticleHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;

    let msc_text = if let Some(ref raw) = entry.mrclass {
        if MSC_MAP.is_empty() {
            Vec::new()
        } else {
            let codes = parse_code(raw);
            let mut texts = Vec::with_capacity(codes.len() + 1);
            for code in codes {
                if let Some(text) = MSC_MAP.get(&code) {
                    texts.push(text.clone());
                }
            }
            texts
        }
    } else {
        Vec::new()
    };

    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };

    rsx! {
        div { class: "bg-blue-100 border-blue-500 border-l-4",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-blue-800", "Article" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-grey-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
                        div { class: "text-gray-600 text-xs font-mono ml-2", "{key}" }
                    }
                }
                p {
                    if let Some(authors) = entry.author {
                        if authors.len() > 3 {
                            for author in authors.iter().take(3) {
                                span { class: "text-blue-700 font-semibold mr-2", "{author}" }
                            }
                            span { class: "font-semibold mr-1", " et al." }
                        } else {
                            for author in authors {
                                span { class: "text-blue-700 font-semibold bg-blue-100 mr-2",
                                    "{author} "
                                }
                            }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-1", "Unknown" }
                    }
                }
                p {
                    if let Some(journal) = &entry.journal {
                        span { class: "text-purple-600 mr-2", "{journal}" }
                    } else {
                        span { class: "text-purple-600 mr-2", "Unknown" }
                    }
                    if let Some(year) = &entry.year {
                        span { class: "text-emerald-700 mr-2", "{year}" }
                    } else {
                        span { class: "text-emerald-700 mr-1", "year" }
                    }
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
                            }
                        }
                    }
                    if let Some(file) = entry.file {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "打开文献",
                            onclick: move |_| {
                                let _ = opener::open(&file);
                            },
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }

                    if let Some(mrclass) = entry.mrclass {
                        div { class: "tooltip",
                            div { class: "tooltip-content",
                                if msc_text.is_empty() {
                                    p { "The MR Class code is not available" }
                                } else {
                                    for text in msc_text {
                                        p { class: "text-left", "{text}" }
                                    }
                                }
                            }
                            div { class: "text-red-500 cursor-pointer", "{mrclass}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn BookHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;

    let msc_text = if let Some(ref raw) = entry.mrclass {
        if MSC_MAP.is_empty() {
            Vec::new()
        } else {
            let codes = parse_code(raw);
            let mut texts = Vec::with_capacity(codes.len() + 1);
            for code in codes {
                if let Some(text) = MSC_MAP.get(&code) {
                    texts.push(text.clone());
                }
            }
            texts
        }
    } else {
        Vec::new()
    };

    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };

    rsx! {
        div { class: "bg-emerald-100 border-emerald-500 border-l-4",
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: "mr-2 text-lg text-emerald-800", "Book" }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-grey-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
                        div { class: "text-gray-600 text-xs font-mono ml-2", "{key}" }
                    }
                }
                p {
                    if let Some(authors) = entry.author {
                        if authors.len() > 3 {
                            for author in authors.iter().take(3) {
                                span { class: "text-blue-700 font-semibold mr-2", "{author}" }
                            }
                            span { class: "font-semibold mr-1", " et al." }
                        } else {
                            for author in authors {
                                span { class: "text-blue-700 font-semibold bg-emerald-100 mr-2",
                                    "{author} "
                                }
                            }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-1", "Unknown" }
                    }
                }
                p {
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
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
                            }
                        }
                    }
                    if let Some(file) = entry.file {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "打开文献",
                            onclick: move |_| {
                                let _ = opener::open(&file);
                            },
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }

                    if let Some(mrclass) = entry.mrclass {
                        div { class: "tooltip",
                            div { class: "tooltip-content",
                                if msc_text.is_empty() {
                                    p { "The MR Class code is not available" }
                                } else {
                                    for text in msc_text {
                                        p { class: "text-left", "{text}" }
                                    }
                                }
                            }
                            div { class: "text-red-500 cursor-pointer", "{mrclass}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ThesisHelper(entry: Reference) -> Element {
    let key = &entry.cite_key;

    let doi_url = if let Some(doi) = entry.doi.clone() {
        format!("https://doi.org/{doi}")
    } else {
        "".to_string()
    };

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
        div { class: if entry.type_ == EntryType::MastersThesis { "bg-pink-100 border-pink-500 border-l-4" } else { "bg-rose-100 border-rose-500 border-l-4" },
            div { class: "card-body",
                div { class: "flex justify-between items-start",
                    div { class: "flex items-start",
                        div { class: if entry.type_ == EntryType::MastersThesis { "mr-2 text-lg text-pink-800" } else { "mr-2 text-lg text-rose-800" },
                            "{type_}"
                        }
                        if let Some(title) = entry.title {
                            span { class: "text-lg text-grey-900 font-serif",
                                ChunksComp { chunks: title, cite_key: key.clone() }
                            }
                        } else {
                            span { class: "text-lg", "No title available" }
                        }
                    }
                    div { class: "flex items-center flex-shrink-0",
                        div { class: "text-gray-600 text-xs font-mono ml-2", "{key}" }
                    }
                }
                p {
                    if let Some(authors) = entry.author {
                        if authors.len() > 3 {
                            for author in authors.iter().take(3) {
                                span { class: "text-blue-700 font-semibold mr-2", "{author}" }
                            }
                            span { class: "font-semibold mr-1", " et al." }
                        } else {
                            for author in authors {
                                span { class: if entry.type_ == EntryType::MastersThesis { "text-blue-700 font-semibold bg-pink-100 mr-2" } else { "text-blue-700 font-semibold bg-rose-100 mr-2" },
                                    "{author} "
                                }
                            }
                        }
                    } else {
                        span { class: "text-blue-700 font-semibold mr-1", "Unknown" }
                    }
                }
                p {
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
                    if let Some(doi) = &entry.doi {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "在浏览器中打开",
                            onclick: move |_| {
                                let _ = opener::open_browser(&doi_url);
                            },
                            div { class: "text-cyan-600 mr-2", "DOI: {doi}" }
                        }
                    } else {
                        if let Some(url) = entry.url {
                            button {
                                class: "tooltip cursor-pointer",
                                "data-tip": "在浏览器中打开",
                                onclick: move |_| {
                                    let _ = opener::open_browser(&url);
                                },
                                div { class: "text-cyan-600 mr-2", "URL" }
                            }
                        }
                    }
                    if let Some(file) = entry.file {
                        button {
                            class: "tooltip cursor-pointer",
                            "data-tip": "打开文献",
                            onclick: move |_| {
                                let _ = opener::open(&file);
                            },
                            div { class: "text-amber-700 mr-2", "PDF" }
                        }
                    }
                }
            }
        }
    }
}
