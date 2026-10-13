//! X11 registration for the Linux client's global helper shortcut.
use crate::Command;
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
pub fn start(events: async_channel::Sender<Command>) -> Result<(), String> {
    let (connection, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let root = connection.setup().roots[screen].root;
    let hotkey = key(&connection, u32::from(b'k'))?;
    for locks in [
        ModMask::default(),
        ModMask::LOCK,
        ModMask::M2,
        ModMask::LOCK | ModMask::M2,
    ] {
        let result = connection
            .grab_key(
                false,
                root,
                ModMask::CONTROL | ModMask::SHIFT | locks,
                hotkey,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .map_err(|e| e.to_string())
            .and_then(|cookie| cookie.check().map_err(|e| e.to_string()));
        if let Err(error) = result {
            let _ = events.try_send(Command::Error(format!("Ctrl+Shift+K: {error}")));
        }
    }
    connection.flush().map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        loop {
            match connection.poll_for_event() {
                Ok(Some(Event::KeyRelease(event))) if event.detail == hotkey => {
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
