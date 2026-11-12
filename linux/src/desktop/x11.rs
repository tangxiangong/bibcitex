//! X11 registration for the Linux client's global helper shortcut.
use super::{ShortcutChange, ShortcutDecision, ShortcutRequest};
use crate::{Command, shortcut::Shortcut};
use std::time::Duration;
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{ConnectionExt as _, GrabMode, ModMask},
    },
    rust_connection::RustConnection,
};

fn key(connection: &RustConnection, symbol: u32) -> Result<u8, String> {
    let setup = connection.setup();
    let mapping = connection
        .get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    mapping
        .keysyms
        .chunks(usize::from(mapping.keysyms_per_keycode))
        .position(|symbols| symbols.contains(&symbol))
        .and_then(|offset| setup.min_keycode.checked_add(offset as u8))
        .ok_or_else(|| "Required keyboard key is unavailable".into())
}
const LOCKS: [u16; 4] = [0, 2, 16, 18];

fn ungrab(connection: &RustConnection, root: u32, key: u8, modifiers: u16) {
    for locks in LOCKS {
        let _ = connection.ungrab_key(key, root, ModMask::from(modifiers | locks));
    }
    let _ = connection.flush();
}

fn grab(connection: &RustConnection, root: u32, shortcut: &Shortcut) -> Result<u8, String> {
    let hotkey = key(connection, shortcut.keysym)?;
    let mut grabbed = Vec::new();
    for locks in LOCKS {
        let modifiers = ModMask::from(shortcut.modifiers | locks);
        let result = connection
            .grab_key(
                false,
                root,
                modifiers,
                hotkey,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .map_err(|e| e.to_string())
            .and_then(|cookie| cookie.check().map_err(|e| e.to_string()));
        if let Err(error) = result {
            for modifiers in grabbed {
                let _ = connection.ungrab_key(hotkey, root, modifiers);
            }
            let _ = connection.flush();
            return Err(format!("{}: {error}", shortcut.value));
        }
        grabbed.push(modifiers);
    }
    connection.flush().map_err(|e| e.to_string())?;
    Ok(hotkey)
}

pub(super) fn start(
    events: async_channel::Sender<Command>,
    shortcut: &str,
    requests: async_channel::Receiver<ShortcutRequest>,
) -> Result<(), String> {
    let (connection, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let root = connection.setup().roots[screen].root;
    let initial = Shortcut::parse(shortcut)
        .and_then(|shortcut| grab(&connection, root, &shortcut).map(|key| (shortcut, key)));
    let mut current = match initial {
        Ok(shortcut) => Some(shortcut),
        Err(error) => {
            let _ = events.try_send(Command::Error(error));
            None
        }
    };
    std::thread::spawn(move || {
        let mut pending: Option<(Shortcut, u8, async_channel::Receiver<ShortcutDecision>)> = None;
        loop {
            if let Some((shortcut, hotkey, decision)) = pending.take() {
                match decision.try_recv() {
                    Ok(ShortcutDecision::Commit(response)) => {
                        if let Some((previous, key)) = current.take() {
                            ungrab(&connection, root, key, previous.modifiers);
                        }
                        current = Some((shortcut, hotkey));
                        let _ = response.try_send(());
                    }
                    Err(async_channel::TryRecvError::Empty) => {
                        pending = Some((shortcut, hotkey, decision))
                    }
                    _ => ungrab(&connection, root, hotkey, shortcut.modifiers),
                }
            }
            while let Ok(request) = requests.try_recv() {
                let result = Shortcut::parse(&request.value).and_then(|shortcut| {
                    if pending.is_some() {
                        return Err("error.desktopBusy".into());
                    }
                    if current
                        .as_ref()
                        .is_some_and(|(active, _)| *active == shortcut)
                    {
                        return Ok(ShortcutChange {
                            value: shortcut.value,
                            decision: None,
                        });
                    }
                    let hotkey = grab(&connection, root, &shortcut)?;
                    let value = shortcut.value.clone();
                    let (decision, receiver) = async_channel::bounded(1);
                    pending = Some((shortcut, hotkey, receiver));
                    Ok(ShortcutChange {
                        value,
                        decision: Some(decision),
                    })
                });
                let _ = request.response.try_send(result);
            }
            if requests.is_closed() {
                break;
            }
            match connection.poll_for_event() {
                Ok(Some(Event::KeyRelease(event)))
                    if current
                        .as_ref()
                        .is_some_and(|(_, key)| event.detail == *key) =>
                {
                    let _ = events.try_send(Command::Helper);
                }
                Ok(_) => {}
                Err(error) => {
                    let _ = events.try_send(Command::Error(error.to_string()));
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    Ok(())
}
