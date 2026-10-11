//! Native controls for the shared query document and tag service.
use super::*;
#[path = "helper_popup.rs"]
mod helper_popup;
use helper_popup::CandidatePopup;
use service::{
    QueryAnalysis, QueryCondition, QueryDocumentRecord, QueryEdit, QueryEditAction, QueryNodeKind,
    QueryStatus, TagAction, TagMutation, TagSnapshot,
};

/// Native session state for ConditionEditor, owned by its workbench.
pub(super) struct ConditionEditor {
    document: QueryDocumentRecord,
    value: Entity<InputState>,
    year_start: Entity<InputState>,
    year_end: Entity<InputState>,
    tags: Vec<service::TagRecord>,
    tag_id: Option<String>,
    _value_subscription: Subscription,
    field: String,
    phrase: bool,
    exclude: bool,
    any: bool,
    join: QueryEditAction,
    target: Option<u32>,
    revision: Option<u64>,
    error: Option<String>,
}

/// Native session state for TagManager, owned by its workbench.
pub(super) struct TagManager {
    snapshot: TagSnapshot,
    name: Entity<InputState>,
    selected: Option<String>,
    path: String,
    cite_key: String,
    confirmation: Option<TagMutation>,
    error: Option<String>,
}

/// Native session state for CandidatePicker, owned by its workbench.
pub(super) struct CandidatePicker {
    trigger: char,
    previous: String,
    filter: Entity<InputState>,
    items: Vec<(String, Option<String>)>,
    cursor: usize,
    target: Option<u32>,
    scroll: ScrollHandle,
    _subscription: Subscription,
    _ime_subscription: Subscription,
}

/// Optional field restriction input, separate from the candidate-name filter.
pub(super) struct OptionalFieldValue {
    field: String,
    previous_scope: Option<String>,
    input: Entity<InputState>,
    _subscription: Subscription,
    _ime_subscription: Subscription,
}

impl TagManager {
    /// Cancel only the innermost pending mutation; returns whether one existed.
    pub(super) fn cancel_confirmation(&mut self) -> bool {
        self.confirmation.take().is_some()
    }
}

