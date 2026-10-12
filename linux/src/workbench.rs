use crate::icons::icon;
use crate::{editor::LibraryEditor, preferences::Preferences};
use bibcitex_service::{self as service, ChunkRecord, LibraryRecord, ReferenceRecord};
use gpui_kit::component::{
    ActiveTheme, Disableable, Sizable, TitleBar,
    button::{Button, ButtonVariants},
    input::{self, Input, InputEvent, InputState},
    menu::{ContextMenuExt, DropdownMenu, PopupMenuItem},
    resizable::{h_resizable, resizable_panel},
    spinner::Spinner,
    tooltip::Tooltip,
};
use gpui_kit::{prelude::FluentBuilder as _, *};
use std::time::Duration;

#[path = "search_ui.rs"]
mod search_ui;
use search_ui::{CandidatePicker, ConditionEditor, OptionalFieldValue, TagManager};

type LinkAction = Box<dyn Fn(&mut Workbench, &mut Context<Workbench>)>;

enum LinkTarget {
    /// An HTTP(S) address; `None` when the value is not one, so it stays plain text.
    Url(&'static str, Option<url::Url>),
    Action(&'static str, LinkAction),
}

fn external_target(label: &str, value: &str) -> LinkTarget {
    let (name, address) = if label == "DOI" {
        let address = if value.starts_with("https://") || value.starts_with("http://") {
            value.to_string()
        } else {
            format!("https://doi.org/{value}")
        };
        ("打开 DOI", address)
    } else {
        ("打开 URL", value.to_string())
    };
    let url = url::Url::parse(&address)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https"));
    LinkTarget::Url(name, url)
}

pub const TYPES: &[(&str, &str)] = &[
    ("all", "全部类型"),
    ("Article", "期刊论文"),
    ("Book", "图书"),
    ("Thesis", "学位论文"),
    ("TechReport", "技术报告"),
    ("Misc", "其他"),
    ("Booklet", "小册子"),
    ("InBook", "书籍章节"),
    ("InCollection", "文集章节"),
    ("InProceedings", "会议论文"),
];
const FIELDS: &[(&str, &str)] = &[
    ("all", "全部字段"),
    ("author", "作者"),
    ("title", "标题"),
    ("journal", "期刊"),
    ("year", "年份"),
];

/// Kept in step with the macOS helper and tray metrics.
const HELPER_HEADER: f32 = 64.;
const HELPER_RADIUS: f32 = 26.;
const TRAY_RADIUS: f32 = 20.;
const TRAY_DETAIL_RADIUS: f32 = 16.;
/// One rounded selection shape for every list row.
const ROW_RADIUS: f32 = 10.;
/// The floating helper controls overlay the list and take no height of their own.
const HELPER_BOTTOM_BAR: f32 = 52.;

pub fn chunks(chunks: &[ChunkRecord]) -> String {
    chunks
        .iter()
        .map(|c| match c.kind {
            service::ChunkKind::Math => format!("${}$", c.text),
            _ => c.text.clone(),
        })
        .collect()
}

fn selectable_text(id: impl Into<ElementId>, value: &str) -> impl IntoElement {
    let escaped = value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br>");
    gpui_kit::component::text::TextView::html(id, format!("<div>{escaped}</div>")).selectable(true)
}

#[derive(Default)]
struct SearchDraft {
    text: String,
    content: Option<input::InputContent>,
    editor: Option<ConditionEditor>,
    field: usize,
    kind: usize,
    document: Option<service::QueryDocumentRecord>,
    tag_revision: Option<u64>,
}

pub struct Workbench {
    advanced: bool,
    inactive_draft: SearchDraft,
    library_draft: Option<SearchDraft>,
    condition_editor: Option<ConditionEditor>,
    query_document: Option<service::QueryDocumentRecord>,
    tag_revision: Option<u64>,
    result_context: Option<service::SearchContext>,
    evidence: Vec<service::SearchEvidence>,
    tag_manager: Option<TagManager>,
    candidates: Option<CandidatePicker>,
    candidate_window: Option<AnyWindowHandle>,
    helper_scope: Option<String>,
    optional_field_value: Option<OptionalFieldValue>,
    pending_trigger: Option<(char, String)>,
    pending_query_projection: Option<(u64, String)>,
    search_composing: bool,
    ime_active: std::collections::HashSet<EntityId>,
    ime_settling: std::collections::HashSet<EntityId>,
    ime_watch_task: Option<Task<()>>,
    query_diagnostics: Vec<service::QueryDiagnostic>,
    query_diagnostic_text: String,
    query_notice: Option<String>,
    native_tag_bindings: std::collections::HashMap<String, String>,
    native_tag_serial: u64,
    pub preferences: Entity<Preferences>,
    search: Entity<InputState>,
    libraries: Vec<LibraryRecord>,
    selected_library: Option<String>,
    references: Vec<ReferenceRecord>,
    selected: Option<usize>,
    field: usize,
    kind: usize,
    loading: bool,
    error: Option<String>,
    generation: u64,
    registry_generation: u64,
    search_task: Option<Task<()>>,
    copied: Option<String>,
    copied_detail: Option<String>,
    copy_task: Option<Task<()>>,
    copy_detail_task: Option<Task<()>>,
    scroll: UniformListScrollHandle,
    helper: bool,
    tray: bool,
    tray_detail: bool,
    pasting: bool,
    failed_paste_key: Option<String>,
    choosing_library: bool,
    library_cursor: usize,
    library_scroll: ScrollHandle,
    editor_window: Option<AnyWindowHandle>,
    renaming: Option<(String, Entity<InputState>)>,
    rename_subscription: Option<Subscription>,
    saving_name: bool,
    placeholder_selecting: bool,
    _subscriptions: Vec<Subscription>,
}

impl Workbench {
    fn search_navigation(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.search.read(cx).focus_handle(cx).is_focused(window)
            && !self.search.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            })
    }
    pub fn paste_in_progress(&self) -> bool {
        self.pasting
    }

