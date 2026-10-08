mod application;
mod brand;
mod editor;
mod icons;
mod math;
mod metadata;
mod preferences;
mod settings;
mod updater;
#[cfg(target_os = "linux")]
mod wayland_activation;
mod workbench;

use bibcitex_linux::{Command, instance};
use gpui_kit::*;
use preferences::Preferences;
use std::borrow::Cow;

gpui_kit::actions!(bibcitex, [Quit]);

struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = icons::load(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        if path == "bibcitex-pin.svg" {
            return Ok(Some(Cow::Borrowed(include_bytes!(
                "../../public/icons/pin.svg"
            ))));
        }
        if path.starts_with("math/") {
            return Ok(math::asset(path));
        }
        if path == "bibcitex-logo.png" {
            Ok(Some(Cow::Borrowed(include_bytes!(
                "../../public/favicon.png"
            ))))
        } else {
            assets::Assets.load(path)
        }
    }
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        assets::Assets.list(path)
    }
}

pub fn window_options(title: &str, width: f32, height: f32) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(100.), px(100.)),
            size: size(px(width), px(height)),
        })),
        titlebar: Some(TitlebarOptions {
            title: Some(title.to_owned().into()),
            ..Default::default()
        }),
        window_min_size: Some(size(
            px(if width > 800. { 800. } else { width }),
            px(if width > 800. { 520. } else { height }),
        )),
        #[cfg(target_os = "linux")]
        icon: image::load_from_memory_with_format(
            include_bytes!("../../assets/app-icons/128x128.png"),
            image::ImageFormat::Png,
        )
        .ok()
        .map(|icon| std::sync::Arc::new(icon.to_rgba8())),
        app_id: Some("io.github.tangxiangong.bibcitex".into()),
        ..Default::default()
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--version") => {
            println!("BibCiTeX {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("--help") => {
            println!("Usage: bibcitex [--helper | --tray | --settings | --quit | --version]");
            return;
        }
        Some(argument) if !matches!(argument, "--helper" | "--tray" | "--settings" | "--quit") => {
            eprintln!("Unknown argument: {argument}");
            std::process::exit(2);
        }
        _ => {}
    }
    let preferences = match Preferences::load() {
        Ok(p) => p,
        Err(error) => {
            eprintln!("Could not read BibCiTeX preferences: {error}");
            std::process::exit(1);
        }
    };
    let command = match std::env::args().nth(1).as_deref() {
        Some("--helper") => Command::Helper,
        Some("--tray") => Command::Tray,
        Some("--settings") => Command::Settings,
        Some("--quit") => Command::Quit,
        _ => Command::Main,
    };
    let command = match (command, std::env::var("XDG_ACTIVATION_TOKEN").ok()) {
        (Command::Main, Some(token)) => Command::MainToken(token),
        (Command::Helper, Some(token)) => Command::HelperToken(token),
        (command, _) => command,
    };
    let (events, receiver) = async_channel::bounded(64);
    let _instance = match instance::acquire(&command, events.clone()) {
        Ok(Some(instance)) => instance,
        Ok(None) => return,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    if matches!(command, Command::Quit) {
        return;
    }
    #[cfg(target_os = "linux")]
    let desktop =
        match bibcitex_linux::desktop::Desktop::start(events.clone(), preferences.chinese()) {
            Ok(desktop) => Some(desktop),
            Err(error) => {
                let _ = events.try_send(Command::Error(error));
                None
            }
        };
    application().with_assets(Assets).run(move |cx| {
        init(cx);
        cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.set_app_identity("io.github.tangxiangong.bibcitex", "BibCiTeX");
        preferences.apply_theme(cx);
        let preferences = cx.new(|_| preferences);
        cx.set_quit_mode(QuitMode::Explicit);
        let updater = updater::Updater::start(preferences.clone(), cx);
        cx.set_global(application::Services {
            updater,
            events: events.clone(),
            #[cfg(target_os = "linux")]
            desktop,
        });
        application::listen(preferences, receiver, cx);
        let _ = events.try_send(command);
    });
}
