//! Convert external launches into the same commands as tray and shortcut activation.
use crate::Command;
use async_channel::Sender;

fn command(args: &[String], token: Option<String>) -> Result<Command, String> {
    match args {
        [] => Ok(token.map(Command::MainToken).unwrap_or(Command::Main)),
        [arg] if arg == "--helper" => {
            Ok(token.map(Command::HelperToken).unwrap_or(Command::Helper))
        }
        [arg] if arg == "--tray" => Ok(Command::Tray),
        [arg] if arg == "--settings" => Ok(Command::Settings),
        [arg] if arg == "--quit" => Ok(Command::Quit),
        _ => Err("Unknown BibCiTeX launch command".into()),
    }
}

fn initial_command(args: &[String], token: Option<String>) -> Option<Command> {
    // NSH already opens the main window. Only an explicit command or a
    // compositor token requires an additional startup action.
    match command(args, token) {
        Ok(Command::Main) | Err(_) => None,
        Ok(command) => Some(command),
    }
}

pub(crate) fn start(events: &Sender<Command>) -> Result<bool, String> {
    // A development host may have runner-specific arguments. Only the app's
    // explicit launch actions have meaning at startup inside the mixed host.
    let args = std::env::args_os()
        .skip(1)
        .filter_map(|s| s.into_string().ok())
        .collect::<Vec<_>>();
    let initial = initial_command(&args, std::env::var("XDG_ACTIVATION_TOKEN").ok());
    #[cfg(unix)]
    {
        use crate::instance::{Instance, acquire};
        use std::cell::RefCell;
        thread_local! { static INSTANCE: RefCell<Option<Instance>> = const { RefCell::new(None) }; }
        if INSTANCE.with_borrow(Option::is_none) {
            match acquire(initial.as_ref().unwrap_or(&Command::Main), events.clone())? {
                Some(guard) => INSTANCE.with_borrow_mut(|instance| *instance = Some(guard)),
                None => return Ok(false),
            }
        }
    }
    if let Some(initial) = initial {
        events.try_send(initial).map_err(|e| e.to_string())?;
    }
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        let events = events.clone();
        nsh_core::prelude::Owner::detached().run(|| {
            drop(nsh_core::element::spawn(async move {
                loop {
                    let launch = nsh_core::element::next_second_instance().await;
                    let next = command(&launch.args, None).unwrap_or_else(Command::Error);
                    if events.send(next).await.is_err() {
                        break;
                    }
                }
            }));
        });
    }
    Ok(true)
}

/// Apply the compositor's startup notification to the actual GTK toplevel.
#[cfg(target_os = "linux")]
pub(crate) async fn activate_wayland(token: &str, surface: &str) -> Result<(), String> {
    use gtk4::{Window, prelude::*};
    let title = match surface {
        "main" => "BibCiTeX",
        "helper" => "BibCiTeX Helper",
        "tray" => "BibCiTeX Tray",
        _ => return Err("Unknown activation surface".into()),
    };
    if token.is_empty() {
        return Err("Missing compositor activation token".into());
    }
    let token = token.to_owned();
    nsh_core::element::run_on_main_thread(move || {
        let window = Window::list_toplevels()
            .into_iter()
            .filter_map(|widget| widget.downcast::<Window>().ok())
            .find(|win| win.title().as_deref() == Some(title))
            .ok_or_else(|| format!("Window unavailable: {title}"))?;
        window.set_startup_id(&token);
        window.present();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(not(target_os = "linux"))]
pub(crate) async fn activate_wayland(_: &str, _: &str) -> Result<(), String> {
    Err("Wayland activation requires Linux".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_only_dispatches_explicit_commands_or_compositor_tokens() {
        assert!(initial_command(&[], None).is_none());
        assert!(initial_command(&["--dev-runner".into()], None).is_none());
        assert!(matches!(
            initial_command(&["--helper".into()], None),
            Some(Command::Helper)
        ));
        assert!(matches!(
            initial_command(&[], Some("startup-token".into())),
            Some(Command::MainToken(token)) if token == "startup-token"
        ));
    }

    #[test]
    fn activation_commands_preserve_tokens_and_reject_internal_actions() {
        assert!(
            matches!(command(&["--helper".into()], Some("from-compositor".into())),
            Ok(Command::HelperToken(t)) if t == "from-compositor")
        );
        assert!(matches!(command(&[], None), Ok(Command::Main)));
        assert!(matches!(
            command(&["--quit".into()], None),
            Ok(Command::Quit)
        ));
        assert!(command(&["--install".into()], None).is_err());
    }
}