impl Workbench {
    /// Reports composition in any search/editor input so window-level shortcuts
    /// never consume the IME candidate navigation or cancellation keys.
    pub(super) fn input_is_composing(&self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.ime_active.is_empty() || !self.ime_settling.is_empty() {
            return true;
        }
        let mut inputs = vec![self.search.clone()];
        if let Some(value) = &self.optional_field_value {
            inputs.push(value.input.clone());
        }
        if let Some(picker) = &self.candidates {
            inputs.push(picker.filter.clone());
        }
        if let Some(editor) = &self.condition_editor {
            inputs.extend([
                editor.value.clone(),
                editor.year_start.clone(),
                editor.year_end.clone(),
            ]);
        }
        if let Some(manager) = &self.tag_manager {
            inputs.push(manager.name.clone());
        }
        inputs.into_iter().any(|input| {
            input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            })
        })
    }

    /// Observe native preedit notifications and unmark-only commits; GPUI does not
    /// emit Change for those transitions. Resume the main query once after commit.
    pub(super) fn synchronize_ime(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut inputs = vec![self.search.clone()];
        if let Some(value) = &self.optional_field_value {
            inputs.push(value.input.clone());
        }
        if let Some(picker) = &self.candidates {
            inputs.push(picker.filter.clone());
        }
        let present: std::collections::HashSet<_> = inputs.iter().map(Entity::entity_id).collect();
        self.ime_active.retain(|id| present.contains(id));
        self.ime_settling.retain(|id| present.contains(id));
        for input in inputs {
            let id = input.entity_id();
            let marked = input.update(cx, |input, cx| {
                input.marked_text_range(window, cx).is_some()
            });
            if marked {
                if self.ime_active.insert(id) {
                    self.pending_trigger = None;
                    self.ime_settling.remove(&id);
                    if id == self.search.entity_id() {
                        self.search_composing = true;
                        self.schedule_search(cx);
                    }
                }
            } else if self.ime_active.remove(&id) {
                self.ime_settling.insert(id);
                if id == self.search.entity_id() {
                    self.search_composing = false;
                    if !self.choosing_library {
                        self.rebind_native_tokens(cx);
                    }
                    self.schedule_search(cx);
                } else if let Some(picker) = self.candidates.as_mut() {
                    picker.cursor = 0;
                    picker.scroll.scroll_to_item(0);
                }
                cx.defer_in(window, move |this, _, cx| {
                    this.ime_settling.remove(&id);
                    cx.notify();
                });
                cx.notify();
            }
        }
        if !self.ime_active.is_empty() && self.ime_watch_task.is_none() {
            self.ime_watch_task = Some(cx.spawn_in(window, async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;
                    let active = this
                        .update_in(cx, |this, window, cx| {
                            this.synchronize_ime(window, cx);
                            let active = !this.ime_active.is_empty();
                            if !active {
                                this.ime_watch_task = None;
                            }
                            active
                        })
                        .unwrap_or(false);
                    if !active {
                        break;
                    }
                }
            }));
        }
    }

    fn search_path(&self) -> String {
        self.libraries
            .iter()
            .find(|l| Some(&l.name) == self.selected_library.as_ref())
            .map(|l| l.path.clone())
            .unwrap_or_default()
    }

    fn localized(&self, chinese: &'static str, english: &'static str, cx: &App) -> &'static str {
        if self.preferences.read(cx).chinese() {
            chinese
        } else {
            english
        }
    }

    fn analysis_error(analysis: &QueryAnalysis, preferences: &Preferences) -> Option<String> {
        (analysis.status != QueryStatus::Valid).then(|| {
            analysis
                .diagnostics
                .iter()
                .map(|diagnostic| preferences.text(&diagnostic.code).to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })
    }

    /// Opens an editable copy of the current valid query; parser failures remain visible and do not replace the draft.
    pub(super) fn edit_conditions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_is_composing(window, cx) {
            return;
        }
        let analysis = self
            .query_document
            .clone()
            .map(service::render_query)
            .unwrap_or_else(|| service::analyze_query(self.search.read(cx).value().to_string()));
        if let Some(error) = Self::analysis_error(&analysis, self.preferences.read(cx)) {
            self.error = Some(error);
            cx.notify();
            return;
        }
        let value = cx.new(|cx| InputState::new(window, cx));
        let value_subscription = cx.subscribe_in(&value, window, |this, _, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                if let Some(editor) = this.condition_editor.as_mut() {
                    editor.tag_id = None;
                }
                cx.notify();
            }
        });
        let tags = match service::tag_snapshot(String::new(), String::new()) {
            Ok(snapshot) => snapshot.tags,
            Err(error) => {
                self.error = Some(error.to_string());
                vec![]
            }
        };
        let chinese = self.preferences.read(cx).chinese();
        self.condition_editor = Some(ConditionEditor {
            document: analysis.document.unwrap_or_default(),
            value,
            year_start: cx.new(|cx| {
                InputState::new(window, cx).placeholder(if chinese {
                    "起始年"
                } else {
                    "From year"
                })
            }),
            year_end: cx.new(|cx| {
                InputState::new(window, cx).placeholder(if chinese {
                    "结束年"
                } else {
                    "To year"
                })
            }),
            tags,
            tag_id: None,
            _value_subscription: value_subscription,
            field: String::new(),
            phrase: false,
            exclude: false,
            any: false,
            join: QueryEditAction::AppendAnd,
            target: None,
            revision: self.tag_revision.or(analysis.tag_revision),
            error: None,
        });
        if let Some(editor) = &self.condition_editor {
            editor.value.update(cx, |input, cx| input.focus(window, cx));
        }
        self.schedule_search(cx);
        self.resize_helper(window, cx);
    }

    fn edit_document(
        &mut self,
        action: QueryEditAction,
        operand: Option<QueryDocumentRecord>,
        cx: &mut Context<Self>,
    ) {
        let Some(editor) = self.condition_editor.as_mut() else {
            return;
        };
        let analysis = service::edit_query(QueryEdit {
            document: editor.document.clone(),
            action,
            target: editor.target,
            operand,
        });
        editor.error = Self::analysis_error(&analysis, self.preferences.read(cx));
        if editor.error.is_none() {
            editor.document = analysis.document.unwrap_or_default();
            editor.revision = analysis.tag_revision.or(editor.revision);
            editor.target = None;
        }
        cx.notify();
    }

    fn add_condition(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_is_composing(window, cx) {
            return;
        }
        let Some(editor) = self.condition_editor.as_ref() else {
            return;
        };
        let mut analysis = service::build_condition(QueryCondition {
            field_name: editor.field.clone(),
            text_value: editor.value.read(cx).value().to_string(),
            phrase: editor.phrase
                && (editor.field.is_empty()
                    || service::search_catalog()
                        .field_descriptors
                        .iter()
                        .any(|field| field.name == editor.field && field.supports_phrase)),
            exclude: editor.exclude,
            tag_id: editor.tag_id.clone(),
        });
        if let Some(error) = Self::analysis_error(&analysis, self.preferences.read(cx)) {
            if let Some(editor) = self.condition_editor.as_mut() {
                editor.error = Some(error);
            }
            cx.notify();
            return;
        }
        if editor.any
            && let Some(document) = analysis.document.as_mut()
        {
            // The form changes the group operator, preserving each typed leaf and order.
            for node in &mut document.nodes {
                if node.kind == QueryNodeKind::And {
                    node.kind = QueryNodeKind::Or;
                }
            }
        }
        let join = editor.join;
        self.edit_document(join, analysis.document, cx);
    }

    fn add_year_range(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_is_composing(window, cx) {
            return;
        }
        let Some(editor) = self.condition_editor.as_ref() else {
            return;
        };
        let value = format!(
            "{}..{}",
            editor.year_start.read(cx).value().trim(),
            editor.year_end.read(cx).value().trim()
        );
        let analysis = service::build_condition(QueryCondition {
            field_name: "year".into(),
            text_value: value,
            phrase: false,
            exclude: editor.exclude,
            tag_id: None,
        });
        if let Some(error) = Self::analysis_error(&analysis, self.preferences.read(cx)) {
            if let Some(editor) = self.condition_editor.as_mut() {
                editor.error = Some(error);
            }
            cx.notify();
            return;
        }
        self.edit_document(editor.join, analysis.document, cx);
    }

    fn apply_conditions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_is_composing(window, cx) {
            return;
        }
        let Some(editor) = self.condition_editor.as_ref() else {
            return;
        };
        let analysis = if editor
            .document
            .nodes
            .iter()
            .any(|node| node.tag_id.is_some())
        {
            match service::refresh_query(editor.document.clone()) {
                Ok(analysis) => analysis,
                Err(error) => {
                    self.error = Some(error.to_string());
                    cx.notify();
                    return;
                }
            }
        } else {
            service::render_query(editor.document.clone())
        };
        if let Some(error) = Self::analysis_error(&analysis, self.preferences.read(cx)) {
            if let Some(editor) = self.condition_editor.as_mut() {
                editor.error = Some(error);
            }
            cx.notify();
            return;
        }
        self.query_document = analysis.document;
        self.tag_revision = analysis.tag_revision.or(editor.revision);
        self.condition_editor = None;
        if !self.install_bound_text(analysis.text, window, cx) {
            cx.notify();
            return;
        }
        self.schedule_search(cx);
        self.focus_search(window, cx);
    }

    /// Builds the condition and tag entry controls for this window without executing a query.
    pub(super) fn condition_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_1()
            .when(self.advanced && !self.choosing_library, |view| {
                view.child(
                    Button::new("edit-search-conditions")
                        .ghost()
                        .small()
                        .label(self.localized("条件", "Conditions", cx))
                        .on_click(
                            cx.listener(|this, _, window, cx| this.edit_conditions(window, cx)),
                        ),
                )
            })
            .child(
                Button::new("manage-tags")
                    .ghost()
                    .small()
                    .label(self.localized("标签", "Tags", cx))
                    .on_click(cx.listener(|this, _, window, cx| this.open_tags(window, cx))),
            )
    }

    /// Renders the current typed tree, basic field controls, tag manager and helper candidates.
    pub(super) fn condition_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div().flex().flex_col();
        if let Some(editor) = &self.condition_editor
            && self.tag_manager.is_none()
        {
            let catalog = service::search_catalog();
            let descriptor = catalog
                .field_descriptors
                .iter()
                .find(|field| field.name == editor.field);
            let supports_phrase =
                editor.field.is_empty() || descriptor.is_some_and(|field| field.supports_phrase);
            let supports_words = editor.field.is_empty()
                || descriptor.is_some_and(|field| field.match_mode == "text");
            let field_label = descriptor
                .map(|field| self.text(&field.label_key, cx))
                .unwrap_or_else(|| self.text("全部字段", cx));
            let fields: Vec<_> = std::iter::once((String::new(), self.text("全部字段", cx), true))
                .chain(catalog.field_descriptors.into_iter().map(|field| {
                    (
                        field.name,
                        self.text(&field.label_key, cx),
                        field.supports_phrase,
                    )
                }))
                .collect();
            let owner = cx.weak_entity();
            let mut nodes = div()
                .id("query-nodes")
                .max_h(px(200.))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_1();
            let mut stack = editor
                .document
                .root
                .map(|root| vec![(root, 0usize)])
                .unwrap_or_default();
            while let Some((index, depth)) = stack.pop() {
                let Some(node) = editor.document.nodes.get(index as usize) else {
                    continue;
                };
                for &child in node.children.iter().rev() {
                    stack.push((child, depth + 1));
                }
                let label = match node.kind {
                    QueryNodeKind::And => "AND".to_owned(),
                    QueryNodeKind::Or => "OR".to_owned(),
                    QueryNodeKind::Not => "NOT".to_owned(),
                    QueryNodeKind::Scope => format!("{}:", node.text_value),
                    QueryNodeKind::Phrase => format!("\"{}\"", node.text_value),
                    QueryNodeKind::Word => node.text_value.clone(),
                };
                nodes = nodes.child(
                    Button::new(("query-node", index as usize))
                        .ghost()
                        .small()
                        .justify_start()
                        .ml(px((depth.min(12) * 12) as f32))
                        .when(editor.target == Some(index), |b| b.bg(cx.theme().muted))
                        .label(label)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(editor) = this.condition_editor.as_mut() {
                                editor.target = Some(index);
                            }
                            cx.notify();
                        })),
                );
            }
            panel = panel
                .p_3()
                .gap_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(nodes)
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            Button::new("query-root")
                                .ghost()
                                .small()
                                .label(self.localized("根组", "Root", cx))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(e) = this.condition_editor.as_mut() {
                                        e.target = None;
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("query-and")
                                .ghost()
                                .small()
                                .label("AND")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(editor) = this.condition_editor.as_mut() {
                                        editor.join = QueryEditAction::AppendAnd;
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("query-or")
                                .ghost()
                                .small()
                                .label("OR")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(editor) = this.condition_editor.as_mut() {
                                        editor.join = QueryEditAction::AppendOr;
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("query-not")
                                .ghost()
                                .small()
                                .label("NOT")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.edit_document(QueryEditAction::Negate, None, cx)
                                })),
                        )
                        .child(
                            Button::new("query-remove")
                                .ghost()
                                .small()
                                .label(self.localized("移除", "Remove", cx))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.edit_document(QueryEditAction::Remove, None, cx)
                                })),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            Button::new("condition-field")
                                .small()
                                .label(field_label)
                                .dropdown_menu(move |mut menu, _, _| {
                                    for (field, label, supports_phrase) in fields.iter().cloned() {
                                        let owner = owner.clone();
                                        menu = menu.item(PopupMenuItem::new(label).on_click(
                                            move |_, _, cx| {
                                                let _ = owner.update(cx, |this, cx| {
                                                    if let Some(editor) =
                                                        this.condition_editor.as_mut()
                                                    {
                                                        editor.field = field.clone();
                                                        editor.tag_id = None;
                                                        if !supports_phrase {
                                                            editor.phrase = false;
                                                            editor.any = false;
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                            },
                                        ));
                                    }
                                    menu
                                }),
                        )
                        .child(Input::new(&editor.value).flex_1().cleanable(true)),
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            Button::new("condition-phrase")
                                .disabled(!supports_phrase)
                                .ghost()
                                .small()
                                .when(editor.phrase, |b| b.bg(cx.theme().muted))
                                .label(self.localized("完整短语", "Exact phrase", cx))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(e) = this.condition_editor.as_mut() {
                                        e.phrase = !e.phrase;
                                        if e.phrase {
                                            e.any = false;
                                        }
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("condition-any")
                                .disabled(!supports_words)
                                .ghost()
                                .small()
                                .when(editor.any, |b| b.bg(cx.theme().muted))
                                .label(self.localized("任意关键词", "Any words", cx))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(e) = this.condition_editor.as_mut() {
                                        e.any = !e.any;
                                        if e.any {
                                            e.phrase = false;
                                            e.exclude = false;
                                        }
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("condition-exclude")
                                .ghost()
                                .small()
                                .when(editor.exclude, |b| b.bg(cx.theme().muted))
                                .label(self.localized("排除", "Exclude", cx))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(e) = this.condition_editor.as_mut() {
                                        e.exclude = !e.exclude;
                                        if e.exclude {
                                            e.any = false;
                                        }
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("condition-add")
                                .small()
                                .label(self.localized("添加条件", "Add condition", cx))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.add_condition(window, cx)
                                })),
                        )
                        .child(
                            Button::new("condition-replace")
                                .ghost()
                                .small()
                                .disabled(editor.target.is_none())
                                .label(self.localized("替换选中条件", "Replace selected", cx))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if let Some(editor) = this.condition_editor.as_mut() {
                                        editor.join = QueryEditAction::Replace;
                                    }
                                    this.add_condition(window, cx);
                                    if let Some(editor) = this.condition_editor.as_mut() {
                                        editor.join = QueryEditAction::AppendAnd;
                                    }
                                })),
                        ),
                )
                .when(editor.field == "year", |v| {
                    v.child(
                        div()
                            .flex()
                            .gap_1()
                            .child(Input::new(&editor.year_start).flex_1())
                            .child(Input::new(&editor.year_end).flex_1())
                            .child(
                                Button::new("add-year-range")
                                    .small()
                                    .label(self.localized("添加年份范围", "Add year range", cx))
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.add_year_range(window, cx)
                                    })),
                            ),
                    )
                })
                .when(editor.field == "tag", |v| {
                    let owner = cx.weak_entity();
                    let tags = editor.tags.clone();
                    v.child(
                        Button::new("choose-condition-tag")
                            .small()
                            .label(self.localized("选择已有标签", "Choose existing tag", cx))
                            .dropdown_menu(move |mut menu, _, _| {
                                for tag in &tags {
                                    let owner = owner.clone();
                                    let tag = tag.clone();
                                    menu =
                                        menu.item(PopupMenuItem::new(tag.name.clone()).on_click(
                                            move |_, window, cx| {
                                                let _ = owner.update(cx, |this, cx| {
                                                    if let Some(editor) =
                                                        this.condition_editor.as_mut()
                                                    {
                                                        editor.tag_id = Some(tag.id.clone());
                                                        editor.value.update(cx, |input, cx| {
                                                            input.set_value(
                                                                tag.name.clone(),
                                                                window,
                                                                cx,
                                                            )
                                                        });
                                                    }
                                                    cx.notify();
                                                });
                                            },
                                        ));
                                }
                                menu
                            }),
                    )
                })
                .when(editor.field == "type", |v| {
                    let owner = cx.weak_entity();
                    let types: Vec<_> = service::search_catalog()
                        .type_descriptors
                        .into_iter()
                        .map(|kind| (kind.name, self.text(&kind.label_key, cx)))
                        .collect();
                    v.child(
                        Button::new("choose-condition-type")
                            .small()
                            .label(self.localized("选择类型", "Choose type", cx))
                            .dropdown_menu(move |mut menu, _, _| {
                                for (kind, label) in &types {
                                    let owner = owner.clone();
                                    let kind = kind.clone();
                                    menu = menu.item(PopupMenuItem::new(label.clone()).on_click(
                                        move |_, window, cx| {
                                            let _ = owner.update(cx, |this, cx| {
                                                if let Some(editor) = this.condition_editor.as_mut()
                                                {
                                                    editor.value.update(cx, |input, cx| {
                                                        input.set_value(kind.clone(), window, cx)
                                                    });
                                                }
                                                cx.notify();
                                            });
                                        },
                                    ));
                                }
                                menu
                            }),
                    )
                })
                .when_some(editor.error.clone(), |v, error| {
                    v.child(div().text_color(cx.theme().danger).child(error))
                })
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            Button::new("apply-conditions")
                                .small()
                                .label(self.text("保存", cx))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.apply_conditions(window, cx)
                                })),
                        )
                        .child(
                            Button::new("cancel-conditions")
                                .ghost()
                                .small()
                                .label(self.text("取消", cx))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.condition_editor = None;
                                    this.schedule_search(cx);
                                    this.focus_search(window, cx);
                                })),
                        ),
                );
        }
        panel.child(self.tag_panel(cx))
    }

    /// Loads the selected bibliography and citation association snapshot; service failures are shown without mutating tags.
    pub(super) fn open_tags(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_some() && !self.current_results_valid(cx) {
            return;
        }
        let path = self.search_path();
        let cite_key = self
            .selected
            .and_then(|i| self.references.get(i))
            .map(|r| r.cite_key.clone())
            .unwrap_or_default();
        match service::tag_snapshot(path.clone(), cite_key.clone()) {
            Ok(snapshot) => {
                self.tag_manager = Some(TagManager {
                    snapshot,
                    name: cx.new(|cx| InputState::new(window, cx)),
                    selected: None,
                    path,
                    cite_key,
                    confirmation: None,
                    error: None,
                });
                self.schedule_search(cx);
            }
            Err(error) => self.error = Some(error.to_string()),
        }
        cx.notify();
    }

    fn change_tag(
        &mut self,
        action: TagAction,
        target: String,
        confirm: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(manager) = self.tag_manager.as_mut() else {
            return;
        };
        let mutation = TagMutation {
            action,
            id: manager.selected.clone().unwrap_or_default(),
            name: manager.name.read(cx).value().to_string(),
            target_id: target,
            path: manager.path.clone(),
            cite_key: manager.cite_key.clone(),
            expected_revision: manager.snapshot.revision,
        };
        if confirm {
            manager.confirmation = Some(mutation);
            cx.notify();
            return;
        }
        self.apply_tag_mutation(mutation, cx);
    }

    fn apply_tag_mutation(&mut self, mutation: TagMutation, cx: &mut Context<Self>) {
        let relocation = (mutation.action == TagAction::Relocate).then(|| {
            let old_path = self
                .tag_manager
                .as_ref()
                .and_then(|manager| {
                    manager
                        .snapshot
                        .sources
                        .iter()
                        .find(|source| source.id == mutation.id)
                })
                .map(|source| source.path.clone());
            (old_path, mutation.path.clone())
        });
        match service::mutate_tag(mutation) {
            Ok(snapshot) => {
                if let Some(manager) = self.tag_manager.as_mut() {
                    manager.snapshot = snapshot;
                    manager.confirmation = None;
                    manager.error = None;
                    if let Some((Some(old_path), new_path)) = &relocation {
                        if manager.path == *old_path {
                            manager.path = new_path.clone();
                        }
                        match service::tag_snapshot(manager.path.clone(), manager.cite_key.clone())
                        {
                            Ok(snapshot) => manager.snapshot = snapshot,
                            Err(error) => manager.error = Some(error.to_string()),
                        }
                    }
                }
            }
            Err(error) => {
                if let Some(manager) = self.tag_manager.as_mut() {
                    manager.error = Some(error.to_string());
                    manager.confirmation = None;
                }
            }
        }
        // A tag revision invalidates every result, including pure negative tag queries.
        self.preferences.update(cx, |_, cx| cx.notify());
        self.generation += 1;
        self.result_context = None;
        self.references.clear();
        self.selected = None;
        cx.notify();
    }

    fn relocate_source(&mut self, source_id: String, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(self.text("选择文件", cx)),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = prompt.await;
            let _ = this.update_in(cx, |this, _, cx| {
                let Some(manager) = this.tag_manager.as_mut() else {
                    return;
                };
                match result {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.first() {
                            manager.confirmation = Some(TagMutation {
                                action: TagAction::Relocate,
                                id: source_id,
                                path: path.to_string_lossy().into_owned(),
                                name: String::new(),
                                target_id: String::new(),
                                cite_key: String::new(),
                                expected_revision: manager.snapshot.revision,
                            });
                        }
                    }
                    Ok(Err(error)) => manager.error = Some(error.to_string()),
                    Err(error) => manager.error = Some(error.to_string()),
                    _ => {}
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn tag_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .id("tag-management-panel")
            .max_h(px(420.))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_2();
        let Some(manager) = &self.tag_manager else {
            return panel;
        };
        panel =
            panel
                .p_3()
                .child(div().font_weight(FontWeight::SEMIBOLD).child(
                    if manager.cite_key.is_empty() {
                        self.localized("标签管理", "Manage tags", cx).to_owned()
                    } else {
                        format!(
                            "{} · {}",
                            self.localized("标签", "Tags", cx),
                            manager.cite_key
                        )
                    },
                ));
        let mut tags = div()
            .id("tag-catalog")
            .max_h(px(180.))
            .overflow_y_scroll()
            .flex()
            .flex_col();
        for (index, tag) in manager.snapshot.tags.iter().enumerate() {
            let id = tag.id.clone();
            let name = tag.name.clone();
            let assigned = manager.snapshot.assigned_ids.contains(&id);
            tags = tags.child(
                Button::new(("tag", index))
                    .ghost()
                    .small()
                    .justify_start()
                    .when(manager.selected.as_ref() == Some(&id), |b| {
                        b.bg(cx.theme().muted)
                    })
                    .label(format!("{}{}", if assigned { "✓ " } else { "" }, tag.name))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if let Some(manager) = this.tag_manager.as_mut() {
                            manager.selected = Some(id.clone());
                            manager
                                .name
                                .update(cx, |input, cx| input.set_value(name.clone(), window, cx));
                        }
                        cx.notify();
                    })),
            );
        }
        panel = panel
            .child(tags)
            .child(Input::new(&manager.name).cleanable(true));
        let selected = manager.selected.is_some();
        let assigned = manager
            .selected
            .as_ref()
            .is_some_and(|id| manager.snapshot.assigned_ids.contains(id));
        let owner = cx.weak_entity();
        let merge_tags = manager.snapshot.tags.clone();
        let selected_id = manager.selected.clone();
        panel = panel.child(
            div()
                .flex()
                .flex_wrap()
                .gap_1()
                .child(
                    Button::new("tag-create")
                        .small()
                        .label(self.localized("新建", "Create", cx))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.change_tag(TagAction::Create, String::new(), false, cx)
                        })),
                )
                .child(
                    Button::new("tag-rename")
                        .small()
                        .disabled(!selected)
                        .label(self.localized("重命名", "Rename", cx))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.change_tag(TagAction::Rename, String::new(), false, cx)
                        })),
                )
                .child(
                    Button::new("tag-delete")
                        .ghost()
                        .small()
                        .disabled(!selected)
                        .label(self.localized("删除", "Delete", cx))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.change_tag(TagAction::Delete, String::new(), true, cx)
                        })),
                )
                .child(
                    Button::new("tag-merge")
                        .ghost()
                        .small()
                        .disabled(!selected)
                        .label(self.localized("合并到", "Merge into", cx))
                        .dropdown_menu(move |mut menu, _, _| {
                            for tag in merge_tags
                                .iter()
                                .filter(|tag| Some(&tag.id) != selected_id.as_ref())
                            {
                                let owner = owner.clone();
                                let id = tag.id.clone();
                                menu = menu.item(PopupMenuItem::new(tag.name.clone()).on_click(
                                    move |_, _, cx| {
                                        let _ = owner.update(cx, |this, cx| {
                                            this.change_tag(TagAction::Merge, id.clone(), true, cx)
                                        });
                                    },
                                ));
                            }
                            menu
                        }),
                )
                .when(!manager.cite_key.is_empty(), |v| {
                    v.child(
                        Button::new("tag-assignment")
                            .small()
                            .disabled(!selected)
                            .label(if assigned {
                                self.localized("移除关联", "Unassign", cx)
                            } else {
                                self.localized("关联文献", "Assign", cx)
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.change_tag(
                                    if assigned {
                                        TagAction::Unassign
                                    } else {
                                        TagAction::Assign
                                    },
                                    String::new(),
                                    false,
                                    cx,
                                )
                            })),
                    )
                }),
        );
        if !manager.snapshot.pending.is_empty() {
            panel = panel.child(div().child(self.localized("待关联", "Pending associations", cx)));
            for (index, pending) in manager.snapshot.pending.iter().enumerate() {
                let pending_id = pending.record_id.clone();
                panel = panel.child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .truncate()
                                .child(format!("{} · {}", pending.cite_key, pending.path)),
                        )
                        .child(
                            Button::new(("relink", index))
                                .small()
                                .disabled(manager.cite_key.is_empty())
                                .label(self.localized("关联到当前文献", "Link to selected", cx))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if let Some(manager) = this.tag_manager.as_mut() {
                                        manager.confirmation = Some(TagMutation {
                                            action: TagAction::Relink,
                                            id: pending_id.clone(),
                                            name: String::new(),
                                            target_id: String::new(),
                                            path: manager.path.clone(),
                                            cite_key: manager.cite_key.clone(),
                                            expected_revision: manager.snapshot.revision,
                                        });
                                    }
                                    cx.notify();
                                })),
                        ),
                );
            }
        }
        if !manager.snapshot.sources.is_empty() {
            panel =
                panel.child(div().child(self.localized("文献来源", "Bibliography sources", cx)));
            for (index, source) in manager.snapshot.sources.iter().enumerate() {
                let source_id = source.id.clone();
                panel = panel.child(
                    div()
                        .flex()
                        .gap_2()
                        .child(div().flex_1().truncate().child(format!(
                            "{}{}",
                            source.path,
                            if source.available {
                                ""
                            } else {
                                " · unavailable"
                            }
                        )))
                        .child(
                            Button::new(("relocate-source", index))
                                .ghost()
                                .small()
                                .label(self.localized(
                                    "重新定位并保留标签",
                                    "Relocate, keep tags",
                                    cx,
                                ))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.relocate_source(source_id.clone(), window, cx)
                                })),
                        ),
                );
            }
        }
        if let Some(mutation) = &manager.confirmation {
            let confirmation = match mutation.action {
                TagAction::Relocate => format!(
                    "{} → {}",
                    manager
                        .snapshot
                        .sources
                        .iter()
                        .find(|source| source.id == mutation.id)
                        .map(|source| source.path.as_str())
                        .unwrap_or(""),
                    mutation.path
                ),
                TagAction::Merge => format!(
                    "{} → {}",
                    manager
                        .snapshot
                        .tags
                        .iter()
                        .find(|tag| tag.id == mutation.id)
                        .map(|tag| tag.name.as_str())
                        .unwrap_or(""),
                    manager
                        .snapshot
                        .tags
                        .iter()
                        .find(|tag| tag.id == mutation.target_id)
                        .map(|tag| tag.name.as_str())
                        .unwrap_or("")
                ),
                TagAction::Relink => format!(
                    "{} → {}",
                    manager
                        .snapshot
                        .pending
                        .iter()
                        .find(|pending| pending.record_id == mutation.id)
                        .map(|pending| pending.cite_key.as_str())
                        .unwrap_or(""),
                    mutation.cite_key
                ),
                _ => manager
                    .snapshot
                    .tags
                    .iter()
                    .find(|tag| tag.id == mutation.id)
                    .map(|tag| tag.name.clone())
                    .unwrap_or_default(),
            };
            let mutation = mutation.clone();
            panel = panel.child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().child(format!(
                        "{} {}",
                        self.localized("确认变更", "Confirm change", cx),
                        confirmation
                    )))
                    .child(
                        Button::new("tag-confirm")
                            .small()
                            .label(self.localized("确认", "Confirm", cx))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.apply_tag_mutation(mutation.clone(), cx)
                            })),
                    )
                    .child(
                        Button::new("tag-cancel")
                            .ghost()
                            .small()
                            .label(self.text("取消", cx))
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(manager) = this.tag_manager.as_mut() {
                                    manager.confirmation = None;
                                }
                                cx.notify();
                            })),
                    ),
            );
        }
        panel
            .child(
                Button::new("refresh-tag-snapshot")
                    .ghost()
                    .small()
                    .label(self.text("刷新", cx))
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(manager) = this.tag_manager.as_mut() {
                            match service::tag_snapshot(
                                manager.path.clone(),
                                manager.cite_key.clone(),
                            ) {
                                Ok(snapshot) => {
                                    manager.snapshot = snapshot;
                                    manager.error = None;
                                    manager.confirmation = None;
                                }
                                Err(error) => manager.error = Some(error.to_string()),
                            }
                        }
                        cx.notify();
                    })),
            )
            .when_some(manager.error.clone(), |v, error| {
                v.child(div().text_color(cx.theme().danger).child(error))
            })
            .child(
                Button::new("close-tag-manager")
                    .ghost()
                    .small()
                    .label(self.text("关闭", cx))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.tag_manager = None;
                        this.reload(cx);
                        this.focus_search(window, cx);
                    })),
            )
    }

    /// Arms a typed helper trigger only at the end of a complete query and outside IME composition.
    pub(super) fn prepare_candidate_trigger(
        &mut self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.helper
            || !self.advanced
            || self.choosing_library
            || self.condition_editor.is_some()
            || self.candidates.is_some()
        {
            return;
        }
        let Some(trigger) = key.chars().next().filter(|c| matches!(c, '@' | '$' | '#')) else {
            return;
        };
        if key.chars().count() != 1 {
            return;
        }
        let text = self.search.read(cx).value().to_string();
        if self.search.read(cx).cursor() != text.len()
            || (!text.is_empty() && !text.ends_with(char::is_whitespace))
        {
            return;
        }
        if self.search.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_some()
        }) {
            return;
        }
        // A complete prefix excludes quoted/unfinished field values. A leaf ending
        // at the final whitespace means that whitespace belongs to an escaped word.
        let analysis = service::analyze_query(text.clone());
        if analysis.status != QueryStatus::Valid
            || analysis.document.as_ref().is_some_and(|doc| {
                doc.nodes.iter().any(|node| {
                    matches!(node.kind, QueryNodeKind::Word | QueryNodeKind::Phrase)
                        && node.end as usize == text.len()
                })
            })
        {
            return;
        }
        let mut escaped = false;
        let mut quoted = false;
        let mut formula = false;
        for character in text.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            match character {
                '\\' => escaped = true,
                '"' => quoted = !quoted,
                '$' if !quoted => formula = !formula,
                _ => {}
            }
        }
        if formula {
            return;
        }
        self.pending_trigger = Some((trigger, text));
    }

    /// Opens candidates only for the exact armed keyboard insertion; programmatic changes and paste do not trigger it.
    pub(super) fn handle_candidate_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.candidates.is_some() {
            self.candidates = None;
            self.close_candidate_window(cx);
        }
        let Some((trigger, previous)) = self.pending_trigger.take() else {
            return;
        };
        if self.search.read(cx).value().as_ref() != format!("{previous}{trigger}") {
            return;
        }
        let expected_text = self.search.read(cx).value().to_string();
        let expected_generation = self.generation;
        cx.defer_in(window, move |this, window, cx| {
            if this.generation != expected_generation + 1
                || this.search.read(cx).value().as_ref() != expected_text
                || this.input_is_composing(window, cx)
            {
                return;
            }
            if !this.install_bound_text(previous.clone(), window, cx) {
                return;
            }
            this.start_candidate_picker(trigger, previous, None, window, cx);
        });
    }

    fn start_candidate_picker(
        &mut self,
        trigger: char,
        previous: String,
        target: Option<u32>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.input_is_composing(window, cx) || self.optional_field_value.is_some() {
            return;
        }
        self.close_candidate_window(cx);
        let catalog = service::search_catalog();
        let items = match trigger {
            '@' => catalog
                .type_descriptors
                .into_iter()
                .map(|descriptor| (descriptor.name, None))
                .collect(),
            '$' => catalog
                .field_descriptors
                .into_iter()
                .map(|descriptor| (descriptor.name, None))
                .collect(),
            '#' => match service::tag_snapshot(self.search_path(), String::new()) {
                Ok(snapshot) => snapshot
                    .tags
                    .into_iter()
                    .map(|t| (t.name, Some(t.id)))
                    .collect(),
                Err(error) => {
                    self.error = Some(error.to_string());
                    vec![]
                }
            },
            _ => vec![],
        };
        let filter = cx.new(|cx| InputState::new(window, cx));
        let ime_subscription = cx.observe_in(&filter, window, |this, _, window, cx| {
            this.synchronize_ime(window, cx)
        });
        let expected_filter = filter.entity_id();
        let subscription = cx.subscribe_in(&filter, window, move |this, _, event, window, cx| {
            if this
                .candidates
                .as_ref()
                .is_none_or(|picker| picker.filter.entity_id() != expected_filter)
            {
                return;
            }
            match event {
                InputEvent::Change => {
                    if let Some(picker) = this.candidates.as_mut() {
                        picker.cursor = 0;
                        picker.scroll.scroll_to_item(0);
                    }
                    cx.notify();
                }
                InputEvent::PressEnter { .. } => this.confirm_candidate(window, cx),
                _ => {}
            }
        });
        self.candidates = Some(CandidatePicker {
            trigger,
            previous,
            filter,
            items,
            cursor: 0,
            target,
            scroll: ScrollHandle::new(),
            _subscription: subscription,
            _ime_subscription: ime_subscription,
        });
        self.search
            .update(cx, |input, cx| input.set_placeholder("", window, cx));
        self.open_candidate_window(window, cx);
        self.focus_candidates(window, cx);
        self.schedule_search(cx);
        self.resize_helper(window, cx);
    }

    fn candidate_label(&self, trigger: char, name: &str, cx: &App) -> SharedString {
        let catalog = service::search_catalog();
        let key = match trigger {
            '@' => catalog
                .type_descriptors
                .iter()
                .find(|descriptor| descriptor.name == name)
                .map(|descriptor| descriptor.label_key.as_str()),
            '$' => catalog
                .field_descriptors
                .iter()
                .find(|descriptor| descriptor.name == name)
                .map(|descriptor| descriptor.label_key.as_str()),
            _ => None,
        };
        key.map(|key| self.text(key, cx))
            .unwrap_or_else(|| name.to_owned().into())
    }

    fn english_candidate_label(&self, trigger: char, name: &str, cx: &App) -> SharedString {
        let catalog = service::search_catalog();
        let key = match trigger {
            '@' => catalog
                .type_descriptors
                .iter()
                .find(|entry| entry.name == name)
                .map(|entry| entry.label_key.as_str()),
            '$' => catalog
                .field_descriptors
                .iter()
                .find(|entry| entry.name == name)
                .map(|entry| entry.label_key.as_str()),
            _ => None,
        };
        let mut english = self.preferences.read(cx).clone();
        english.language = "en".into();
        key.map(|key| english.text(key))
            .unwrap_or_else(|| name.to_owned().into())
    }

    fn candidate_items(&self, cx: &App) -> Vec<(String, Option<String>)> {
        let Some(picker) = self.candidates.as_ref() else {
            return vec![];
        };
        let filter = picker.filter.read(cx).value().to_lowercase();
        picker
            .items
            .iter()
            .filter(|(name, _)| {
                name.to_lowercase().contains(&filter)
                    || self
                        .candidate_label(picker.trigger, name, cx)
                        .to_lowercase()
                        .contains(&filter)
                    || (self.preferences.read(cx).chinese()
                        && self
                            .english_candidate_label(picker.trigger, name, cx)
                            .to_lowercase()
                            .contains(&filter))
            })
            .cloned()
            .collect()
    }

    /// Moves the candidate selection by a signed offset and clamps it to the filtered catalog.
    pub(super) fn candidate_navigation(&mut self, direction: isize, cx: &mut Context<Self>) {
        let count = self.candidate_items(cx).len();
        if let Some(picker) = self.candidates.as_mut() {
            picker.cursor = (picker.cursor as isize + direction)
                .clamp(0, count.saturating_sub(1) as isize) as usize;
            picker.scroll.scroll_to_item(picker.cursor);
        }
        cx.notify();
    }

    /// Restores the pre-trigger query and its native ID annotations, then resumes searching.
    pub(super) fn cancel_candidate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = self.candidates.take() {
            self.close_candidate_window(cx);
            self.install_bound_text(picker.previous, window, cx);
            self.focus_query_tail(window, cx);
            self.schedule_search(cx);
            self.focus_search(window, cx);
        }
    }

    /// Moves focus into the optional candidate filter while retaining the main query draft.
    pub(super) fn focus_candidates(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(picker) = &self.candidates {
            picker
                .filter
                .update(cx, |input, cx| input.focus(window, cx));
        }
    }

    /// Apply a selected type/tag immediately, or select a keyword scope and offer
    /// its separate optional restriction input. IME submission never confirms a row.
    pub(super) fn confirm_candidate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_is_composing(window, cx) {
            return;
        }
        let Some(picker) = self.candidates.as_ref() else {
            return;
        };
        if picker.filter.update(cx, |input, cx| {
            input.marked_text_range(window, cx).is_some()
        }) {
            return;
        }
        let Some((name, tag_id)) = self.candidate_items(cx).get(picker.cursor).cloned() else {
            return;
        };
        let previous = picker.previous.clone();
        let trigger = picker.trigger;
        let target = picker.target;
        let previous_scope = self.helper_scope.clone();
        let selected_field = (trigger == '$').then(|| name.clone());
        if trigger == '$' {
            self.helper_scope = Some(name);
        } else {
            let base = self.query_document.clone().unwrap_or_else(|| {
                service::analyze_query(previous.clone())
                    .document
                    .unwrap_or_default()
            });
            let operand = service::build_condition(QueryCondition {
                field_name: if trigger == '@' { "type" } else { "tag" }.into(),
                text_value: name,
                phrase: false,
                exclude: false,
                tag_id,
            });
            if let Some(error) = Self::analysis_error(&operand, self.preferences.read(cx)) {
                self.error = Some(error);
                cx.notify();
                return;
            }
            let mut analysis = service::edit_query(QueryEdit {
                document: base,
                action: if target.is_some() {
                    QueryEditAction::Replace
                } else {
                    QueryEditAction::AppendAnd
                },
                target,
                operand: operand.document,
            });
            if let Some(document) = analysis.document.clone()
                && document.nodes.iter().any(|node| node.tag_id.is_some())
            {
                match service::refresh_query(document) {
                    Ok(result) => analysis = result,
                    Err(error) => {
                        self.error = Some(error.to_string());
                        cx.notify();
                        return;
                    }
                }
            }
            if let Some(error) = Self::analysis_error(&analysis, self.preferences.read(cx)) {
                self.error = Some(error);
                cx.notify();
                return;
            }
            self.query_document = analysis.document;
            self.tag_revision = analysis.tag_revision.or(self.tag_revision);
            if !self.install_bound_text(analysis.text, window, cx) {
                cx.notify();
                return;
            }
        }
        self.candidates = None;
        self.close_candidate_window(cx);
        if let Some(field) = selected_field {
            self.open_optional_field(field, previous_scope, window, cx);
        } else {
            self.focus_query_tail(window, cx);
            self.schedule_search(cx);
            self.resize_helper(window, cx);
        }
    }

    fn tag_capsule_ranges(
        document: &QueryDocumentRecord,
    ) -> std::collections::HashMap<u32, std::ops::Range<usize>> {
        let parents: std::collections::HashMap<u32, usize> = document
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.kind == QueryNodeKind::Not && node.children.len() == 1)
            .map(|(index, node)| (node.children[0], index))
            .collect();
        document
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.kind == QueryNodeKind::Scope
                    && node.text_value == "tag"
                    && node.children.len() == 1
            })
            .map(|(index, node)| {
                let mut outer = index;
                while let Some(parent) = parents.get(&(outer as u32)) {
                    outer = *parent;
                }
                let outer = &document.nodes[outer];
                (node.children[0], outer.start as usize..outer.end as usize)
            })
            .collect()
    }

    /// Attach stable tag IDs to exact native byte ranges; the input engine moves or
    /// removes these annotations with edits, composition, paste and undo. Returns
    /// false without changing the input or binding map if a range is invalid.
    pub(super) fn install_bound_text(
        &mut self,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.input_is_composing(window, cx) {
            return false;
        }
        let previous_selection = self.search.read(cx).selected_range();
        let previous_length = self.search.read(cx).value().len();
        // Keep annotations belonging to inactive mode/library drafts. Replacing
        // the active InputContent clears its native undo history, so older active
        // annotations no longer need an identity entry.
        let retained: std::collections::HashSet<String> = self
            .inactive_draft
            .content
            .iter()
            .chain(
                self.library_draft
                    .as_ref()
                    .and_then(|draft| draft.content.as_ref()),
            )
            .flat_map(|content| {
                content
                    .tokens()
                    .iter()
                    .map(|token| token.token().id().to_string())
            })
            .collect();
        let mut bindings = self.native_tag_bindings.clone();
        bindings.retain(|key, _| retained.contains(key));
        let mut content = input::InputContent::new(text.clone());
        if let Some(document) = &self.query_document {
            let scope_ranges = Self::tag_capsule_ranges(document);
            for (index, node) in document.nodes.iter().enumerate() {
                if let Some(id) = &node.tag_id {
                    // Including the tag prefix keeps a leading combining mark
                    // inside a complete grapheme and makes field edits atomic.
                    let range = scope_ranges
                        .get(&(index as u32))
                        .cloned()
                        .unwrap_or(node.start as usize..node.end as usize);
                    let Some(value) = text.get(range.clone()) else {
                        self.error = Some(self.text("InvalidTagBinding", cx).to_string());
                        return false;
                    };
                    {
                        self.native_tag_serial = self.native_tag_serial.wrapping_add(1);
                        let annotation = self.native_tag_serial.to_string();
                        bindings.insert(annotation.clone(), id.clone());
                        match content.clone().with_token(
                            range,
                            input::InlineToken::new(annotation, value.to_owned()).with_label(
                                format!(
                                    "{}{}",
                                    if value.starts_with('-') || value.starts_with("NOT") {
                                        "− #"
                                    } else {
                                        "#"
                                    },
                                    node.text_value
                                ),
                            ),
                        ) {
                            Ok(updated) => content = updated,
                            Err(error) => {
                                self.error = Some(error.to_string());
                                return false;
                            }
                        }
                    }
                }
            }
        }
        if self.helper
            && let Some(document) = &self.query_document
        {
            let mut stack: Vec<u32> = document.root.into_iter().collect();
            while let Some(index) = stack.pop() {
                let Some(node) = document.nodes.get(index as usize) else {
                    continue;
                };
                if node.kind == QueryNodeKind::And {
                    stack.extend(node.children.iter().rev().copied());
                    continue;
                }
                let condition = if node.kind == QueryNodeKind::Not {
                    node.children
                        .first()
                        .and_then(|child| document.nodes.get(*child as usize))
                } else {
                    Some(node)
                };
                let Some(condition) = condition.filter(|node| node.kind == QueryNodeKind::Scope)
                else {
                    continue;
                };
                let range = node.start as usize..node.end as usize;
                if content
                    .tokens()
                    .iter()
                    .any(|token| token.range().start < range.end && range.start < token.range().end)
                {
                    continue;
                }
                if let Some(value) = text.get(range.clone()) {
                    self.native_tag_serial = self.native_tag_serial.wrapping_add(1);
                    let annotation = format!("condition:{}", self.native_tag_serial);
                    match content.clone().with_token(
                        range,
                        input::InlineToken::new(annotation, value.to_owned()).with_label({
                            let field = service::search_catalog()
                                .field_descriptors
                                .into_iter()
                                .find(|field| field.name == condition.text_value)
                                .map(|field| self.text(&field.label_key, cx).to_string())
                                .unwrap_or_else(|| condition.text_value.clone());
                            let operand = condition
                                .children
                                .first()
                                .and_then(|index| document.nodes.get(*index as usize));
                            let value = operand
                                .map(|operand| {
                                    if matches!(
                                        operand.kind,
                                        QueryNodeKind::Word | QueryNodeKind::Phrase
                                    ) {
                                        operand.text_value.clone()
                                    } else {
                                        text.get(operand.start as usize..operand.end as usize)
                                            .unwrap_or_default()
                                            .to_owned()
                                    }
                                })
                                .unwrap_or_default();
                            format!(
                                "{}{}: {}",
                                if node.kind == QueryNodeKind::Not {
                                    "− "
                                } else {
                                    ""
                                },
                                field,
                                value
                            )
                        }),
                    ) {
                        Ok(updated) => content = updated,
                        Err(error) => {
                            self.error = Some(error.to_string());
                            return false;
                        }
                    }
                }
            }
        }
        self.native_tag_bindings = bindings;
        self.search.update(cx, |input, cx| {
            let new_length = content.text().len();
            input.set_value(content, window, cx);
            let selection =
                if previous_selection.is_empty() && previous_selection.end == previous_length {
                    new_length..new_length
                } else {
                    previous_selection.start.min(new_length)..previous_selection.end.min(new_length)
                };
            input.set_selected_range(selection, cx);
        });
        true
    }

    /// Reattach only annotations surviving native editing and still belonging to
    /// a tag-scoped leaf. The native ranges, never fuzzy text matching, own identity.
    /// Keeps only ID annotations whose exact native ranges still identify tag-scoped leaves.
    pub(super) fn rebind_native_tokens(&mut self, cx: &mut Context<Self>) {
        if !self.advanced {
            self.query_document = None;
            return;
        }
        let content = self.search.read(cx).content();
        let analysis = service::analyze_query(content.text().to_string());
        let Some(mut document) = analysis.document else {
            self.query_document = None;
            return;
        };
        let scope_ranges = Self::tag_capsule_ranges(&document);
        let mut bound = false;
        let mut retained = std::collections::HashSet::new();
        let mut stack = document
            .root
            .map(|id| vec![(id, false)])
            .unwrap_or_default();
        while let Some((index, inherited_tag)) = stack.pop() {
            let Some(node) = document.nodes.get_mut(index as usize) else {
                continue;
            };
            let tag_scope = if node.kind == QueryNodeKind::Scope {
                node.text_value == "tag"
            } else {
                inherited_tag
            };
            if tag_scope
                && matches!(node.kind, QueryNodeKind::Word | QueryNodeKind::Phrase)
                && let Some(token) = content.tokens().iter().find(|token| {
                    token.range() == (node.start as usize..node.end as usize)
                        || scope_ranges
                            .get(&index)
                            .is_some_and(|range| *range == token.range())
                })
                && let Some(tag_id) = self.native_tag_bindings.get(token.token().id().as_ref())
            {
                node.tag_id = Some(tag_id.clone());
                retained.insert(token.token().id().to_string());
                bound = true;
            }
            stack.extend(node.children.iter().map(|&id| (id, tag_scope)));
        }
        // Editing a tag's field scope revokes that occurrence permanently; merely
        // typing `tag:` again must not resurrect an old identity annotation.
        for token in content.tokens() {
            if !retained.contains(token.token().id().as_ref()) {
                self.native_tag_bindings.remove(token.token().id().as_ref());
            }
        }
        self.query_document = bound.then_some(document);
    }

    /// Returns true only for the current query version and a service-validated source/tag snapshot; stale or failed validation clears actionable results.
    pub(super) fn current_results_valid(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(context) = self.result_context.clone() else {
            return false;
        };
        if context.query_version != self.generation {
            return false;
        }
        match service::validate_search_context(context) {
            Ok(true) => true,
            Ok(false) => {
                self.schedule_search(cx);
                false
            }
            Err(error) => {
                self.result_context = None;
                self.references.clear();
                self.selected = None;
                self.error = Some(error.to_string());
                cx.notify();
                false
            }
        }
    }
    /// Close the native candidate surface without touching the active query.
    pub(super) fn close_candidate_window(&mut self, cx: &mut App) {
        if let Some(handle) = self.candidate_window.take() {
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
    }

    fn open_candidate_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_candidate_window(cx);
        let parent = window.window_handle();
        let owner = cx.entity();
        let labels: Vec<_> = self
            .candidates
            .as_ref()
            .map(|picker| {
                self.candidate_items(cx)
                    .iter()
                    .map(|(name, _)| self.candidate_label(picker.trigger, name, cx))
                    .collect()
            })
            .unwrap_or_default();
        let popup_size = helper_popup::candidate_size(&labels);
        let anchor_x = (window.viewport_size().width - px(18.) - px(180.)).max(px(18.));
        let mut options = crate::window_options(
            "BibCiTeX",
            popup_size.width.into(),
            popup_size.height.into(),
        );
        options.titlebar = None;
        options.focus = false;
        options.is_resizable = false;
        options.is_minimizable = false;
        options.window_min_size = Some(size(px(140.), px(40.)));
        options.kind = WindowKind::AnchoredPopup(popup::PopupOptions {
            parent,
            anchor_rect: Bounds::new(point(anchor_x, px(8.)), size(px(180.), px(48.))),
            anchor: popup::PopupAnchor::BottomLeft,
            gravity: popup::PopupGravity::BottomRight,
            constraint_adjustment: popup::PopupConstraintAdjustment::all(),
            offset: point(px(0.), px(6.)),
            grab: false,
        });
        let make = |window: &mut Window, cx: &mut App| {
            cx.new(|cx| CandidatePopup::new(owner.clone(), parent, window, cx))
        };
        match open_window(options, cx, make) {
            Ok((handle, _)) => self.candidate_window = Some(handle),
            Err(_) => {
                // Platforms lacking native anchored popups still get a separate
                // window; an in-window fallback would clip at the compact header.
                let mut options = crate::window_options(
                    "BibCiTeX",
                    popup_size.width.into(),
                    popup_size.height.into(),
                );
                options.titlebar = None;
                options.focus = false;
                options.is_resizable = false;
                options.is_minimizable = false;
                options.window_min_size = Some(size(px(140.), px(40.)));
                options.kind = WindowKind::PopUp;
                let bounds = window.bounds();
                options.window_bounds = Some(WindowBounds::Windowed(Bounds::new(
                    bounds.origin + point(anchor_x, px(HELPER_HEADER)),
                    popup_size,
                )));
                match open_window(options, cx, |window, cx| {
                    cx.new(|cx| CandidatePopup::new(owner, parent, window, cx))
                }) {
                    Ok((handle, _)) => self.candidate_window = Some(handle),
                    Err(error) => {
                        self.error = Some(error.to_string());
                        self.candidates = None;
                    }
                }
            }
        }
    }

    /// Focus only at confirmation/cancellation boundaries, keeping the caret after
    /// all confirmed capsules and leaving ordinary selection edits untouched.
    fn focus_query_tail(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input_is_composing(window, cx) {
            return;
        }
        let placeholder = self.text("搜索文献、作者、标题", cx);
        self.search.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx)
        });
        window.activate_window();
        self.search.update(cx, |input, cx| {
            let mut end = input.value().len();
            input.set_selected_range(end..end, cx);
            if input.tokens().last().is_some_and(|token| {
                input
                    .value()
                    .get(token.range().end..end)
                    .is_some_and(|suffix| suffix.chars().all(|character| character == ')'))
            }) {
                input.insert(" ", window, cx);
                end = input.value().len();
            }
            input.set_selected_range(end..end, cx);
            input.focus(window, cx);
        });
    }

    /// Edit type/tag capsules by reopening their name picker. Other expressions
    /// remain atomically selectable and editable as source text.
    pub(super) fn edit_inline_token(
        &mut self,
        range: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.helper || self.candidates.is_some() || self.input_is_composing(window, cx) {
            return;
        }
        self.rebind_native_tokens(cx);
        let document = self.query_document.clone().unwrap_or_else(|| {
            service::analyze_query(self.search.read(cx).value().to_string())
                .document
                .unwrap_or_default()
        });
        let selected = document.nodes.iter().enumerate().find(|(_, node)| {
            node.start as usize == range.start
                && node.end as usize == range.end
                && node.kind == QueryNodeKind::Scope
        });
        if let Some((index, node)) = selected
            && matches!(node.text_value.as_str(), "tag" | "type")
        {
            let trigger = if node.text_value == "tag" { '#' } else { '@' };
            self.start_candidate_picker(
                trigger,
                self.search.read(cx).value().to_string(),
                Some(index as u32),
                window,
                cx,
            );
        } else {
            self.search.update(cx, |input, cx| {
                input.set_selected_range(range, cx);
                input.focus(window, cx);
            });
        }
    }

    /// Inline input filters candidate names only; it never collects a field value.
    pub(super) fn helper_filter_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().when_some(self.candidates.as_ref(), |view, picker| {
            view.flex_none()
                .w(px(180.))
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .py_1()
                .rounded_lg()
                .border_2()
                .border_color(cx.theme().link)
                .child(div().text_sm().child(picker.trigger.to_string()))
                .child(
                    Input::new(&picker.filter)
                        .appearance(false)
                        .focus_bordered(false)
                        .min_w_0()
                        .flex_1(),
                )
        })
    }

    /// Delete an adjacent confirmed capsule through the shared tree editor, so
    /// removing a filter also removes its Boolean separator and empty groups.
    /// Plain text, selections, nested inputs and IME retain native deletion.
    pub(super) fn backspace_scope_capsule(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.helper
            || self.candidates.is_some()
            || self.optional_field_value.is_some()
            || self.condition_editor.is_some()
            || self.tag_manager.is_some()
            || self.input_is_composing(window, cx)
        {
            return false;
        }
        let input = self.search.read(cx);
        let selection = input.selected_range();
        if !input.focus_handle(cx).is_focused(window) || !selection.is_empty() {
            return false;
        }
        if selection.start == 0 {
            if self.helper_scope.take().is_none() {
                return false;
            }
            self.pending_trigger = None;
            self.pending_query_projection = None;
            self.error = None;
            self.schedule_search(cx);
            self.resize_helper(window, cx);
            return true;
        }
        let content = input.content();
        let caret = selection.start;
        let at_tail = caret == content.text().len();
        let Some(token) = content.tokens().iter().rev().find(|token| {
            token.range().end == caret
                || (at_tail
                    && token.range().end < caret
                    && content
                        .text()
                        .get(token.range().end..caret)
                        .is_some_and(|suffix| {
                            suffix
                                .chars()
                                .all(|character| character.is_whitespace() || character == ')')
                        }))
        }) else {
            return false;
        };
        let range = token.range();
        // Reconstruct bindings from surviving native annotations before editing.
        // Invalid handwritten text is left to the native text editor.
        self.rebind_native_tokens(cx);
        let analysis = service::analyze_query(content.text().to_string());
        if analysis.status != QueryStatus::Valid {
            return false;
        }
        let document = self
            .query_document
            .clone()
            .or(analysis.document)
            .unwrap_or_default();
        let Some(target) = document.nodes.iter().position(|node| {
            matches!(node.kind, QueryNodeKind::Scope | QueryNodeKind::Not)
                && node.start as usize == range.start
                && node.end as usize == range.end
        }) else {
            return false;
        };
        let edited = service::edit_query(QueryEdit {
            document,
            action: QueryEditAction::Remove,
            target: Some(target as u32),
            operand: None,
        });
        if let Some(error) = Self::analysis_error(&edited, self.preferences.read(cx)) {
            self.error = Some(error);
            cx.notify();
            return true;
        }
        let previous_document = self.query_document.clone();
        self.query_document = edited.document;
        if !self.install_bound_text(edited.text, window, cx) {
            self.query_document = previous_document;
            cx.notify();
            return true;
        }
        let preceding = content
            .tokens()
            .iter()
            .take_while(|token| token.range().end <= range.start)
            .last()
            .map(|token| token.range().end)
            .unwrap_or(range.start.saturating_sub(1));
        self.search.update(cx, |input, cx| {
            let cursor = if at_tail {
                input.value().len()
            } else {
                preceding.min(input.value().len())
            };
            input.set_selected_range(cursor..cursor, cx);
        });
        self.pending_trigger = None;
        self.pending_query_projection = None;
        self.error = None;
        self.schedule_search(cx);
        self.resize_helper(window, cx);
        true
    }

    /// One current keyword scope; replacing or removing it never nests extra scopes
    /// into the saved base document. An empty base stays idle.
    pub(super) fn helper_scope_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().when_some(self.helper_scope.clone(), |view, field| {
            let label = self.candidate_label('$', &field, cx);
            view.flex_none()
                .max_w(px(150.))
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .rounded_lg()
                .bg(cx.theme().muted)
                .child(
                    Button::new("helper-field-scope")
                        .disabled(self.optional_field_value.is_some())
                        .ghost()
                        .small()
                        .label(label)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.start_candidate_picker(
                                '$',
                                this.search.read(cx).value().to_string(),
                                None,
                                window,
                                cx,
                            )
                        })),
                )
                .child(
                    Button::new("remove-helper-scope")
                        .disabled(self.optional_field_value.is_some())
                        .ghost()
                        .xsmall()
                        .icon(icon("x"))
                        .accessibility_label(self.text("移除", cx))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.helper_scope = None;
                            this.focus_query_tail(window, cx);
                            this.schedule_search(cx);
                        })),
                )
        })
    }
    fn open_optional_field(
        &mut self,
        field: String,
        previous_scope: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placeholder = self.text("字段值（可选）", cx);
        let input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        let expected = input.entity_id();
        let subscription = cx.subscribe_in(&input, window, move |this, _, event, window, cx| {
            if this
                .optional_field_value
                .as_ref()
                .is_none_or(|value| value.input.entity_id() != expected)
            {
                return;
            }
            if matches!(event, InputEvent::PressEnter { .. }) {
                this.finish_optional_field(false, window, cx);
            }
        });
        let ime_subscription = cx.observe_in(&input, window, |this, _, window, cx| {
            this.synchronize_ime(window, cx)
        });
        self.optional_field_value = Some(OptionalFieldValue {
            field,
            previous_scope,
            input: input.clone(),
            _subscription: subscription,
            _ime_subscription: ime_subscription,
        });
        input.update(cx, |input, cx| input.focus(window, cx));
        self.schedule_search(cx);
        self.resize_helper(window, cx);
    }

    /// Empty Enter or Escape keeps the selected scope. Nonempty Enter appends an
    /// explicit field condition and restores the prior default keyword scope.
    pub(super) fn finish_optional_field(
        &mut self,
        skip: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.input_is_composing(window, cx) {
            return;
        }
        let Some(value) = self.optional_field_value.as_ref() else {
            return;
        };
        let text = value.input.read(cx).value().to_string();
        if !skip && !text.trim().is_empty() {
            let condition = service::build_condition(QueryCondition {
                field_name: value.field.clone(),
                text_value: text,
                phrase: false,
                exclude: false,
                tag_id: None,
            });
            if let Some(error) = Self::analysis_error(&condition, self.preferences.read(cx)) {
                self.error = Some(error);
                cx.notify();
                return;
            }
            let base = self.query_document.clone().unwrap_or_else(|| {
                service::analyze_query(self.search.read(cx).value().to_string())
                    .document
                    .unwrap_or_default()
            });
            let analysis = service::edit_query(QueryEdit {
                document: base,
                action: QueryEditAction::AppendAnd,
                target: None,
                operand: condition.document,
            });
            if let Some(error) = Self::analysis_error(&analysis, self.preferences.read(cx)) {
                self.error = Some(error);
                cx.notify();
                return;
            }
            self.helper_scope = value.previous_scope.clone();
            self.query_document = analysis.document;
            if !self.install_bound_text(analysis.text, window, cx) {
                cx.notify();
                return;
            }
        }
        self.optional_field_value = None;
        self.error = None;
        self.focus_query_tail(window, cx);
        self.schedule_search(cx);
        self.resize_helper(window, cx);
    }

    /// Render only the optional field-value step; the first inline field remains
    /// exclusively a candidate-name filter.
    pub(super) fn optional_field_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().when_some(self.optional_field_value.as_ref(), |view, value| {
            view.flex_none()
                .w(px(230.))
                .px_2()
                .py_1()
                .border_2()
                .rounded_lg()
                .border_color(cx.theme().link)
                .child(
                    Input::new(&value.input)
                        .appearance(false)
                        .focus_bordered(false),
                )
        })
    }
}
