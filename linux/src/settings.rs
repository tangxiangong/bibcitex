use crate::{icons::icon, preferences::Preferences};
use gpui_kit::component::{
    ActiveTheme, Disableable, IndexPath, Sizable, TitleBar,
    button::Button,
    form::{Field, Form},
    searchable_list::SearchableListItem,
    select::{Select, SelectEvent, SelectState},
    switch::Switch,
};
use gpui_kit::{prelude::FluentBuilder as _, *};

#[derive(Clone)]
struct Choice {
    value: String,
    label: SharedString,
}
impl SearchableListItem for Choice {
    type Value = String;
    fn title(&self) -> SharedString {
        self.label.clone()
    }
    fn value(&self) -> &String {
        &self.value
    }
}
type Choices = Entity<SelectState<Vec<Choice>>>;

fn citation_family(format: &str) -> &'static str {
    if format == "raw"
        || !bibcitex_service::CITATION_FORMATS
            .iter()
            .any(|(id, _)| *id == format)
    {
        "raw"
    } else if format.starts_with("typst") {
        "typst"
    } else {
        "latex"
    }
}

fn choices(group: &str, prefs: &Preferences) -> Vec<Choice> {
    let values: &[(&str, &str)] = match group {
        "appearance" => &[("system", "跟随系统"), ("light", "浅色"), ("dark", "深色")],
        "language" => &[
            ("system", "跟随系统"),
            ("zh-Hans", "简体中文"),
            ("en", "English"),
        ],
        "citation_family" => &[("raw", "key"), ("latex", "LaTeX"), ("typst", "Typst")],
        "citation_format" => bibcitex_service::CITATION_FORMATS,
        "citation_mode" => &[("merged", "合并"), ("individual", "逐项")],
        _ => &[("stable", "正式版"), ("beta", "Beta"), ("alpha", "Alpha")],
    };
    values
        .iter()
        .copied()
        .filter(|(value, _)| {
            group != "citation_format"
                || (*value != "raw"
                    && citation_family(value) == citation_family(&prefs.citation_format))
        })
        .map(|(value, label)| Choice {
            value: value.into(),
            label: prefs.text(label),
        })
        .collect()
}