    fn begin_rename(
        &mut self,
        library: LibraryRecord,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.saving_name {
            return;
        }
        let input = cx.new(|cx| InputState::new(window, cx));
        input.update(cx, |input, cx| {
            input.set_value(library.name.clone(), window, cx);
            input.focus(window, cx);
        });
        self.rename_subscription =
            Some(cx.subscribe_in(&input, window, |this, _, event, _, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.save_name(cx);
                }
            }));
        self.renaming = Some((library.name, input));
        cx.notify();
    }

    fn save_name(&mut self, cx: &mut Context<Self>) {
        if self.saving_name {
            return;
        }
        let Some((old_name, input)) = &self.renaming else {
            return;
        };
        let name = input.read(cx).value().trim().to_owned();
        if name.is_empty() {
            self.error = Some(self.text("文献库名称不能为空", cx).to_string());
            cx.notify();
            return;
        }
        if name == *old_name {
            self.renaming = None;
            self.rename_subscription = None;
            cx.notify();
            return;
        }
        let old_name = old_name.clone();
        self.saving_name = true;
        let job = cx.background_executor().spawn(async move {
            let record = service::update_library(old_name.clone(), name, None, None)?;
            Ok::<_, service::CoreError>((old_name, record.name))
        });
        cx.spawn(async move |this, cx| {
            let result = job.await;
            let _ = this.update(cx, |this, cx| {
                this.saving_name = false;
                match result {
                    Ok((old_name, name)) => {
                        this.renaming = None;
                        this.rename_subscription = None;
                        this.library_saved(Some(&old_name), name, cx);
                    }
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    pub fn new(
        preferences: Entity<Preferences>,
        helper: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx).placeholder(preferences.read(cx).text(if helper {
                "搜索文献、作者、标题"
            } else {
                "搜索文献"
            }))
        });
        let ime_subscription = cx.observe_in(&search, window, |this, _, window, cx| {
            this.synchronize_ime(window, cx)
        });
        let appearance_subscription = cx.observe_window_appearance(window, |this, _, cx| {
            let prefs = this.preferences.read(cx).clone();
            if prefs.appearance == "system" {
                prefs.apply_theme(cx);
            }
        });
        let input_subscription =
            cx.subscribe_in(&search, window, |this, _, event, window, cx| match event {
                InputEvent::Change => {
                    this.search_composing = this.search.update(cx, |input, cx| {
                        input.marked_text_range(window, cx).is_some()
                    });
                    if !this.search_composing
                        && this.pending_trigger.is_none()
                        && !this.choosing_library
                    {
                        this.rebind_native_tokens(cx);
                    }
                    this.library_cursor = 0;
                    this.failed_paste_key = None;
                    this.error = None;
                    if !this.search_composing {
                        this.handle_candidate_change(window, cx);
                    }
                    this.schedule_search(cx);
                    this.resize_helper(window, cx);
                }
                InputEvent::PressEnter { .. } if this.helper => {
                    if this.input_is_composing(window, cx) {
                        return;
                    }
                    if this.candidates.is_some() {
                        this.confirm_candidate(window, cx);
                        return;
                    }
                    if this.condition_editor.is_some()
                        || this.optional_field_value.is_some()
                        || this.tag_manager.is_some()
                    {
                        return;
                    }
                    if this.choosing_library {
                        if let Some(name) = this
                            .matching_libraries(cx)
                            .get(this.library_cursor)
                            .filter(|l| l.available)
                            .map(|l| l.name.clone())
                        {
                            this.select_library(name, cx);
                            let placeholder = this.text("搜索文献、作者、标题", cx);
                            this.search.update(cx, |input, cx| {
                                input.set_placeholder(placeholder, window, cx);
                            });
                            this.restore_library_draft(window, cx);
                            this.resize_helper(window, cx);
                        }
                    } else {
                        this.activate_selected(window, cx);
                    }
                }
                _ => {}
            });
        let preference_subscription = cx.observe_in(&preferences, window, |this, _, window, cx| {
            let placeholder = this.text(
                if this.helper && (this.candidates.is_some() || this.optional_field_value.is_some())
                {
                    ""
                } else if this.choosing_library {
                    "搜索或选择文献库"
                } else if this.helper {
                    "搜索文献、作者、标题"
                } else {
                    "搜索文献"
                },
                cx,
            );
            this.search.update(cx, |input, cx| {
                input.set_placeholder(placeholder, window, cx)
            });
            this.schedule_search(cx);
            this.reload(cx);
            cx.notify();
        });
        if helper {
            search.update(cx, |input, cx| input.focus(window, cx));
        }
        let updater = cx.global::<crate::application::Services>().updater.clone();
        let update_subscription = cx.observe(&updater, |_, _, cx| cx.notify());
        let activation_subscription = cx.observe_window_activation(window, |this, window, cx| {
            if this.input_is_composing(window, cx) {
                return;
            }
            if this.helper && !window.is_window_active() && this.candidates.is_some() {
                cx.spawn_in(window, async move |this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(30))
                        .await;
                    let _ = this.update_in(cx, |this, window, cx| {
                        if this.input_is_composing(window, cx) {
                            return;
                        }
                        let popup_active = this.candidate_window.is_some_and(|handle| {
                            handle
                                .update(cx, |_, popup, _| popup.is_window_active())
                                .unwrap_or(false)
                        });
                        if !window.is_window_active() && !popup_active {
                            this.candidates = None;
                            this.close_candidate_window(cx);
                            window.remove_window();
                        }
                    });
                })
                .detach();
                return;
            }
            if (this.helper || this.tray)
                && !window.is_window_active()
                && !this.pasting
                && this.tag_manager.is_none()
                && this.condition_editor.is_none()
                && this.candidates.is_none()
            {
                window.remove_window();
            }
        });
        let selected_library = preferences.read(cx).library.clone();
        let advanced = helper || preferences.read(cx).main_advanced_search_enabled;
        let mut this = Self {
            advanced,
            inactive_draft: SearchDraft::default(),
            library_draft: None,
            condition_editor: None,
            query_document: None,
            tag_revision: None,
            result_context: None,
            evidence: vec![],
            tag_manager: None,
            candidates: None,
            candidate_window: None,
            helper_scope: None,
            optional_field_value: None,
            pending_trigger: None,
            pending_query_projection: None,
            search_composing: false,
            ime_active: std::collections::HashSet::new(),
            ime_settling: std::collections::HashSet::new(),
            ime_watch_task: None,
            query_diagnostics: vec![],
            query_diagnostic_text: String::new(),
            query_notice: None,
            native_tag_bindings: std::collections::HashMap::new(),
            native_tag_serial: 0,
            preferences,
            search,
            libraries: vec![],
            selected_library,
            references: vec![],
            selected: None,
            field: 0,
            kind: 0,
            loading: false,
            error: None,
            generation: 0,
            registry_generation: 0,
            search_task: None,
            copied: None,
            copied_detail: None,
            copy_task: None,
            copy_detail_task: None,
            scroll: UniformListScrollHandle::new(),
            helper,
            tray: false,
            tray_detail: false,
            pasting: false,
            failed_paste_key: None,
            choosing_library: false,
            library_cursor: 0,
            library_scroll: ScrollHandle::new(),
            editor_window: None,
            renaming: None,
            rename_subscription: None,
            saving_name: false,
            placeholder_selecting: false,
            _subscriptions: vec![
                input_subscription,
                ime_subscription,
                preference_subscription,
                appearance_subscription,
                update_subscription,
                activation_subscription,
            ],
        };
        this._subscriptions.push(cx.on_release(|this, cx| {
            this.close_candidate_window(cx);
        }));
        this.reload(cx);
        this
    }

    pub fn new_tray(
        preferences: Entity<Preferences>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut view = Self::new(preferences, false, window, cx);
        view.tray = true;
        view.advanced = view.preferences.read(cx).tray_advanced_search_enabled;
        view.selected_library = view.preferences.read(cx).tray_library.clone();
        view.reload(cx);
        view.focus_search(window, cx);
        view
    }
    pub fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }
    pub fn report_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.error = Some(error);
        cx.notify();
    }
    pub fn paste_failed(&mut self, error: String, key: String, cx: &mut Context<Self>) {
        self.error = Some(error);
        self.failed_paste_key = Some(key);
        cx.notify();
    }

    pub fn text(&self, key: &str, cx: &App) -> SharedString {
        self.preferences.read(cx).text(key)
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.registry_generation += 1;
        let revision = self.registry_generation;
        let helper = self.helper;
        let job = cx.background_executor().spawn(async move {
            let libraries = service::libraries().map_err(|e| e.to_string())?;
            let current = if helper {
                service::helper_current()
                    .map_err(|e| e.to_string())?
                    .filter(|l| l.available)
                    .map(|l| l.name)
            } else {
                None
            };
            Ok::<_, String>((libraries, current))
        });
        cx.spawn(async move |this, cx| {
            let result = job.await;
            let _ = this.update(cx, |this, cx| {
                if this.registry_generation != revision {
                    return;
                }
                match result {
                    Ok((libraries, current)) => {
                        let selected_path = this
                            .libraries
                            .iter()
                            .find(|l| Some(&l.name) == this.selected_library.as_ref())
                            .map(|l| l.path.clone());
                        this.libraries = libraries;
                        if helper {
                            this.selected_library = current.filter(|name| {
                                this.libraries
                                    .iter()
                                    .any(|l| &l.name == name && l.available)
                            });
                            this.choosing_library = this.selected_library.is_none();
                        }
                        // A missing saved library stays selected as unavailable; never substitute another.
                        if !helper
                            && !this
                                .libraries
                                .iter()
                                .any(|l| Some(&l.name) == this.selected_library.as_ref())
                        {
                            this.selected_library = this
                                .libraries
                                .iter()
                                .find(|l| Some(&l.path) == selected_path.as_ref())
                                .or_else(|| this.libraries.iter().find(|l| l.available))
                                .map(|l| l.name.clone());
                            this.preferences.update(cx, |p, cx| {
                                if this.tray {
                                    p.tray_library = this.selected_library.clone();
                                } else {
                                    p.library = this.selected_library.clone();
                                }
                                cx.notify();
                            });
                            this.persist(cx);
                        }
                        this.schedule_search(cx);
                    }
                    Err(error) => this.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn library_saved(&mut self, old_name: Option<&str>, name: String, cx: &mut Context<Self>) {
        if old_name.is_none() || self.selected_library.as_deref() == old_name {
            self.selected_library = Some(name.clone());
            self.preferences.update(cx, |p, cx| {
                p.library = Some(name);
                cx.notify();
            });
            self.persist(cx);
        }
        crate::application::send(bibcitex_linux::Command::RefreshLibraries, cx);
    }

    fn toggle_advanced(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let previous = SearchDraft {
            text: self.search.read(cx).value().to_string(),
            content: Some(self.search.read(cx).content()),
            editor: self.condition_editor.take(),
            field: self.field,
            kind: self.kind,
            document: self.query_document.take(),
            tag_revision: self.tag_revision.take(),
        };
        let restored = std::mem::replace(&mut self.inactive_draft, previous);
        self.condition_editor = restored.editor;
        self.advanced = !self.advanced;
        self.field = restored.field;
        self.kind = restored.kind;
        self.query_document = restored.document;
        self.tag_revision = restored.tag_revision;
        self.search.update(cx, |input, cx| {
            input.set_value(
                restored
                    .content
                    .unwrap_or_else(|| input::InputContent::new(restored.text)),
                window,
                cx,
            )
        });
        if !self.helper {
            self.preferences.update(cx, |preferences, cx| {
                if self.tray {
                    preferences.tray_advanced_search_enabled = self.advanced;
                } else {
                    preferences.main_advanced_search_enabled = self.advanced;
                }
                cx.notify();
            });
            self.persist(cx);
        }
        self.schedule_search(cx);
    }

    fn advanced_toggle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("advanced-search-mode")
            .ghost()
            .small()
            .label(if self.preferences.read(cx).chinese() {
                "高级搜索"
            } else {
                "Advanced search"
            })
            .when(self.advanced, |button| {
                button.bg(cx.theme().muted).icon(icon("check"))
            })
            .on_click(cx.listener(|this, _, window, cx| this.toggle_advanced(window, cx)))
    }

    fn schedule_search(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        self.search_task = None;
        let selected_id = self
            .selected
            .and_then(|i| self.references.get(i))
            .map(|r| r.id.clone());
        self.result_context = None;
        self.query_notice = None;
        self.query_diagnostics.clear();
        self.evidence.clear();
        self.references.clear();
        self.selected = None;
        let query = self.search.read(cx).value().to_string();
        let path = self
            .libraries
            .iter()
            .find(|l| Some(&l.name) == self.selected_library.as_ref() && l.available)
            .map(|l| l.path.clone());
        self.loading = false;
        if self.search_composing
            || self.condition_editor.is_some()
            || self.candidates.is_some()
            || self.optional_field_value.is_some()
            || self.tag_manager.is_some()
            || self.choosing_library
            || (self.helper && query.trim().is_empty())
        {
            cx.notify();
            return;
        }
        let Some(path) = path else {
            cx.notify();
            return;
        };
        self.loading = true;
        let field = FIELDS[self.field].0.to_owned();
        let kind = TYPES[self.kind].0.to_owned();
        let mut document = self.query_document.clone();
        let mut request_text = document
            .clone()
            .map(service::render_query)
            .filter(|analysis| analysis.status == service::QueryStatus::Valid)
            .map(|analysis| analysis.text)
            .unwrap_or(query.clone());
        if self.helper
            && let Some(field) = self.helper_scope.clone()
        {
            let analysis = document
                .clone()
                .map(service::render_query)
                .unwrap_or_else(|| service::analyze_query(query.clone()));
            if analysis.status == service::QueryStatus::Valid
                && let Some(mut base) = analysis.document
                && let Some(root) = base.root
            {
                let index = base.nodes.len() as u32;
                base.nodes.push(service::QueryNodeRecord {
                    kind: service::QueryNodeKind::Scope,
                    text_value: field,
                    tag_id: None,
                    children: vec![root],
                    start: 0,
                    end: 0,
                });
                base.root = Some(index);
                let scoped = service::render_query(base);
                if scoped.status != service::QueryStatus::Valid {
                    self.loading = false;
                    self.query_diagnostic_text = scoped.text;
                    self.query_diagnostics = scoped.diagnostics;
                    self.error = Some(
                        self.query_diagnostics
                            .iter()
                            .map(|d| self.text(&d.code, cx).to_string())
                            .collect::<Vec<_>>()
                            .join("; "),
                    );
                    cx.notify();
                    return;
                }
                document = scoped.document;
                request_text = scoped.text;
            }
        }
        let request = service::AdvancedSearchRequest {
            path,
            mode: if self.advanced {
                service::SearchMode::Advanced
            } else {
                service::SearchMode::Ordinary
            },
            input_kind: if self.advanced && document.is_some() {
                service::QueryInputKind::Bound
            } else {
                service::QueryInputKind::Text
            },
            text: request_text,
            expected_tag_revision: document
                .as_ref()
                .filter(|doc| doc.nodes.iter().any(|node| node.tag_id.is_some()))
                .and(self.tag_revision),
            document,
            default_field: field,
            type_filter: kind,
            query_version: generation,
        };
        let executor = cx.background_executor().clone();
        self.search_task = Some(cx.spawn(async move |this, cx| {
            executor.timer(Duration::from_millis(100)).await;
            let result = executor
                .spawn(async move { service::advanced_search(request) })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.loading = false;
                match result {
                    Ok(response) => {
                        if response.analysis.status == service::QueryStatus::StaleQuery {
                            if let Some(document) = this.query_document.clone() {
                                match service::refresh_query(document) {
                                    Ok(analysis)
                                        if analysis.status == service::QueryStatus::Valid =>
                                    {
                                        this.query_document = analysis.document;
                                        this.tag_revision = analysis.tag_revision;
                                        this.schedule_search(cx);
                                        this.pending_query_projection =
                                            Some((this.generation, analysis.text));
                                        return;
                                    }
                                    Ok(analysis) => {
                                        this.error = Some(
                                            analysis
                                                .diagnostics
                                                .iter()
                                                .map(|diagnostic| {
                                                    this.text(&diagnostic.code, cx).to_string()
                                                })
                                                .collect::<Vec<_>>()
                                                .join("; "),
                                        )
                                    }
                                    Err(error) => this.error = Some(error.to_string()),
                                }
                            } else {
                                this.error = Some(
                                    if this.preferences.read(cx).chinese() {
                                        "数据已变化，请刷新搜索结果"
                                    } else {
                                        "The data changed. Refresh the search results."
                                    }
                                    .into(),
                                );
                            }
                            cx.notify();
                            return;
                        }
                        this.query_diagnostic_text = response.analysis.text.clone();
                        this.query_diagnostics = response.analysis.diagnostics.clone();
                        if response.analysis.status != service::QueryStatus::Valid {
                            this.error = Some(
                                this.query_diagnostics
                                    .iter()
                                    .map(|diagnostic| this.text(&diagnostic.code, cx).to_string())
                                    .collect::<Vec<_>>()
                                    .join("; "),
                            );
                            cx.notify();
                            return;
                        }
                        this.query_notice =
                            (!response.analysis.diagnostics.is_empty()).then(|| {
                                response
                                    .analysis
                                    .diagnostics
                                    .iter()
                                    .map(|diagnostic| this.text(&diagnostic.code, cx).to_string())
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            });
                        this.result_context = response.context;
                        this.evidence = response.evidence;
                        this.references = response.references;
                        this.selected = selected_id
                            .as_ref()
                            .and_then(|id| this.references.iter().position(|r| &r.id == id))
                            .or_else(|| (!this.references.is_empty()).then_some(0));
                        this.error = None;
                        this.scroll
                            .scroll_to_item(this.selected.unwrap_or(0), ScrollStrategy::Top);
                    }
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn select_library(&mut self, name: String, cx: &mut Context<Self>) {
        if self.helper && !self.libraries.iter().any(|l| l.name == name && l.available) {
            return;
        }
        if self.selected_library.as_ref() != Some(&name) {
            self.references.clear();
            self.selected = None;
        }
        self.selected_library = Some(name.clone());
        if self.helper {
            if let Some(library) = self.libraries.iter().find(|l| l.name == name) {
                let path = library.path.clone();
                self.mutate(move || service::helper_select(name, path).map(|_| ()), cx);
            }
        } else {
            self.preferences.update(cx, |p, cx| {
                if self.tray {
                    p.tray_library = Some(name);
                } else {
                    p.library = Some(name);
                }
                cx.notify();
            });
            self.persist(cx);
        }
        self.choosing_library = false;
        self.schedule_search(cx);
    }

    fn persist(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.preferences.read(cx).save() {
            self.error = Some(error);
        }
    }

    fn mutate(
        &mut self,
        action: impl FnOnce() -> Result<(), service::CoreError> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        let job = cx.background_executor().spawn(async move { action() });
        cx.spawn(async move |this, cx| {
            let result = job.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        crate::application::send(bibcitex_linux::Command::RefreshLibraries, cx)
                    }
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn copy(&mut self, value: String, cx: &mut Context<Self>) {
        // GPUI retains clipboard ownership for the application's lifetime on Linux.
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
        self.copied = Some(value);
        self.copy_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1400))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.copied = None;
                cx.notify();
            });
        }));
        cx.notify();
    }
    fn copy_detail(&mut self, value: String, cx: &mut Context<Self>) {
        if !self.current_results_valid(cx) {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
        self.copied_detail = Some(value);
        self.copy_detail_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(1400))
                .await;
            let _ = this.update(cx, |this, cx| {
                this.copied_detail = None;
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn open_file(&mut self, path: &str, cx: &mut Context<Self>) {
        let path = std::path::Path::new(path);
        let absolute = if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            self.libraries
                .iter()
                .find(|l| Some(&l.name) == self.selected_library.as_ref())
                .and_then(|l| std::path::Path::new(&l.path).parent())
                .map(|base| base.join(path))
        };
        match absolute
            .as_deref()
            .and_then(|p| url::Url::from_file_path(p).ok())
        {
            Some(url) => cx.open_url(url.as_str()),
            None => {
                self.error = Some(self.text("Invalid bibliography path", cx).to_string());
                cx.notify();
            }
        }
    }

    fn activate_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_is_composing(window, cx) {
            return;
        }
        if self.pasting || !self.current_results_valid(cx) {
            return;
        }
        let Some(key) = self
            .selected
            .and_then(|i| self.references.get(i))
            .map(|r| r.cite_key.clone())
        else {
            return;
        };
        self.copy(key.clone(), cx);
        #[cfg(target_os = "linux")]
        {
            let Some(desktop) = cx.global::<crate::application::Services>().desktop.clone() else {
                self.paste_failed("Desktop paste is unavailable".into(), key, cx);
                return;
            };
            self.pasting = true;
            let revision = self.generation;
            let search_context = self.result_context.clone();
            let events = cx.global::<crate::application::Services>().events.clone();
            cx.spawn_in(window, async move |this, cx| {
                if let Err(error) = desktop.prepare().await {
                    let _ = this.update_in(cx, |this, _, cx| {
                        this.pasting = false;
                        this.paste_failed(error, key, cx);
                    });
                    return;
                }
                // Closing our surface lets the compositor restore its previous focus.
                if !this
                    .update_in(cx, |this, window, cx| {
                        if this.generation != revision || !this.current_results_valid(cx) {
                            this.pasting = false;
                            return false;
                        }
                        this.copy(key.clone(), cx);
                        window.remove_window();
                        true
                    })
                    .is_ok_and(|ready| ready)
                {
                    return;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(180))
                    .await;
                if let Some(context) = search_context {
                    if !service::validate_search_context(context).unwrap_or(false) {
                        let _ = events
                            .send(bibcitex_linux::Command::Error(
                                "Search results changed; search again before pasting".into(),
                            ))
                            .await;
                        let _ = this.update(cx, |this, cx| {
                            this.pasting = false;
                            cx.notify();
                        });
                        return;
                    }
                } else {
                    return;
                }
                let result = desktop.paste().await;
                let _ = this.update(cx, |this, cx| {
                    this.pasting = false;
                    cx.notify();
                });
                if let Err(error) = result {
                    let _ = events
                        .send(bibcitex_linux::Command::PasteFailed { error, key })
                        .await;
                }
            })
            .detach();
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = window;
            self.paste_failed("Linux desktop integration requires Linux".into(), key, cx);
        }
    }

    fn matching_libraries(&self, cx: &App) -> Vec<&LibraryRecord> {
        let query = self.search.read(cx).value().trim().to_lowercase();
        self.libraries
            .iter()
            .filter(|l| l.name.to_lowercase().contains(&query))
            .collect()
    }

    fn restore_library_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.choosing_library = false;
        if let Some(draft) = self.library_draft.take() {
            self.query_document = draft.document;
            self.tag_revision = draft.tag_revision;
            self.condition_editor = draft.editor;
            self.field = draft.field;
            self.kind = draft.kind;
            self.search.update(cx, |input, cx| {
                input.set_value(
                    draft
                        .content
                        .unwrap_or_else(|| input::InputContent::new(draft.text)),
                    window,
                    cx,
                )
            });
        } else {
            self.search
                .update(cx, |input, cx| input.set_value("", window, cx));
        }
        let placeholder = self.text("搜索文献、作者、标题", cx);
        self.search.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx);
            input.focus(window, cx);
        });
        self.schedule_search(cx);
        self.resize_helper(window, cx);
    }

    fn toggle_library_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.choosing_library {
            self.choosing_library = false;
            self.restore_library_draft(window, cx);
            return;
        }
        self.library_draft = Some(SearchDraft {
            text: self.search.read(cx).value().to_string(),
            content: Some(self.search.read(cx).content()),
            editor: self.condition_editor.take(),
            field: self.field,
            kind: self.kind,
            document: self.query_document.take(),
            tag_revision: self.tag_revision.take(),
        });
        self.choosing_library = true;
        self.library_cursor = 0;
        self.library_scroll.scroll_to_item(0);
        let placeholder = self.text(
            if self.choosing_library {
                "搜索或选择文献库"
            } else {
                "搜索文献、作者、标题"
            },
            cx,
        );
        self.search.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx);
            input.set_value("", window, cx);
            input.focus(window, cx);
        });
        self.schedule_search(cx);
        self.resize_helper(window, cx);
        cx.notify();
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.choosing_library {
            let count = self.matching_libraries(cx).len();
            if count > 0 {
                self.library_cursor = self
                    .library_cursor
                    .saturating_add_signed(delta)
                    .min(count - 1);
                self.library_scroll.scroll_to_item(self.library_cursor);
            }
            cx.notify();
            return;
        }
        if self.references.is_empty() || self.loading {
            return;
        }
        let index = self
            .selected
            .map_or(0, |i| i.saturating_add_signed(delta))
            .min(self.references.len() - 1);
        self.selected = Some(index);
        self.scroll.scroll_to_item(index, ScrollStrategy::Center);
        cx.notify();
    }

    pub fn open_editor(&mut self, library: Option<LibraryRecord>, cx: &mut Context<Self>) {
        if self
            .editor_window
            .is_some_and(|w| w.update(cx, |_, w, _| w.activate_window()).is_ok())
        {
            return;
        }
        let owner = cx.weak_entity();
        let prefs = self.preferences.clone();
        match open_window(
            crate::window_options("BibCiTeX", 520., 520.),
            cx,
            move |window, cx| cx.new(|cx| LibraryEditor::new(library, prefs, owner, window, cx)),
        ) {
            Ok((window, _)) => self.editor_window = Some(window),
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    fn open_helper(&mut self, cx: &mut Context<Self>) {
        crate::application::send(bibcitex_linux::Command::Helper, cx);
    }
    fn open_settings(&mut self, cx: &mut Context<Self>) {
        crate::application::send(bibcitex_linux::Command::Settings, cx);
    }

    fn resize_helper(&self, window: &mut Window, cx: &App) {
        if self.helper {
            let count = if self.choosing_library {
                self.matching_libraries(cx).len()
            } else {
                self.references.len()
            };
            let list_height = if self.candidates.is_some() || self.optional_field_value.is_some() {
                0.
            } else if self.loading {
                112.
            } else if !self.choosing_library && self.search.read(cx).value().trim().is_empty() {
                0.
            } else if count == 0 {
                112.
            } else {
                (count as f32 * if self.choosing_library { 58. } else { 82. } + 16.).min(460.)
            };
            let height = HELPER_HEADER
                + list_height
                + if self.error.is_some() { 40. } else { 0. }
                + if self.condition_editor.is_some() || self.tag_manager.is_some() {
                    350.
                } else {
                    0.
                };
            let available = window.display(cx).map(|display| display.bounds().size);
            let width = available.map_or(px(720.), |bounds| {
                px(720.).min((bounds.width - px(40.)).max(px(320.)))
            });
            let height = available.map_or(px(height), |bounds| {
                px(height).min((bounds.height - px(120.)).max(px(HELPER_HEADER)))
            });
            let desired = size(width, height);
            if window.viewport_size() != desired {
                window.resize(desired);
            }
        }
    }

    fn shows_bottom_bar(&self, cx: &App) -> bool {
        self.helper
            && !self.loading
            && !self.choosing_library
            && !self.references.is_empty()
            && !self.search.read(cx).value().trim().is_empty()
    }

    /// Ghosts rows passing beneath the floating controls.
    fn bottom_dissolve(&self, cx: &Context<Self>) -> impl IntoElement {
        let background = cx.theme().background;
        div()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(HELPER_BOTTOM_BAR))
            .bg(linear_gradient(
                180.,
                linear_color_stop(
                    Hsla {
                        a: 0.,
                        ..background
                    },
                    0.,
                ),
                linear_color_stop(background, 1.),
            ))
    }

    /// Floating controls, no bar: Enter (cross-app paste) and Tab as clickable
    /// hints, grouped in one capsule at the trailing edge.
    fn bottom_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .right(px(12.))
            .bottom(px(8.))
            .flex()
            .items_center()
            .gap(px(2.))
            .p(px(4.))
            .rounded_full()
            .bg(cx.theme().popover)
            .border_1()
            .border_color(cx.theme().border)
            .shadow_sm()
            .child(
                self.key_hint("helper-paste", "粘贴引用键", "↵", cx)
                    .disabled(self.selected.is_none())
                    .on_click(
                        cx.listener(|this, _, window, cx| this.activate_selected(window, cx)),
                    ),
            )
            .child(
                self.key_hint("helper-switch", "切换文献库", "⇥", cx)
                    .on_click(
                        cx.listener(|this, _, window, cx| {
                            this.toggle_library_selection(window, cx)
                        }),
                    ),
            )
    }

    /// A footer control: a label and its outlined key.
    fn key_hint(
        &self,
        id: &'static str,
        label: &'static str,
        key: &'static str,
        cx: &App,
    ) -> Button {
        let muted = cx.theme().muted_foreground;
        Button::new(id)
            .ghost()
            .small()
            .rounded_full()
            .accessibility_label(self.text(label, cx))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(muted)
                            .child(self.text(label, cx)),
                    )
                    .child(
                        div()
                            .min_w(px(18.))
                            .h(px(18.))
                            .px(px(3.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(cx.theme().border)
                            .text_size(px(11.))
                            .text_color(muted)
                            .child(key),
                    ),
            )
    }

    fn filter(&self, types: bool, cx: &Context<Self>) -> impl IntoElement {
        let options = if types { TYPES } else { FIELDS };
        let selected = if types { self.kind } else { self.field };
        let prefs = self.preferences.read(cx).clone();
        let owner = cx.weak_entity();
        Button::new(if types { "type" } else { "field" })
            .small()
            .label(prefs.text(options[selected].1))
            .icon(icon("chevronDown"))
            .dropdown_menu(move |mut menu, _, _| {
                for (index, (_, label)) in options.iter().enumerate() {
                    let owner = owner.clone();
                    menu = menu.item(PopupMenuItem::new(prefs.text(label)).on_click(
                        move |_, _, cx| {
                            let _ = owner.update(cx, |this, cx| {
                                if types {
                                    this.kind = index;
                                } else {
                                    this.field = index;
                                }
                                this.schedule_search(cx);
                            });
                        },
                    ));
                }
                menu
            })
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let prefs = self.preferences.read(cx).clone();
        let owner = cx.weak_entity();
        let mut list = div().id("libraries").overflow_y_scroll().flex_1().p_2();
        for (index, library) in self.libraries.iter().enumerate() {
            let name = library.name.clone();
            let selected = self.selected_library.as_ref() == Some(&name);
            let menu_library = library.clone();
            let menu_owner = owner.clone();
            let menu_prefs = prefs.clone();
            let rename_input = self
                .renaming
                .as_ref()
                .filter(|(old, _)| old == &name)
                .map(|(_, input)| input.clone());
            list = list.child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .rounded(px(ROW_RADIUS))
                    .when(selected, |e| e.bg(cx.theme().muted))
                    .when_some(rename_input.clone(), |row, input| {
                        row.child(Input::new(&input).disabled(self.saving_name).flex_1())
                    })
                    .when(rename_input.is_none(), |row| {
                        row.child(
                            Button::new(("library", index))
                                .ghost()
                                .flex_1()
                                .min_w_0()
                                .h_auto()
                                .py_2()
                                .icon(if library.pinned {
                                    icon("pin")
                                } else {
                                    icon("folderOpen")
                                })
                                .accessibility_label(name.clone())
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .min_w_0()
                                        .items_start()
                                        .gap_1()
                                        .child(
                                            div()
                                                .truncate()
                                                .when(!library.available, |e| {
                                                    e.text_color(cx.theme().muted_foreground)
                                                })
                                                .child(name.clone()),
                                        )
                                        .when(!library.available, |e| {
                                            e.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(prefs.text("不可用")),
                                            )
                                        })
                                        .when_some(
                                            library
                                                .description
                                                .clone()
                                                .filter(|text| !text.is_empty()),
                                            |e, text| {
                                                e.child(
                                                    div()
                                                        .truncate()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(text),
                                                )
                                            },
                                        ),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.select_library(name.clone(), cx)
                                })),
                        )
                    })
                    .child(
                        Button::new(("library-menu", index))
                            .ghost()
                            .small()
                            .icon(icon("more"))
                            .accessibility_label(prefs.text("文献库操作"))
                            .tooltip(prefs.text("文献库操作"))
                            .dropdown_menu(move |menu, _, _| {
                                let edit_owner = menu_owner.clone();
                                let edit_library = menu_library.clone();
                                let rename_owner = menu_owner.clone();
                                let rename_library = menu_library.clone();
                                let file_owner = menu_owner.clone();
                                let file_path = menu_library.path.clone();
                                let pin_owner = menu_owner.clone();
                                let pin_library = menu_library.clone();
                                let remove_owner = menu_owner.clone();
                                let remove_name = menu_library.name.clone();
                                menu.item(PopupMenuItem::new(menu_prefs.text("编辑")).on_click(
                                    move |_, _, cx| {
                                        let _ = edit_owner.update(cx, |this, cx| {
                                            this.open_editor(Some(edit_library.clone()), cx)
                                        });
                                    },
                                ))
                                .item(PopupMenuItem::new(menu_prefs.text("重命名")).on_click(
                                    move |_, window, cx| {
                                        let _ = rename_owner.update(cx, |this, cx| {
                                            this.begin_rename(rename_library.clone(), window, cx)
                                        });
                                    },
                                ))
                                .item(PopupMenuItem::new(menu_prefs.text("打开文件")).on_click(
                                    move |_, _, cx| {
                                        let _ = file_owner
                                            .update(cx, |this, cx| this.open_file(&file_path, cx));
                                    },
                                ))
                                .item(
                                    PopupMenuItem::new(menu_prefs.text(if menu_library.pinned {
                                        "取消置顶"
                                    } else {
                                        "置顶"
                                    }))
                                    .on_click(
                                        move |_, _, cx| {
                                            let library = pin_library.clone();
                                            let _ = pin_owner.update(cx, |this, cx| {
                                                this.mutate(
                                                    move || {
                                                        service::set_library_pinned(
                                                            library.name,
                                                            !library.pinned,
                                                        )
                                                    },
                                                    cx,
                                                )
                                            });
                                        },
                                    ),
                                )
                                .separator()
                                .item(
                                    PopupMenuItem::new(menu_prefs.text("移除")).on_click(
                                        move |_, _, cx| {
                                            let name = remove_name.clone();
                                            let _ = remove_owner.update(cx, |this, cx| {
                                                this.mutate(
                                                    move || service::remove_library(name),
                                                    cx,
                                                )
                                            });
                                        },
                                    ),
                                )
                            }),
                    ),
            );
        }
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().sidebar)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .child(prefs.text("文献库"))
                    .child(
                        Button::new("add")
                            .ghost()
                            .small()
                            .icon(icon("folderAdd"))
                            .accessibility_label(prefs.text("新增文献库"))
                            .tooltip(prefs.text("新增文献库"))
                            .on_click(cx.listener(|this, _, _, cx| this.open_editor(None, cx))),
                    ),
            )
            .when(self.libraries.is_empty(), |e| {
                e.child(
                    div()
                        .p_4()
                        .text_color(cx.theme().muted_foreground)
                        .child(prefs.text("暂无文献库")),
                )
            })
            .child(list)
    }

    fn row(&self, index: usize, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let generation = self.generation;
        let record = &self.references[index];
        let key = record.cite_key.clone();
        let evidence = self.evidence.iter().find(|e| e.reference_id == record.id);
        let title_ranges: Vec<_> = evidence
            .into_iter()
            .flat_map(|e| &e.ranges)
            .filter(|range| range.field_name == "title" && range.entry_index == 0)
            .map(|range| (range.start as usize, range.end as usize))
            .collect();
        let title = crate::math::rich_highlighted(&record.title, &title_ranges, cx);
        let mut author_offset = 0;
        let mut author_highlights = Vec::new();
        for (index, name) in record.author.iter().enumerate() {
            for range in evidence
                .into_iter()
                .flat_map(|e| &e.ranges)
                .filter(|range| range.field_name == "author" && range.entry_index as usize == index)
            {
                let start = range.start as usize;
                let end = range.end as usize;
                if start < end && name.get(start..end).is_some() {
                    author_highlights.push((
                        author_offset + start..author_offset + end,
                        HighlightStyle {
                            color: Some(cx.theme().link),
                            font_weight: Some(FontWeight::SEMIBOLD),
                            ..Default::default()
                        },
                    ));
                }
            }
            author_offset += name.len() + 2;
        }
        let author = StyledText::new(record.author.join(", "))
            .with_highlights(combine_highlights(vec![], author_highlights).collect::<Vec<_>>());
        let kind = TYPES
            .iter()
            .find(|(kind, _)| *kind == record.entry_type)
            .map_or(record.entry_type.as_str(), |(_, label)| *label);
        let venue = record
            .full_journal
            .clone()
            .filter(|v| !v.is_empty())
            .or_else(|| record.journal.clone().filter(|v| !v.is_empty()))
            .or_else(|| (!record.book_title.is_empty()).then(|| chunks(&record.book_title)))
            .or_else(|| (!record.publisher.is_empty()).then(|| record.publisher.join(", ")))
            .or_else(|| record.school.clone().filter(|v| !v.is_empty()))
            .or_else(|| record.institution.clone().filter(|v| !v.is_empty()))
            .or_else(|| (!record.organization.is_empty()).then(|| record.organization.join(", ")))
            .or_else(|| record.how_published.clone())
            .unwrap_or_default();
        let metadata = [
            self.text(kind, cx).to_string(),
            record.year.map(|y| y.to_string()).unwrap_or_default(),
            venue,
            evidence
                .map(|e| {
                    format!(
                        "{}{}",
                        e.fields.join(", "),
                        if e.fuzzy {
                            format!(" · ~ {}", e.fuzzy_terms.join(", "))
                        } else {
                            String::new()
                        }
                    )
                })
                .unwrap_or_default(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
        let menu_key = key.clone();
        let menu_source = record.source.clone();
        let owner = cx.weak_entity();
        let copy_label = self.text("复制引用键", cx);
        let source_label = self.text("复制 BibTeX", cx);
        let evidence_summary = evidence.map(|e| {
            format!(
                "{}{}{}",
                e.fields.join(", "),
                if e.fuzzy {
                    if self.preferences.read(cx).chinese() {
                        " · 拼写补充"
                    } else {
                        " · Spelling supplement"
                    }
                } else {
                    ""
                },
                if e.fuzzy && !e.fuzzy_terms.is_empty() {
                    format!(": {}", e.fuzzy_terms.join(", "))
                } else {
                    String::new()
                }
            )
        });
        let mut tag_line = div().flex().gap_1().items_center().overflow_hidden();
        for (tag_index, tag) in record.tags.iter().take(2).enumerate() {
            let color =
                u32::from_str_radix(tag.color.trim_start_matches('#'), 16).unwrap_or(0x64748B);
            tag_line = tag_line.child(
                Button::new(("reference-tag", tag_index))
                    .ghost()
                    .small()
                    .h(px(18.))
                    .rounded_full()
                    .text_size(px(10.))
                    .bg(cx.theme().muted)
                    .max_w(px(65.))
                    .overflow_hidden()
                    .label(tag.name.clone())
                    .icon(icon("tag").size(px(10.)).text_color(gpui::rgb(color)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        if this.generation == generation && this.current_results_valid(cx) {
                            this.selected = Some(index);
                            this.open_tags(window, cx);
                        }
                    })),
            );
        }
        if record.tags.len() > 2 {
            tag_line = tag_line.child(
                Button::new(("reference-tags-overflow", index))
                    .ghost()
                    .small()
                    .label(format!("+{}", record.tags.len() - 2))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        if this.generation == generation && this.current_results_valid(cx) {
                            this.selected = Some(index);
                            this.open_tags(window, cx);
                        }
                    })),
            );
        }
        let row = div()
            .id(("reference", index))
            .when_some(evidence_summary, |row, summary| {
                row.tooltip(move |window, cx| Tooltip::new(summary.clone()).build(window, cx))
            })
            .w_full()
            .h(px(if self.helper {
                82.
            } else if self.tray {
                88.
            } else {
                92.
            }))
            .px_3()
            .py_2()
            .flex()
            .gap_2()
            .rounded(px(ROW_RADIUS))
            .when(self.selected == Some(index), |e| e.bg(cx.theme().muted))
            .hover(|e| e.bg(cx.theme().muted))
            .when(self.helper, |e| e.text_size(px(13.)))
            .on_click(cx.listener(move |this, _, window, cx| {
                if this.generation != generation || !this.current_results_valid(cx) {
                    return;
                }
                this.selected = Some(index);
                if this.helper {
                    this.activate_selected(window, cx);
                }
                cx.notify();
            }))
            .when(self.helper, |row| {
                row.child(
                    icon("fileText")
                        .size(px(15.))
                        .mt(px(2.))
                        .text_color(cx.theme().muted_foreground),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().line_clamp(2).child(title))
                    .child(
                        div()
                            .text_sm()
                            .when(self.helper, |text| text.text_size(px(11.)))
                            .text_color(cx.theme().muted_foreground)
                            .truncate()
                            .child(author),
                    )
                    .child(
                        div()
                            .text_xs()
                            .when(self.helper, |text| text.text_size(px(11.)))
                            .text_color(cx.theme().muted_foreground)
                            .truncate()
                            .child(metadata),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap_1()
                    .max_w(px(160.))
                    .when(!self.helper, |row| {
                        row.child(
                            Button::new(("copy-key", index))
                                .ghost()
                                .small()
                                .rounded_full()
                                .text_size(px(11.))
                                .text_color(cx.theme().link)
                                .bg(cx.theme().muted)
                                .icon(
                                    icon(if self.copied.as_ref() == Some(&key) {
                                        "check"
                                    } else {
                                        "copy"
                                    })
                                    .size(px(10.)),
                                )
                                .label(if self.copied.as_ref() == Some(&key) {
                                    self.text("已复制", cx)
                                } else {
                                    key.clone().into()
                                })
                                .tooltip(self.text("复制引用键", cx))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    if this.generation == generation
                                        && this.current_results_valid(cx)
                                    {
                                        this.copy(key.clone(), cx);
                                    }
                                })),
                        )
                    })
                    .when(self.helper, |row| {
                        row.child(
                            div()
                                .text_size(px(11.))
                                .text_color(cx.theme().link)
                                .flex_shrink_0()
                                .child(record.cite_key.clone()),
                        )
                    })
                    .when(!record.tags.is_empty(), |column| column.child(tag_line)),
            );
        let tag_label = self.text("标签", cx);
        row.context_menu(move |menu, _, _| {
            let key_owner = owner.clone();
            let source_owner = owner.clone();
            let key = menu_key.clone();
            let source = menu_source.clone();
            let tag_owner = owner.clone();
            menu.item(
                PopupMenuItem::new(tag_label.clone()).on_click(move |_, window, cx| {
                    let _ = tag_owner.update(cx, |this, cx| {
                        if this.generation == generation && this.current_results_valid(cx) {
                            this.selected = Some(index);
                            this.open_tags(window, cx);
                        }
                    });
                }),
            )
            .item(
                PopupMenuItem::new(copy_label.clone()).on_click(move |_, _, cx| {
                    let _ = key_owner.update(cx, |this, cx| {
                        if this.generation == generation && this.current_results_valid(cx) {
                            this.copy(key.clone(), cx);
                        }
                    });
                }),
            )
            .item(
                PopupMenuItem::new(source_label.clone()).on_click(move |_, _, cx| {
                    let _ = source_owner.update(cx, |this, cx| {
                        if this.generation == generation && this.current_results_valid(cx) {
                            this.copy(source.clone(), cx);
                        }
                    });
                }),
            )
        })
        .into_any_element()
    }

    fn results(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.loading {
            return div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(Spinner::new().small())
                .into_any_element();
        }
        if self.references.is_empty() {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .when(self.libraries.is_empty(), |e| {
                    e.child(crate::brand::logo(96.))
                })
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(self.text("暂无可显示的文献", cx)),
                )
                .into_any_element();
        }
        uniform_list(
            "references",
            self.references.len(),
            cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                range.map(|i| this.row(i, cx)).collect()
            }),
        )
        .track_scroll(&self.scroll)
        .flex_1()
        .min_h_0()
        .px_2()
        .when(self.helper, |list| list.py_2())
        // An inset rather than a row: the last result can still scroll clear of the controls.
        .when(self.shows_bottom_bar(cx), |list| {
            list.pb(px(HELPER_BOTTOM_BAR))
        })
        .into_any_element()
    }

    fn content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self
            .selected_library
            .clone()
            .unwrap_or_else(|| self.text("文献工作台", cx).to_string());
        let count = self
            .text("{0} 条文献", cx)
            .replace("{0}", &self.references.len().to_string());
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_w_0()
            .when(!self.libraries.is_empty(), |e| {
                e.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(17.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(count),
                                ),
                        )
                        .child(
                            Input::new(&self.search)
                                .disabled(
                                    self.condition_editor.is_some() || self.tag_manager.is_some(),
                                )
                                .prefix(icon("search").size(px(15.)))
                                .appearance(false)
                                .focus_bordered(false)
                                .h(px(32.))
                                .bg(cx.theme().muted)
                                .rounded_md()
                                .text_size(px(13.))
                                .cleanable(true),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(self.advanced_toggle(cx))
                                .child(self.condition_controls(cx))
                                .child(self.filter(true, cx))
                                .child(self.filter(false, cx)),
                        ),
                )
            })
            .child(self.condition_panel(cx))
            .child(self.results(cx))
    }

    fn tray_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let prefs = self.preferences.read(cx).clone();
        let owner = cx.weak_entity();
        let libraries = self.libraries.clone();
        let unavailable = prefs.text("不可用").to_string();
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("BibCiTeX"))
                            .child(div().flex_1())
                            // One capsule for the toolbar, as on the macOS tray.
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .p_1()
                                    .rounded_full()
                                    .bg(cx.theme().muted)
                                    .child(
                                        Button::new("tray-detail")
                                            .ghost()
                                            .small()
                                            .icon(icon(if self.tray_detail {
                                                "panelRightClose"
                                            } else {
                                                "panelRightOpen"
                                            }))
                                            .accessibility_label(prefs.text("文献详情"))
                                            .tooltip(prefs.text("文献详情"))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.tray_detail = !this.tray_detail;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("tray-refresh")
                                            .ghost()
                                            .small()
                                            .icon(icon("refresh"))
                                            .accessibility_label(prefs.text("刷新"))
                                            .tooltip(prefs.text("刷新"))
                                            .on_click(
                                                cx.listener(|this, _, _, cx| this.reload(cx)),
                                            ),
                                    )
                                    .child(
                                        Button::new("tray-main")
                                            .ghost()
                                            .small()
                                            .icon(icon("externalLink"))
                                            .accessibility_label(prefs.text("显示窗口"))
                                            .tooltip(prefs.text("显示窗口"))
                                            .on_click(|_, _, cx| {
                                                crate::application::send(
                                                    bibcitex_linux::Command::Main,
                                                    cx,
                                                )
                                            }),
                                    )
                                    .child(
                                        Button::new("tray-close")
                                            .ghost()
                                            .small()
                                            .icon(icon("x"))
                                            .accessibility_label(prefs.text("关闭"))
                                            .tooltip(prefs.text("关闭"))
                                            .on_click(|_, window, _| window.remove_window()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_1()
                            .rounded_lg()
                            .bg(cx.theme().muted)
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                icon("search")
                                    .size(px(15.))
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                Input::new(&self.search)
                                    .disabled(
                                        self.condition_editor.is_some()
                                            || self.tag_manager.is_some(),
                                    )
                                    .appearance(false)
                                    .focus_bordered(false)
                                    .flex_1(),
                            )
                            .child(div().w(px(1.)).h(px(16.)).bg(cx.theme().border))
                            .child(
                                Button::new("tray-library")
                                    .ghost()
                                    .small()
                                    .w(px(150.))
                                    .icon(icon("library").size(px(13.)))
                                    .accessibility_label(prefs.text("文献库"))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_size(px(12.))
                                            .child(self.selected_library.clone().unwrap_or_else(
                                                || prefs.text("暂无文献库").to_string(),
                                            )),
                                    )
                                    .child(icon("chevronDown").size(px(10.)))
                                    .dropdown_menu(move |mut menu, _, _| {
                                        for library in &libraries {
                                            let owner = owner.clone();
                                            let name = library.name.clone();
                                            let label = if library.available {
                                                name.clone()
                                            } else {
                                                format!("{name} · {unavailable}")
                                            };
                                            menu = menu.item(PopupMenuItem::new(label).on_click(
                                                move |_, _, cx| {
                                                    let _ = owner.update(cx, |this, cx| {
                                                        this.select_library(name.clone(), cx)
                                                    });
                                                },
                                            ));
                                        }
                                        menu
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.advanced_toggle(cx))
                            .child(self.condition_controls(cx))
                            .child(self.filter(true, cx))
                            .child(self.filter(false, cx))
                            .child(div().flex_1())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if self.loading {
                                        prefs.text("正在搜索").to_string()
                                    } else {
                                        prefs
                                            .text("{0} 篇文献")
                                            .replace("{0}", &self.references.len().to_string())
                                    }),
                            ),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(self.results(cx))
                    .when(self.tray_detail, |e| {
                        e.child(
                            div()
                                .absolute()
                                .top_3()
                                .right_3()
                                .bottom_3()
                                .w(px(282.))
                                .rounded(px(TRAY_DETAIL_RADIUS))
                                .border_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().background)
                                .shadow_lg()
                                .overflow_hidden()
                                .child(self.tray_inspector(cx)),
                        )
                    }),
            )
    }

    /// A detail field: its caption, an optional trailing action on the same line, and its value.
    fn detail_field(
        &self,
        label: SharedString,
        accessory: Option<Button>,
        content: AnyElement,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(label),
                    )
                    .children(accessory),
            )
            .child(content)
    }

    fn detail_copy(
        &self,
        id: &'static str,
        label: &'static str,
        symbol: &'static str,
        value: String,
        cx: &mut Context<Self>,
    ) -> Button {
        let generation = self.generation;
        let copied = self.copied_detail.as_ref() == Some(&value);
        Button::new(id)
            .ghost()
            .xsmall()
            .icon(icon(if copied { "check" } else { symbol }).size(px(12.)))
            .when(copied, |button| button.label(self.text("已复制", cx)))
            .accessibility_label(self.text(label, cx))
            .tooltip(self.text(label, cx))
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.generation == generation {
                    this.copy_detail(value.clone(), cx);
                }
            }))
    }

    /// A detail value that opens its target when clicked.
    fn detail_link(
        &self,
        id: impl Into<ElementId>,
        value: &str,
        target: LinkTarget,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (label, action): (&'static str, LinkAction) = match target {
            LinkTarget::Url(label, Some(url)) => (
                label,
                Box::new(move |_: &mut Self, cx: &mut Context<Self>| cx.open_url(url.as_str())),
            ),
            LinkTarget::Url(_, None) => {
                return selectable_text(id, value).into_any_element();
            }
            LinkTarget::Action(label, action) => (label, action),
        };
        let generation = self.generation;
        let tooltip = self.text(label, cx);
        div()
            .id(id)
            .cursor_pointer()
            .text_color(cx.theme().link)
            .hover(|style| style.underline())
            .child(value.to_string())
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.generation == generation && this.current_results_valid(cx) {
                    action(this, cx);
                }
            }))
            .into_any_element()
    }

    fn tray_inspector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.selected.and_then(|i| self.references.get(i));
        let heading = match selected {
            Some(record) => crate::math::rich(&record.title, cx),
            None => self.text("文献详情", cx).into_any_element(),
        };
        let mut panel = div().size_full().flex().flex_col().child(
            div()
                .px(px(14.))
                .py_2()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(cx.theme().border)
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(heading),
                )
                .child(
                    Button::new("tray-close-detail")
                        .ghost()
                        .small()
                        .icon(icon("x"))
                        .accessibility_label(self.text("关闭", cx))
                        .tooltip(self.text("关闭", cx))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.tray_detail = false;
                            this.focus_search(window, cx);
                            cx.notify();
                        })),
                ),
        );
        if let Some(record) = selected {
            let mut body = div()
                .id("tray-detail-body")
                .overflow_y_scroll()
                .flex_1()
                .min_h_0()
                .p(px(14.))
                .flex()
                .flex_col()
                .gap_3();
            for (i, (label, value)) in crate::metadata::fields(record).into_iter().enumerate() {
                if value.is_empty()
                    || !matches!(
                        label,
                        "引用键" | "作者" | "类型" | "年份" | "期刊" | "DOI" | "URL" | "摘要"
                    )
                {
                    continue;
                }
                let accessory = (label == "引用键").then(|| {
                    self.detail_copy("tray-copy-key", "复制引用键", "copy", value.clone(), cx)
                });
                let content = match label {
                    "摘要" => crate::math::rich(&record.abstract_text, cx),
                    "DOI" | "URL" => self.detail_link(
                        ("tray-link", i),
                        &value,
                        external_target(label, &value),
                        cx,
                    ),
                    _ => selectable_text(("tray-meta", i), &value).into_any_element(),
                };
                body = body.child(self.detail_field(self.text(label, cx), accessory, content, cx));
            }
            let copy = self.detail_copy(
                "tray-copy-source",
                "复制 BibTeX",
                "clipboard",
                record.source.clone(),
                cx,
            );
            let source = div()
                .text_xs()
                .child(selectable_text("tray-source", &record.source))
                .into_any_element();
            body = body.child(self.detail_field("BibTeX".into(), Some(copy), source, cx));
            panel = panel.child(body);
        } else {
            panel = panel.child(
                div()
                    .flex_1()
                    .p_4()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.text("选择一条文献查看字段和操作", cx)),
            );
        }
        panel
    }

    fn inspector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let heading = match self.selected.and_then(|i| self.references.get(i)) {
            Some(record) => crate::math::rich(&record.title, cx),
            None => self.text("文献详情", cx).into_any_element(),
        };
        let mut body = div()
            .id("inspector-body")
            .flex_1()
            .overflow_y_scroll()
            .p_4()
            .flex()
            .flex_col()
            .gap_4();
        if let Some(record) = self.selected.and_then(|i| self.references.get(i)) {
            let generation = self.generation;
            let bibliography = self
                .libraries
                .iter()
                .find(|library| Some(&library.name) == self.selected_library.as_ref())
                .map(|library| library.path.clone());
            for (field_index, (label, value)) in
                crate::metadata::fields(record).into_iter().enumerate()
            {
                if value.is_empty() {
                    continue;
                }
                let accessory = (label == "引用键").then(|| {
                    self.detail_copy("detail-key", "复制引用键", "copy", value.clone(), cx)
                });
                let content = match label {
                    "摘要" => crate::math::rich(&record.abstract_text, cx),
                    "书名" => crate::math::rich(&record.book_title, cx),
                    "期号" => crate::math::rich(&record.issue, cx),
                    "备注" => crate::math::rich(&record.note, cx),
                    "DOI" | "URL" => self.detail_link(
                        ("metadata", field_index),
                        &value,
                        external_target(label, &value),
                        cx,
                    ),
                    "文件" => {
                        let file = value.clone();
                        let bibliography = bibliography.clone();
                        self.detail_link(
                            ("metadata", field_index),
                            &value,
                            LinkTarget::Action(
                                "打开文件",
                                Box::new(
                                    move |this: &mut Workbench, cx: &mut Context<Workbench>| {
                                        if this.generation != generation {
                                            return;
                                        }
                                        if let Some(bibliography) = &bibliography {
                                            match bibcitex_linux::files::attachment_path(
                                                &file,
                                                std::path::Path::new(bibliography),
                                                dirs::home_dir().as_deref(),
                                            ) {
                                                Ok(path) => cx.open_with_system(&path),
                                                Err(error) => this.report_error(error, cx),
                                            }
                                        }
                                    },
                                ),
                            ),
                            cx,
                        )
                    }
                    _ => selectable_text(("metadata", field_index), &value).into_any_element(),
                };
                body = body.child(self.detail_field(self.text(label, cx), accessory, content, cx));
            }
            let copy = self.detail_copy(
                "detail-bib",
                "复制 BibTeX",
                "clipboard",
                record.source.clone(),
                cx,
            );
            let source = div()
                .text_xs()
                .child(selectable_text("bibtex", &record.source))
                .into_any_element();
            body = body.child(self.detail_field("BibTeX".into(), Some(copy), source, cx));
        } else {
            body = body.child(self.text("选择一条文献查看字段和操作", cx));
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().sidebar)
            .child(
                div()
                    .p_4()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .text_size(px(17.))
                    .child(heading),
            )
            .child(body)
    }
}

