use crate::{
    preferences::Preferences,
    settings::{About, Settings},
    workbench::Workbench,
};
use bibcitex_linux::Command;
use gpui_kit::*;

pub struct Services {
    pub updater: Entity<crate::updater::Updater>,
    pub events: async_channel::Sender<Command>,
    #[cfg(target_os = "linux")]
    pub desktop: Option<bibcitex_linux::desktop::Desktop>,
}
impl Global for Services {}
pub fn send(command: Command, cx: &App) {
    let _ = cx.global::<Services>().events.try_send(command);
}

#[derive(Default)]
struct Windows {
    main: Option<(AnyWindowHandle, Entity<Workbench>)>,
    helper: Option<(AnyWindowHandle, Entity<Workbench>)>,
    tray: Option<(AnyWindowHandle, Entity<Workbench>)>,
    settings: Option<AnyWindowHandle>,
    about: Option<AnyWindowHandle>,
}

pub fn listen(
    preferences: Entity<Preferences>,
    receiver: async_channel::Receiver<Command>,
    cx: &App,
) {
    cx.spawn(async move |cx| {
        let mut windows = Windows::default();
        while let Ok(command) = receiver.recv().await {
            cx.update(|cx| windows.handle(command, &preferences, cx));
        }
    })
    .detach();
}