pub struct Settings {
    preferences: Entity<Preferences>,
    error: Option<String>,
    updater: Entity<crate::updater::Updater>,
    appearance: Choices,
    language: Choices,
    channel: Choices,
    citation_family: Choices,
    citation_format: Choices,
    citation_mode: Choices,
    _subscriptions: Vec<Subscription>,
}
impl Settings {
    pub fn new(
        preferences: Entity<Preferences>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let p = preferences.read(cx).clone();
        let appearance = cx.new(|cx| {
            SelectState::new(
                choices("appearance", &p),
                Some(IndexPath::new(0)),
                window,
                cx,
            )
        });
        let language = cx.new(|cx| {
            SelectState::new(choices("language", &p), Some(IndexPath::new(0)), window, cx)
        });
        let channel = cx.new(|cx| {
            SelectState::new(choices("channel", &p), Some(IndexPath::new(0)), window, cx)
        });
        let citation_family_state = cx.new(|cx| {
            SelectState::new(
                choices("citation_family", &p),
                Some(IndexPath::new(0)),
                window,
                cx,
            )
        });
        let citation_format =
            cx.new(|cx| SelectState::new(choices("citation_format", &p), None, window, cx));
        let citation_mode = cx.new(|cx| {
            SelectState::new(
                choices("citation_mode", &p),
                Some(IndexPath::new(0)),
                window,
                cx,
            )
        });
        let updater = cx.global::<crate::application::Services>().updater.clone();
        let subscriptions = vec![
            cx.subscribe_in(
                &citation_family_state,
                window,
                |this, _, event, window, cx| {
                    if let SelectEvent::Confirm(Some(value)) = event {
                        if value == citation_family(&this.preferences.read(cx).citation_format) {
                            return;
                        }
                        let command = match value.as_str() {
                            "latex" => "cite",
                            "typst" => "typst",
                            _ => "raw",
                        };
                        this.set_citation(true, command, window, cx);
                    }
                },
            ),
            cx.subscribe_in(&citation_format, window, |this, _, event, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.set_citation(true, value, window, cx);
                }
            }),
            cx.subscribe_in(&citation_mode, window, |this, _, event, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.set_citation(false, value, window, cx);
                }
            }),
            cx.observe(&updater, |_, _, cx| cx.notify()),
            cx.subscribe_in(&appearance, window, |this, _, event, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.set(true, value, window, cx);
                }
            }),
            cx.subscribe_in(&language, window, |this, _, event, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.set(false, value, window, cx);
                }
            }),
            cx.subscribe_in(&channel, window, |this, _, event, _, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.preferences.update(cx, |p, cx| {
                        p.update_channel = value.clone();
                        this.error = p.save().err();
                        cx.notify();
                    });
                    this.updater.update(cx, |updater, cx| {
                        updater.release = None;
                        updater.checked = false;
                        cx.notify();
                    });
                }
            }),
        ];
        let mut this = Self {
            preferences,
            error: None,
            updater,
            appearance,
            language,
            channel,
            citation_family: citation_family_state,
            citation_format,
            citation_mode,
            _subscriptions: subscriptions,
        };
        this.sync_choices(window, cx);
        this
    }
    fn sync_choices(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let p = self.preferences.read(cx).clone();
        let family = citation_family(&p.citation_format).to_owned();
        for (state, group, value) in [
            (&self.appearance, "appearance", &p.appearance),
            (&self.language, "language", &p.language),
            (&self.channel, "channel", &p.update_channel),
            (&self.citation_family, "citation_family", &family),
            (&self.citation_format, "citation_format", &p.citation_format),
            (&self.citation_mode, "citation_mode", &p.citation_mode),
        ] {
            state.update(cx, |state, cx| {
                state.set_items(choices(group, &p), window, cx);
                state.set_selected_value(value, window, cx);
            });
        }
    }
    fn set_citation(
        &mut self,
        format: bool,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.preferences.update(cx, |p, cx| {
            if format {
                p.citation_format = value.into();
            } else {
                p.citation_mode = value.into();
            }
            self.error = p.save().err();
            cx.notify();
        });
        self.sync_choices(window, cx);
        cx.refresh_windows();
    }
    fn set(&mut self, appearance: bool, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.preferences.update(cx, |p, cx| {
            if appearance {
                p.appearance = value.into();
            } else {
                p.language = value.into();
            }
            p.apply_theme(cx);
            #[cfg(target_os = "linux")]
            if let Some(desktop) = &cx.global::<crate::application::Services>().desktop {
                desktop.language(p.chinese());
            }
            self.error = p.save().err();
            cx.notify();
        });
        self.sync_choices(window, cx);
        cx.refresh_windows();
    }
}
fn section(title: SharedString, content: impl IntoElement, cx: &App) -> impl IntoElement {
    div()
        .id(title.clone())
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .px_1()
                .text_size(px(13.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(
            div()
                .p_3()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .child(content),
        )
}
impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.preferences.read(cx).clone();
        window.set_window_title(&p.text("设置"));
        let managed = std::env::current_exe()
            .is_ok_and(|path| bibcitex_linux::updates::managed_install(&path));
        let updater = self.updater.read(cx);
        let busy = updater.busy;
        let installed = updater.installed;
        let available = updater.release.as_ref().map(|r| r.manifest.version.clone());
        let update_error = updater.error.clone();
        let up_to_date = updater.checked && available.is_none() && update_error.is_none();
        let mut updates = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                Form::horizontal()
                    .label_width(px(150.))
                    .child(
                        Field::new()
                            .label(p.text("当前版本"))
                            .child(div().w_full().text_right().child(env!("CARGO_PKG_VERSION"))),
                    )
                    .child(
                        Field::new().label(p.text("更新通道")).child(
                            Select::new(&self.channel)
                                .small()
                                .icon(icon("chevronDown"))
                                .accessibility_label(p.text("更新通道")),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(p.text("自动下载并安装更新"))
                    .child(
                        Switch::new("automatic-updates")
                            .small()
                            .checked(p.automatic_updates && !managed)
                            .disabled(managed)
                            .accessibility_label(p.text("自动下载并安装更新"))
                            .on_change(cx.listener(|this, value, _, cx| {
                                this.preferences.update(cx, |p, cx| {
                                    p.automatic_updates = *value;
                                    this.error = p.save().err();
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                div().flex().justify_end().child(
                    Button::new("check-updates")
                        .small()
                        .label(p.text("检查更新"))
                        .disabled(busy || installed)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.updater.update(cx, |updater, cx| {
                                updater.check(this.preferences.clone(), cx)
                            })
                        })),
                ),
            );
        if let Some(version) = available {
            updates = updates.child(
                Button::new("install-update")
                    .label(format!("{} {version}", p.text("下载并安装")))
                    .disabled(busy || installed)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.updater.update(cx, |updater, cx| updater.install(cx))
                    })),
            );
        }
        if installed {
            updates = updates.child(
                Button::new("restart")
                    .label(p.text("重启以完成更新"))
                    .on_click(|_, _, cx| cx.restart()),
            );
        }
        if up_to_date {
            updates = updates.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(p.text("当前已是最新版本")),
            );
        }
        if let Some(error) = update_error {
            updates = updates.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(p.text(&error)),
            );
        }
        let content = div()
            .id("settings-content")
            .overflow_y_scroll()
            .flex_1()
            .min_h_0()
            .p_5()
            .flex()
            .flex_col()
            .gap_5()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().foreground)
            .text_size(px(13.))
            .child(section(
                p.text("外观"),
                Form::horizontal().label_width(px(150.)).child(
                    Field::new().label(p.text("主题")).child(
                        Select::new(&self.appearance)
                            .small()
                            .icon(icon("chevronDown"))
                            .accessibility_label(p.text("主题")),
                    ),
                ),
                cx,
            ))
            .child(section(
                p.text("语言"),
                Form::horizontal().label_width(px(150.)).child(
                    Field::new().label(p.text("语言")).child(
                        Select::new(&self.language)
                            .small()
                            .icon(icon("chevronDown"))
                            .accessibility_label(p.text("语言")),
                    ),
                ),
                cx,
            ))
            .child(section(
                p.text("引用"),
                Form::horizontal()
                    .label_width(px(150.))
                    .child(
                        Field::new().label(p.text("引用格式")).child(
                            Select::new(&self.citation_family)
                                .small()
                                .icon(icon("chevronDown"))
                                .accessibility_label(p.text("引用格式")),
                        ),
                    )
                    .when(citation_family(&p.citation_format) != "raw", |form| {
                        form.child(
                            Field::new().label(p.text("引用指令")).child(
                                Select::new(&self.citation_format)
                                    .small()
                                    .icon(icon("chevronDown"))
                                    .accessibility_label(p.text("引用指令")),
                            ),
                        )
                    })
                    .child(
                        Field::new().label(p.text("多选输出方式")).child(
                            Select::new(&self.citation_mode)
                                .small()
                                .icon(icon("chevronDown"))
                                .disabled(citation_family(&p.citation_format) == "raw")
                                .accessibility_label(p.text("多选输出方式")),
                        ),
                    ),
                cx,
            ))
            .child(section(p.text("更新"), updates, cx))
            .when_some(self.error.clone(), |body, error| {
                body.child(div().text_color(cx.theme().danger).child(error))
            });
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().foreground)
            .child(TitleBar::new().flex_none().child(p.text("设置")))
            .child(content)
    }
}

pub struct About {
    preferences: Entity<Preferences>,
}

impl About {
    pub fn new(preferences: Entity<Preferences>, cx: &mut Context<Self>) -> Self {
        cx.observe(&preferences, |_, _, cx| cx.notify()).detach();
        Self { preferences }
    }
}

impl Render for About {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.preferences.read(cx).text("关于 BibCiTeX");
        window.set_window_title(&title);
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(TitleBar::new().flex_none().child(title))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .p_7()
                    .child(crate::brand::logo(96.))
                    .child(div().text_size(px(24.)).child("BibCiTeX"))
                    .child(div().text_size(px(13.)).child(env!("CARGO_PKG_VERSION")))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child("MIT OR Apache-2.0"),
                    ),
            )
    }
}
