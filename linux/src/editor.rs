use crate::{icons::icon, preferences::Preferences, workbench::Workbench};
use bibcitex_service::{self as service, LibraryRecord};
use gpui_kit::component::{
    ActiveTheme, Disableable,
    button::{Button, ButtonVariants},
    form::{Field, Form},
    input::{self, Input, InputEvent, InputState, Textarea, TextareaState},
};
use gpui_kit::{prelude::FluentBuilder as _, *};

pub struct LibraryEditor {
    library: Option<LibraryRecord>,
    preferences: Entity<Preferences>,
    owner: WeakEntity<Workbench>,
    name: Entity<InputState>,
    path: Entity<InputState>,
    description: Entity<TextareaState>,
    _subscriptions: Vec<Subscription>,
    saving: bool,
    error: Option<String>,
}

impl LibraryEditor {
    pub fn new(
        library: Option<LibraryRecord>,
        preferences: Entity<Preferences>,
        owner: WeakEntity<Workbench>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name = cx.new(|cx| {
            InputState::new(window, cx).placeholder(preferences.read(cx).text("为文献库起一个名字"))
        });
        let path = cx.new(|cx| {
            InputState::new(window, cx).placeholder(preferences.read(cx).text("尚未选择文件"))
        });
        let description = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder(preferences.read(cx).text("简单描述一下这个文献库..."))
        });
        if let Some(library) = &library {
            name.update(cx, |input, cx| {
                input.set_value(library.name.clone(), window, cx)
            });
            path.update(cx, |input, cx| {
                input.set_value(library.path.clone(), window, cx)
            });
            description.update(cx, |input, cx| {
                input.set_value(library.description.clone().unwrap_or_default(), window, cx)
            });
        }
        name.update(cx, |input, cx| input.focus(window, cx));
        let submit = cx.subscribe_in(&name, window, |this, _, event, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                this.save(window, cx);
            } else if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        Self {
            _subscriptions: vec![submit],
            library,
            preferences,
            owner,
            name,
            path,
            description,
            saving: false,
            error: None,
        }
    }

    fn choose_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(self.preferences.read(cx).text("选择文件")),
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = prompt.await;
            let _ = this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(Ok(Some(paths))) => {
                        if let Some(path) = paths.first() {
                            if !path
                                .extension()
                                .is_some_and(|e| e.eq_ignore_ascii_case("bib"))
                            {
                                this.error = Some(
                                    this.preferences
                                        .read(cx)
                                        .text("Invalid bibliography path")
                                        .to_string(),
                                );
                            } else {
                                if this.name.read(cx).value().trim().is_empty() {
                                    let name = path
                                        .file_stem()
                                        .map(|s| s.to_string_lossy().into_owned())
                                        .unwrap_or_default();
                                    this.name
                                        .update(cx, |input, cx| input.set_value(name, window, cx));
                                }
                                this.path.update(cx, |input, cx| {
                                    input.set_value(path.to_string_lossy().into_owned(), window, cx)
                                });
                            }
                        }
                    }
                    Ok(Err(error)) => this.error = Some(error.to_string()),
                    Err(error) => this.error = Some(error.to_string()),
                    _ => {}
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        let name = self.name.read(cx).value().trim().to_owned();
        let path = self.path.read(cx).value().trim().to_owned();
        let description = self.description.read(cx).value().trim().to_owned();
        if name.is_empty() || path.is_empty() {
            self.error = Some(
                self.preferences
                    .read(cx)
                    .text(if name.is_empty() {
                        "文献库名称不能为空"
                    } else {
                        "尚未选择文件"
                    })
                    .to_string(),
            );
            cx.notify();
            return;
        }
        let old_name = self.library.as_ref().map(|l| l.name.clone());
        self.saving = true;
        self.error = None;
        let job = cx.background_executor().spawn(async move {
            if let Some(old_name) = old_name {
                service::update_library(old_name, name, Some(path), Some(description))
            } else {
                service::add_library(name, path, Some(description))
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = job.await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.saving = false;
                match result {
                    Ok(library) => {
                        let old_name = this.library.as_ref().map(|l| l.name.as_str());
                        let _ = this.owner.update(cx, |owner, cx| {
                            owner.library_saved(old_name, library.name, cx)
                        });
                        window.remove_window();
                    }
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

impl Render for LibraryEditor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.preferences.read(cx).clone();
        let can_save = !self.saving
            && !self.name.read(cx).value().trim().is_empty()
            && !self.path.read(cx).value().trim().is_empty();
        div()
            .id("library-form")
            .overflow_y_scroll()
            .size_full()
            .p(px(28.))
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .capture_action(cx.listener(|this, _: &input::Escape, window, cx| {
                if !this.saving {
                    window.remove_window();
                    cx.stop_propagation();
                }
            }))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap(px(14.))
                    .mb(px(24.))
                    .child(
                        div()
                            .size(px(48.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_lg()
                            .bg(cx.theme().muted)
                            .child(icon("folderAdd").size(px(24.)).text_color(cx.theme().link)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .pt(px(3.))
                            .child(
                                div()
                                    .text_size(px(20.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(p.text(if self.library.is_some() {
                                        "编辑文献库"
                                    } else {
                                        "新增文献库"
                                    })),
                            )
                            .when(self.library.is_none(), |header| {
                                header.child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(p.text("添加一个 .bib 文件到你的工作空间")),
                                )
                            }),
                    ),
            )
            .child(
                Form::vertical()
                    .gap(px(20.))
                    .child(
                        Field::new()
                            .label(p.text("文献库名称"))
                            .child(Input::new(&self.name).disabled(self.saving)),
                    )
                    .child(
                        Field::new().label(p.text("文件路径")).child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .p_4()
                                .rounded_lg()
                                .bg(cx.theme().muted)
                                .border_1()
                                .border_color(cx.theme().border)
                                .child(
                                    icon("fileText")
                                        .size(px(24.))
                                        .text_color(cx.theme().muted_foreground),
                                )
                                .child(
                                    Input::new(&self.path)
                                        .appearance(false)
                                        .readonly(true)
                                        .flex_1(),
                                )
                                .child(
                                    Button::new("choose")
                                        .label(p.text("选择文件"))
                                        .disabled(self.saving)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.choose_file(window, cx)
                                        })),
                                ),
                        ),
                    )
                    .child(
                        Field::new().label(p.text("描述")).child(
                            Textarea::new(&self.description)
                                .h(px(88.))
                                .disabled(self.saving),
                        ),
                    ),
            )
            .when_some(self.error.clone(), |body, error| {
                body.child(
                    div()
                        .mt_3()
                        .text_sm()
                        .text_color(cx.theme().danger)
                        .child(error),
                )
            })
            .child(
                div()
                    .mt(px(24.))
                    .pt_4()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .flex()
                    .justify_end()
                    .gap_3()
                    .child(
                        Button::new("cancel")
                            .label(p.text("取消"))
                            .disabled(self.saving)
                            .on_click(|_, window, _| window.remove_window()),
                    )
                    .child(
                        Button::new("save")
                            .primary()
                            .label(p.text("保存"))
                            .disabled(!can_save)
                            .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    ),
            )
    }
}