impl Windows {
    fn handle(&mut self, command: Command, preferences: &Entity<Preferences>, cx: &mut App) {
        match command {
            Command::Quit => cx.quit(),
            Command::CheckUpdates => {
                self.handle(Command::Settings, preferences, cx);
                let updater = cx.global::<Services>().updater.clone();
                updater.update(cx, |updater, cx| updater.check(preferences.clone(), cx));
            }
            Command::About => {
                if self.about.is_some_and(|window| {
                    window
                        .update(cx, |_, window, _| window.activate_window())
                        .is_ok()
                }) {
                    return;
                }
                let prefs = preferences.clone();
                match open_window(
                    decorated_options(&prefs.read(cx).text("关于 BibCiTeX"), 400., 340.),
                    cx,
                    |_, cx| cx.new(|cx| About::new(prefs, cx)),
                ) {
                    Ok((window, _)) => self.about = Some(window),
                    Err(error) => self.error(error.to_string(), preferences, cx),
                }
            }
            Command::Settings => {
                if self.settings.is_some_and(|window| {
                    window
                        .update(cx, |_, window, _| window.activate_window())
                        .is_ok()
                }) {
                    return;
                }
                let prefs = preferences.clone();
                match open_window(
                    decorated_options(&prefs.read(cx).text("设置"), 480., 480.),
                    cx,
                    |window, cx| cx.new(|cx| Settings::new(prefs, window, cx)),
                ) {
                    Ok((window, _)) => self.settings = Some(window),
                    Err(error) => self.error(error.to_string(), preferences, cx),
                }
            }
            Command::RefreshLibraries => {
                for (_, view) in [&self.main, &self.helper, &self.tray].into_iter().flatten() {
                    view.update(cx, |view, cx| view.reload(cx));
                }
            }
            Command::Error(error) => self.error(error, preferences, cx),
            Command::PasteFailed { error, key } => {
                self.handle(Command::Helper, preferences, cx);
                if let Some((_, helper)) = &self.helper {
                    helper.update(cx, |helper, cx| helper.paste_failed(error, key, cx));
                }
            }
            Command::TrayAvailable(available) => {
                if !available && cx.windows().is_empty() {
                    self.handle(Command::Main, preferences, cx);
                }
            }
            command => {
                let token = match &command {
                    Command::HelperToken(token) | Command::MainToken(token) => Some(token.clone()),
                    _ => None,
                };
                let helper = matches!(command, Command::Helper | Command::HelperToken(_));
                let tray = matches!(command, Command::Tray | Command::TrayAt { .. });
                let tray_anchor = match command {
                    Command::TrayAt { x, y } => Some(point(px(x as f32), px(y as f32))),
                    _ => None,
                };
                if (helper || tray)
                    && self
                        .helper
                        .as_ref()
                        .is_some_and(|(_, view)| view.read(cx).paste_in_progress())
                {
                    return;
                }
                if helper && let Some((handle, _)) = self.tray.take() {
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                }
                if tray && let Some((handle, _)) = self.helper.take() {
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                }
                let slot = if helper {
                    &mut self.helper
                } else if tray {
                    &mut self.tray
                } else {
                    &mut self.main
                };
                if let Some((handle, entity)) = slot
                    && handle
                        .update(cx, |_, window, cx| {
                            if helper && window.is_window_active() {
                                window.remove_window();
                                return;
                            }
                            #[cfg(target_os = "linux")]
                            if helper && !window.is_window_active() {
                                let external = cx.active_window().is_none();
                                if let Some(error) = cx
                                    .global::<Services>()
                                    .desktop
                                    .as_ref()
                                    .and_then(|desktop| desktop.capture(external).err())
                                {
                                    entity.update(cx, |view, cx| view.report_error(error, cx));
                                }
                            }
                            activate(window, token.as_deref());
                            entity.update(cx, |view, cx| {
                                view.reload(cx);
                                view.focus_search(window, cx);
                            });
                        })
                        .is_ok()
                {
                    return;
                }
                #[cfg(target_os = "linux")]
                let capture_error = if helper {
                    let owned_active = cx.windows().into_iter().any(|handle| {
                        handle
                            .update(cx, |_, window, _| window.is_window_active())
                            .unwrap_or(false)
                    });
                    cx.global::<Services>()
                        .desktop
                        .as_ref()
                        .and_then(|desktop| desktop.capture(!owned_active).err())
                } else {
                    None
                };
                let (width, height) = if helper {
                    (720., 56.)
                } else if tray {
                    (680., 580.)
                } else {
                    (1100., 720.)
                };
                let prefs = preferences.clone();
                let mut options = crate::window_options(
                    if tray { "BibCiTeX Tray" } else { "BibCiTeX" },
                    width,
                    height,
                );
                if !helper && !tray {
                    let chrome = gpui_kit::component::TitleBar::window_options();
                    options.titlebar = chrome.titlebar;
                    options.app_owns_titlebar_drag = chrome.app_owns_titlebar_drag;
                    options.window_decorations = Some(WindowDecorations::Client);
                }
                if helper || tray {
                    options.titlebar = None;
                    options.kind = WindowKind::PopUp;
                    options.is_resizable = false;
                    options.is_minimizable = false;
                    options.window_decorations = Some(WindowDecorations::Client);
                    options.window_background = WindowBackgroundAppearance::Transparent;
                    options.window_min_size =
                        Some(size(px(320.), px(if helper { 56. } else { 300. })));
                    let display = tray_anchor
                        .and_then(|anchor| {
                            cx.displays()
                                .into_iter()
                                .find(|display| display.bounds().contains(&anchor))
                        })
                        .or_else(|| cx.primary_display());
                    if let Some(display) = display {
                        options.display_id = Some(display.id());
                        let bounds = display.bounds();
                        let width = px(width).min((bounds.size.width - px(40.)).max(px(320.)));
                        let height = px(height).min((bounds.size.height - px(16.)).max(px(56.)));
                        options.window_bounds = Some(WindowBounds::Windowed(Bounds {
                            origin: point(
                                if helper {
                                    bounds.origin.x + (bounds.size.width - width) / 2.
                                } else {
                                    bounds.origin.x + bounds.size.width - width - px(8.)
                                },
                                if helper {
                                    bounds.origin.y + bounds.size.height / 6.
                                } else {
                                    bounds.origin.y + px(6.)
                                },
                            ),
                            size: size(width, height),
                        }));
                        if let Some(anchor) = tray_anchor {
                            let left = bounds.origin.x + px(8.);
                            let top = bounds.origin.y + px(8.);
                            let right = bounds.origin.x + bounds.size.width - width - px(8.);
                            let bottom = bounds.origin.y + bounds.size.height - height - px(8.);
                            let origin = point(
                                (anchor.x - width / 2.).clamp(left, right.max(left)),
                                (if anchor.y < bounds.origin.y + bounds.size.height / 2. {
                                    anchor.y + px(16.)
                                } else {
                                    anchor.y - height - px(16.)
                                })
                                .clamp(top, bottom.max(top)),
                            );
                            options.window_bounds = Some(WindowBounds::Windowed(Bounds {
                                origin,
                                size: size(width, height),
                            }));
                            #[cfg(target_os = "linux")]
                            if crate::wayland_activation::supports_layer_shell() {
                                use gpui_kit::layer_shell::{
                                    Anchor, KeyboardInteractivity, LayerShellOptions,
                                };
                                options.kind = WindowKind::LayerShell(LayerShellOptions {
                                    namespace: "io.github.tangxiangong.bibcitex.tray".into(),
                                    anchor: Anchor::TOP | Anchor::LEFT,
                                    margin: Some((
                                        origin.y - bounds.origin.y,
                                        px(0.),
                                        px(0.),
                                        origin.x - bounds.origin.x,
                                    )),
                                    exclusive_zone: Some(px(-1.)),
                                    keyboard_interactivity: KeyboardInteractivity::Exclusive,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
                match open_window(options, cx, move |window, cx| {
                    cx.new(|cx| {
                        let mut view = if tray {
                            Workbench::new_tray(prefs, window, cx)
                        } else {
                            Workbench::new(prefs, helper, window, cx)
                        };
                        #[cfg(target_os = "linux")]
                        if let Some(error) = capture_error {
                            view.report_error(error, cx);
                        }
                        view.focus_search(window, cx);
                        activate(window, token.as_deref());
                        view
                    })
                }) {
                    Ok(window) => *slot = Some(window),
                    Err(error) => self.error(error.to_string(), preferences, cx),
                }
            }
        }
    }
    fn error(&mut self, error: String, preferences: &Entity<Preferences>, cx: &mut App) {
        if self
            .main
            .as_ref()
            .is_none_or(|(window, _)| window.update(cx, |_, _, _| ()).is_err())
        {
            self.handle(Command::Main, preferences, cx);
        }
        if let Some((_, main)) = &self.main {
            main.update(cx, |main, cx| main.report_error(error, cx));
        }
    }
}

fn activate(window: &Window, token: Option<&str>) {
    #[cfg(target_os = "linux")]
    if let Some(token) = token {
        if let Err(error) = crate::wayland_activation::activate(window, token) {
            eprintln!("Wayland activation: {error}");
            window.activate_window();
        }
        return;
    }
    #[cfg(not(target_os = "linux"))]
    let _ = token;
    window.activate_window();
}

fn decorated_options(title: &str, width: f32, height: f32) -> WindowOptions {
    let mut options = crate::window_options(title, width, height);
    let chrome = gpui_kit::component::TitleBar::window_options();
    options.titlebar = chrome.titlebar;
    options.app_owns_titlebar_drag = chrome.app_owns_titlebar_drag;
    options.window_decorations = Some(WindowDecorations::Client);
    options
}
