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

enum DesktopSetting {
    Tray(bool),
    Shortcut(String),
}

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
    shortcut_focus: FocusHandle,
    recording_shortcut: bool,
    desktop_busy: bool,
    desktop_error: Option<String>,
    _subscriptions: Vec<Subscription>,
}
impl Settings {
    pub fn new(
        preferences: Entity<Preferences>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let p = preferences.read(cx).clone();
        let shortcut_focus = cx.focus_handle();
        let focus_out = cx.listener(|this: &mut Self, _, _, cx| {
            this.recording_shortcut = false;
            cx.notify();
        });
        let focus_subscription =
            window.on_focus_out(&shortcut_focus, cx, move |event, window, cx| {
                focus_out(&event, window, cx)
            });
        let activation_subscription =
            cx.observe_window_activation(window, |this: &mut Self, window, cx| {
                if !window.is_window_active() {
                    this.recording_shortcut = false;
                    cx.notify();
                }
            });
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
            focus_subscription,
            activation_subscription,
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
            shortcut_focus,
            recording_shortcut: false,
            desktop_busy: false,
            desktop_error: None,
            _subscriptions: subscriptions,
        };
        this.sync_choices(window, cx);
        this
    }
    pub fn capture_current_shortcut(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.recording_shortcut {
            return false;
        }
        self.recording_shortcut = false;
        let value = self.preferences.read(cx).helper_shortcut.clone();
        self.set_desktop(DesktopSetting::Shortcut(value), cx);
        true
    }
    fn set_desktop(&mut self, setting: DesktopSetting, cx: &mut Context<Self>) {
        if self.desktop_busy {
            return;
        }
        #[cfg(target_os = "linux")]
        {
            let Some(desktop) = cx.global::<crate::application::Services>().desktop.clone() else {
                self.desktop_error = Some("error.desktopUnavailable".into());
                cx.notify();
                return;
            };
            let p = self.preferences.read(cx);
            let previous_tray = p.tray_icon_enabled;
            let preferences = self.preferences.clone();
            self.desktop_busy = true;
            self.desktop_error = None;
            cx.notify();
            cx.spawn(async move |this, cx| {
                let result = match &setting {
                    DesktopSetting::Tray(enabled) => {
                        desktop.set_tray_enabled(*enabled).await.map(|_| None)
                    }
                    DesktopSetting::Shortcut(value) => {
                        desktop.prepare_shortcut(value.clone()).await.map(Some)
                    }
                };
                let applied = result.is_ok();
                let mut prepared = None;
                let mut result = result.and_then(|change| {
                    prepared = change;
                    if desktop.wayland && matches!(setting, DesktopSetting::Shortcut(_)) {
                        return Ok(());
                    }
                    preferences
                        .update(cx, |p, cx| {
                            let before = p.clone();
                            match &setting {
                                DesktopSetting::Tray(enabled) => p.tray_icon_enabled = *enabled,
                                DesktopSetting::Shortcut(_) => {
                                    if let Some(change) = &prepared {
                                        p.helper_shortcut = change.value.clone();
                                    }
                                }
                            }
                            if let Err(error) = p.save() {
                                *p = before;
                                return Err(error);
                            }
                            cx.notify();
                            Ok(())
                        })
                        .map_err(|error| error.to_string())
                });
                if result.is_ok()
                    && let Some(change) = prepared.take()
                {
                    result = change.commit().await;
                }
                drop(prepared);
                if applied
                    && result.is_err()
                    && matches!(setting, DesktopSetting::Tray(_))
                    && let Err(error) = desktop.set_tray_enabled(previous_tray).await
                {
                    result = Err(error);
                }
                let failure = result.as_ref().err().cloned();
                let updated = this.update(cx, |this, cx| {
                    this.desktop_error = result.err();
                    this.desktop_busy = false;
                    cx.notify();
                });
                if updated.is_err()
                    && let Some(error) = failure
                {
                    cx.update(|cx| {
                        crate::application::send(bibcitex_linux::Command::Error(error), cx)
                    });
                }
            })
            .detach();
        }
        #[cfg(not(target_os = "linux"))]
        {
            match setting {
                DesktopSetting::Tray(value) => {
                    let _ = value;
                }
                DesktopSetting::Shortcut(value) => {
                    let _ = value;
                }
            }
            self.desktop_error = Some("error.desktopUnavailable".into());
            cx.notify();
        }
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
        #[cfg(target_os = "linux")]
        let desktop = cx.global::<crate::application::Services>().desktop.as_ref();
        #[cfg(target_os = "linux")]
        let (available, portal, description) = (
            desktop.is_some(),
            desktop.is_some_and(|desktop| desktop.wayland),
            desktop
                .map(|desktop| desktop.shortcut_description())
                .unwrap_or_default(),
        );
        #[cfg(not(target_os = "linux"))]
        let (available, portal, description) = (false, false, String::new());
        let disabled = self.desktop_busy || !available;
        let mut general = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(p.text("显示托盘图标"))
                    .child(
                        Switch::new("tray-icon")
                            .small()
                            .checked(p.tray_icon_enabled)
                            .disabled(disabled)
                            .accessibility_label(p.text("显示托盘图标"))
                            .on_change(cx.listener(|this, enabled, _, cx| {
                                this.set_desktop(DesktopSetting::Tray(*enabled), cx)
                            })),
                    ),
            )
            .child(
                Form::horizontal().label_width(px(125.)).child(
                    Field::new().label(p.text("助手快捷键")).child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("record-shortcut")
                                    .track_focus(&self.shortcut_focus)
                                    .small()
                                    .min_w(px(150.))
                                    .label(if self.recording_shortcut {
                                        p.text("请按快捷键…")
                                    } else {
                                        let value = if portal {
                                            &description
                                        } else {
                                            &p.helper_shortcut
                                        };
                                        bibcitex_linux::shortcut::Shortcut::parse(value)
                                            .map(|s| s.display_value().into())
                                            .unwrap_or_else(|_| {
                                                if value.is_empty() {
                                                    p.text("未设置")
                                                } else {
                                                    value.clone().into()
                                                }
                                            })
                                    })
                                    .accessibility_label(p.text("助手快捷键"))
                                    .tooltip(p.text("录制快捷键"))
                                    .disabled(disabled)
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        if portal {
                                            this.set_desktop(
                                                DesktopSetting::Shortcut(String::new()),
                                                cx,
                                            );
                                        } else {
                                            this.recording_shortcut = !this.recording_shortcut;
                                            this.desktop_error = None;
                                            this.shortcut_focus.focus(window, cx);
                                            cx.notify();
                                        }
                                    })),
                            )
                            .when(!portal, |row| {
                                row.child(
                                    Button::new("reset-shortcut")
                                        .small()
                                        .label(p.text("恢复默认"))
                                        .disabled(disabled)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.recording_shortcut = false;
                                            this.set_desktop(
                                                DesktopSetting::Shortcut(
                                                    bibcitex_linux::shortcut::DEFAULT_SHORTCUT
                                                        .into(),
                                                ),
                                                cx,
                                            );
                                        })),
                                )
                            }),
                    ),
                ),
            );
        if let Some(error) = &self.desktop_error {
            general = general.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(p.text(error)),
            );
        }
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
            .child(section(p.text("通用"), general, cx))
            .child(section(
                p.text("外观"),
                Form::horizontal()
                    .label_width(px(150.))
                    .child(
                        Field::new().label(p.text("主题")).child(
                            Select::new(&self.appearance)
                                .small()
                                .icon(icon("chevronDown"))
                                .accessibility_label(p.text("主题")),
                        ),
                    )
                    .child(
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
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if !this.recording_shortcut {
                    return;
                }
                let key = event.keystroke.key.as_str();
                if key == "tab" {
                    this.recording_shortcut = false;
                    cx.notify();
                    return;
                }
                cx.stop_propagation();
                if key == "escape" {
                    this.recording_shortcut = false;
                    cx.notify();
                    return;
                }
                if event.is_held
                    || matches!(
                        key,
                        "control" | "ctrl" | "alt" | "shift" | "super" | "meta" | "cmd"
                    )
                {
                    return;
                }
                let m = event.keystroke.modifiers;
                let modifiers = (u16::from(m.control) * 4)
                    | (u16::from(m.alt) * 8)
                    | u16::from(m.shift)
                    | (u16::from(m.platform) * 64);
                match bibcitex_linux::shortcut::Shortcut::from_key(key, modifiers) {
                    Ok(shortcut) => {
                        this.recording_shortcut = false;
                        this.set_desktop(DesktopSetting::Shortcut(shortcut.value), cx);
                    }
                    Err(error) => {
                        this.desktop_error = Some(error);
                        cx.notify();
                    }
                }
            }))
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