impl Render for Workbench {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.synchronize_ime(window, cx);
        if !self.input_is_composing(window, cx)
            && let Some((generation, text)) = self.pending_query_projection.take()
            && generation == self.generation
        {
            self.install_bound_text(text, window, cx);
        }
        if self.helper {
            self.resize_helper(window, cx);
            if self.placeholder_selecting != self.choosing_library {
                self.placeholder_selecting = self.choosing_library;
                let placeholder = self.text(
                    if self.choosing_library {
                        "搜索或选择文献库"
                    } else {
                        "搜索文献、作者、标题"
                    },
                    cx,
                );
                self.search.update(cx, |input, cx| {
                    input.set_placeholder(placeholder, window, cx)
                });
            }
        }
        let prefs = self.preferences.read(cx).clone();
        let update_available = cx
            .global::<crate::application::Services>()
            .updater
            .read(cx)
            .release
            .is_some();
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .text_sm()
            .when(self.helper || self.tray, |root| {
                root.rounded(px(if self.helper {
                    HELPER_RADIUS
                } else {
                    TRAY_RADIUS
                }))
                .overflow_hidden()
                .border_1()
                .border_color(cx.theme().border)
            })
            .capture_action(cx.listener(|this, _: &input::Backspace, window, cx| {
                if this.backspace_scope_capsule(window, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &input::Paste, _, _| {
                this.pending_trigger = None;
            }))
            .capture_action(cx.listener(|this, _: &input::Undo, _, _| {
                this.pending_trigger = None;
            }))
            .capture_action(cx.listener(|this, _: &input::Redo, _, _| {
                this.pending_trigger = None;
            }))
            .capture_action(cx.listener(|this, _: &input::IndentInline, window, cx| {
                if this.input_is_composing(window, cx) {
                    return;
                }
                if this.optional_field_value.is_some() {
                    this.finish_optional_field(true, window, cx);
                    cx.stop_propagation();
                } else if this.candidates.is_some() {
                    this.focus_candidates(window, cx);
                    cx.stop_propagation();
                } else if this.helper && this.search_navigation(window, cx) {
                    this.toggle_library_selection(window, cx);
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &input::MoveUp, window, cx| {
                if this.input_is_composing(window, cx) {
                    return;
                }
                if this.candidates.is_some() {
                    this.candidate_navigation(-1, cx);
                    cx.stop_propagation();
                } else if this.search_navigation(window, cx) {
                    this.move_selection(-1, cx);
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &input::MoveDown, window, cx| {
                if this.input_is_composing(window, cx) {
                    return;
                }
                if this.candidates.is_some() {
                    this.candidate_navigation(1, cx);
                    cx.stop_propagation();
                } else if this.search_navigation(window, cx) {
                    this.move_selection(1, cx);
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &input::Escape, window, cx| {
                if this.input_is_composing(window, cx) {
                    return;
                }
                if this.candidates.is_some() {
                    this.cancel_candidate(window, cx);
                    cx.stop_propagation();
                } else if this.optional_field_value.is_some() {
                    this.finish_optional_field(true, window, cx);
                    cx.stop_propagation();
                } else if this.tag_manager.is_some() {
                    if let Some(manager) = this.tag_manager.as_mut()
                        && manager.cancel_confirmation()
                    {
                        cx.notify();
                        cx.stop_propagation();
                        return;
                    }
                    this.tag_manager = None;
                    this.reload(cx);
                    this.focus_search(window, cx);
                    cx.stop_propagation();
                } else if this.choosing_library {
                    this.restore_library_draft(window, cx);
                    cx.stop_propagation();
                } else if this.condition_editor.is_some() {
                    this.condition_editor = None;
                    this.schedule_search(cx);
                    this.focus_search(window, cx);
                    cx.stop_propagation();
                } else if this.renaming.is_some() && !this.saving_name {
                    this.renaming = None;
                    this.rename_subscription = None;
                    this.focus_search(window, cx);
                    cx.notify();
                    cx.stop_propagation();
                } else if (this.helper || this.tray) && this.search_navigation(window, cx) {
                    window.remove_window();
                    cx.stop_propagation();
                }
            }))
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.pending_trigger = None;
                let focused = this.search.read(cx).focus_handle(cx).is_focused(window);
                let composing = this.input_is_composing(window, cx);
                if composing {
                    return;
                }
                if focused
                    && !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.platform
                {
                    this.prepare_candidate_trigger(
                        event
                            .keystroke
                            .key_char
                            .as_deref()
                            .unwrap_or(&event.keystroke.key),
                        window,
                        cx,
                    );
                }
                match event.keystroke.key.as_str() {
                    "up" if this.candidates.is_some() => {
                        this.candidate_navigation(-1, cx);
                        cx.stop_propagation();
                    }
                    "down" if this.candidates.is_some() => {
                        this.candidate_navigation(1, cx);
                        cx.stop_propagation();
                    }
                    "tab" if this.optional_field_value.is_some() => {
                        this.finish_optional_field(true, window, cx);
                        cx.stop_propagation();
                    }
                    "tab" if this.candidates.is_some() => {
                        this.focus_candidates(window, cx);
                        cx.stop_propagation();
                    }
                    "escape" if this.candidates.is_some() => {
                        this.cancel_candidate(window, cx);
                        cx.stop_propagation();
                    }
                    "escape" if this.optional_field_value.is_some() => {
                        this.finish_optional_field(true, window, cx);
                        cx.stop_propagation();
                    }
                    "escape" if this.tag_manager.is_some() => {
                        if let Some(manager) = this.tag_manager.as_mut()
                            && manager.cancel_confirmation()
                        {
                            cx.notify();
                            cx.stop_propagation();
                            return;
                        }
                        this.tag_manager = None;
                        this.reload(cx);
                        this.focus_search(window, cx);
                        cx.stop_propagation();
                    }
                    "up" if focused => {
                        this.move_selection(-1, cx);
                        cx.stop_propagation();
                    }
                    "down" if focused => {
                        this.move_selection(1, cx);
                        cx.stop_propagation();
                    }
                    "tab" if this.helper && focused => {
                        this.toggle_library_selection(window, cx);
                        cx.stop_propagation();
                    }
                    "q" if event.keystroke.modifiers.control => {
                        crate::application::send(bibcitex_linux::Command::Quit, cx);
                        cx.stop_propagation();
                    }
                    "escape" if this.choosing_library => {
                        this.restore_library_draft(window, cx);
                        cx.stop_propagation();
                    }
                    "escape" if this.condition_editor.is_some() => {
                        this.condition_editor = None;
                        this.schedule_search(cx);
                        this.focus_search(window, cx);
                        cx.stop_propagation();
                    }
                    "escape" if this.helper || this.tray => {
                        window.remove_window();
                        cx.stop_propagation();
                    }
                    "f" if event.keystroke.modifiers.control => {
                        this.search.update(cx, |input, cx| input.focus(window, cx));
                        cx.stop_propagation();
                    }
                    "k" if event.keystroke.modifiers.control
                        && event.keystroke.modifiers.shift
                        && !this.helper =>
                    {
                        this.open_helper(cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }));
        if self.helper {
            root =
                root.child(
                    div()
                        .h(px(HELPER_HEADER))
                        .flex_none()
                        .px(px(18.))
                        .flex()
                        .items_center()
                        .gap(px(14.))
                        .child(
                            icon("search")
                                .size(px(18.))
                                .text_color(cx.theme().muted_foreground),
                        )
                        .child(self.helper_scope_control(cx))
                        .child(
                            Input::new(&self.search)
                                .disabled(
                                    self.candidates.is_some()
                                        || self.optional_field_value.is_some()
                                        || self.condition_editor.is_some()
                                        || self.tag_manager.is_some(),
                                )
                                .appearance(false)
                                .focus_bordered(false)
                                .text_size(px(20.))
                                .cleanable(false)
                                .on_token_click({
                                    let owner = cx.weak_entity();
                                    move |event, window, cx| {
                                        let range = event.range();
                                        let _ = owner.update(cx, |this, cx| {
                                            this.edit_inline_token(range, window, cx)
                                        });
                                    }
                                })
                                .min_w_0()
                                .flex_1(),
                        )
                        .child(self.helper_filter_control(cx))
                        .child(self.optional_field_control(cx))
                        .when(
                            !self.choosing_library
                                && self.candidates.is_none()
                                && self.optional_field_value.is_none()
                                && self.selected_library.is_some(),
                            |header| {
                                header.child(
                                    Button::new("choose-library")
                                        .ghost()
                                        .small()
                                        .rounded_full()
                                        .max_w(px(160.))
                                        .bg(cx.theme().muted)
                                        .icon(icon("library").size(px(14.)))
                                        .accessibility_label(prefs.text("切换文献库"))
                                        .accessibility_label(prefs.text("切换文献库 (Tab)"))
                                        .tooltip(prefs.text("切换文献库 (Tab)"))
                                        .child(div().min_w_0().truncate().text_size(px(11.)).child(
                                            self.selected_library.clone().unwrap_or_else(|| {
                                                self.text("文献库", cx).to_string()
                                            }),
                                        ))
                                        .child(icon("chevronDown").size(px(9.)))
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.toggle_library_selection(window, cx);
                                        })),
                                )
                            },
                        ),
                );
            if self.choosing_library {
                let mut libraries = div()
                    .id("helper-libraries")
                    .overflow_y_scroll()
                    .track_scroll(&self.library_scroll)
                    .flex_1()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .p_2();
                if self.matching_libraries(cx).is_empty() {
                    libraries = libraries.child(
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.text("未找到文献库，请先到主窗口添加文献库", cx)),
                    );
                }
                for (i, library) in self.matching_libraries(cx).into_iter().enumerate() {
                    let name = library.name.clone();
                    libraries = libraries.child(
                        Button::new(("helper-library", i))
                            .ghost()
                            .disabled(!library.available)
                            .h(px(56.))
                            .mb(px(2.))
                            .rounded(px(ROW_RADIUS))
                            .when(i == self.library_cursor, |button| {
                                button.bg(cx.theme().muted)
                            })
                            .w_full()
                            .justify_start()
                            .icon(icon("library").size(px(15.)))
                            .accessibility_label(name.clone())
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .items_start()
                                    .gap(px(3.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .truncate()
                                            .when(!library.available, |e| {
                                                e.text_color(cx.theme().muted_foreground)
                                            })
                                            .child(name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(cx.theme().muted_foreground)
                                            .truncate()
                                            .child(if library.available {
                                                library.path.clone()
                                            } else {
                                                format!(
                                                    "{} · {}",
                                                    self.text("不可用", cx),
                                                    library.path
                                                )
                                            }),
                                    ),
                            )
                            .when(self.selected_library.as_ref() == Some(&name), |button| {
                                button.child(icon("check").size(px(13.)))
                            })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.select_library(name.clone(), cx);
                                let placeholder = this.text("搜索文献、作者、标题", cx);
                                this.search.update(cx, |input, cx| {
                                    input.set_placeholder(placeholder, window, cx);
                                    input.focus(window, cx);
                                });
                                this.restore_library_draft(window, cx);
                                this.resize_helper(window, cx);
                            })),
                    );
                }
                root = root.child(libraries);
            } else if !self.search.read(cx).value().trim().is_empty() {
                root = root.child(
                    div()
                        .relative()
                        .border_t_1()
                        .border_color(cx.theme().border)
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(self.results(cx))
                        .when(self.shows_bottom_bar(cx), |area| {
                            area.child(self.bottom_dissolve(cx))
                                .child(self.bottom_bar(cx))
                        }),
                );
            }
        } else if self.tray {
            root = root.child(self.tray_content(cx));
        } else {
            root = root
                .child(
                    TitleBar::new()
                        .h(px(52.))
                        .flex_none()
                        .pr_3()
                        .flex()
                        .items_center()
                        .gap_2()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(
                            Button::new("sidebar")
                                .ghost()
                                .icon(icon(if prefs.sidebar {
                                    "panelLeftClose"
                                } else {
                                    "panelLeftOpen"
                                }))
                                .accessibility_label(prefs.text("文献库"))
                                .tooltip(prefs.text("文献库"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.preferences.update(cx, |p, cx| {
                                        p.sidebar = !p.sidebar;
                                        cx.notify();
                                    });
                                    this.persist(cx);
                                })),
                        )
                        .child(crate::brand::logo(48.))
                        .child(div().flex_1())
                        .child(
                            Button::new("helper")
                                .ghost()
                                .icon(icon("search"))
                                .accessibility_label(prefs.text("快捷助手"))
                                .tooltip(prefs.text("快捷助手"))
                                .on_click(cx.listener(|this, _, _, cx| this.open_helper(cx))),
                        )
                        .child(
                            Button::new("refresh")
                                .ghost()
                                .icon(icon("refresh"))
                                .accessibility_label(prefs.text("刷新"))
                                .tooltip(prefs.text("刷新"))
                                .on_click(cx.listener(|this, _, _, cx| this.reload(cx))),
                        )
                        .child(
                            Button::new("settings")
                                .when(update_available, |button| button.label(prefs.text("更新")))
                                .ghost()
                                .icon(icon("settings"))
                                .accessibility_label(prefs.text("设置"))
                                .tooltip(prefs.text("设置"))
                                .on_click(cx.listener(|this, _, _, cx| this.open_settings(cx))),
                        )
                        .child(
                            Button::new("about")
                                .ghost()
                                .icon(icon("info"))
                                .accessibility_label(prefs.text("关于 BibCiTeX"))
                                .tooltip(prefs.text("关于 BibCiTeX"))
                                .on_click(|_, _, cx| {
                                    crate::application::send(bibcitex_linux::Command::About, cx);
                                }),
                        )
                        .child(div().mx_1().w_px().h(px(20.)).bg(cx.theme().border))
                        .child(
                            Button::new("inspector")
                                .ghost()
                                .icon(icon(if prefs.inspector {
                                    "panelRightClose"
                                } else {
                                    "panelRightOpen"
                                }))
                                .accessibility_label(prefs.text("文献详情"))
                                .tooltip(prefs.text("文献详情"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.preferences.update(cx, |p, cx| {
                                        p.inspector = !p.inspector;
                                        cx.notify();
                                    });
                                    this.persist(cx);
                                })),
                        ),
                )
                .child(
                    h_resizable("workbench-panes")
                        .child(
                            resizable_panel()
                                .size(px(220.))
                                .size_range(px(180.)..px(300.))
                                .visible(prefs.sidebar)
                                .child(self.sidebar(cx)),
                        )
                        .child(
                            resizable_panel()
                                .size_range(px(330.)..px(2400.))
                                .child(self.content(cx)),
                        )
                        .child(
                            resizable_panel()
                                .size(px(350.))
                                .size_range(px(280.)..px(520.))
                                .visible(prefs.inspector)
                                .child(self.inspector(cx)),
                        ),
                );
        }
        if self.helper || self.tray {
            root = root.child(self.condition_panel(cx));
        }
        if let Some(diagnostic) = self.query_diagnostics.first().cloned() {
            let generation = self.generation;
            root = root.child(
                Button::new("locate-query-diagnostic")
                    .ghost()
                    .small()
                    .label(self.text(&diagnostic.code, cx))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if generation != this.generation {
                            return;
                        }
                        let text = this.search.read(cx).value();
                        if text.as_ref() != this.query_diagnostic_text {
                            return;
                        }
                        let range = diagnostic.start as usize..diagnostic.end as usize;
                        if text.get(range.clone()).is_some() {
                            this.search.update(cx, |input, cx| {
                                input.set_selected_range(range, cx);
                                input.focus(window, cx);
                            });
                        }
                    })),
            );
        }
        if let Some(notice) = &self.query_notice {
            root = root.child(
                div()
                    .p_2()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(notice.clone()),
            );
        }
        if let Some(error) = &self.error {
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .p_2()
                    .when(self.helper, |bar| {
                        bar.h(px(40.)).flex_none().text_size(px(11.))
                    })
                    .text_color(cx.theme().danger)
                    .child(div().flex_1().line_clamp(2).child(self.text(error, cx)))
                    .when_some(
                        self.failed_paste_key.clone().filter(|_| self.helper),
                        |bar, key| {
                            bar.child(
                                Button::new("failed-paste-copy")
                                    .ghost()
                                    .small()
                                    .label(self.text("复制引用键", cx))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.copy(key.clone(), cx)
                                    })),
                            )
                        },
                    )
                    .child(
                        Button::new("dismiss-error")
                            .ghost()
                            .small()
                            .icon(icon("x"))
                            .accessibility_label(prefs.text("关闭"))
                            .tooltip(prefs.text("关闭"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.error = None;
                                cx.notify();
                            })),
                    ),
            );
        }
        root
    }
}
