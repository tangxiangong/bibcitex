use crate::Command;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use x11rb::{
    CURRENT_TIME,
    connection::Connection,
    protocol::{
        Event,
        xproto::{
            self, AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, GrabMode, ModMask,
        },
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};

#[derive(Default, Clone, Copy)]
pub struct Target {
    recent: Option<(u32, u32)>,
    captured: Option<(u32, u32)>,
}
fn atom(connection: &RustConnection, name: &[u8]) -> Result<u32, String> {
    connection
        .intern_atom(false, name)
        .map_err(|e| e.to_string())?
        .reply()
        .map(|r| r.atom)
        .map_err(|e| e.to_string())
}
fn property(
    connection: &RustConnection,
    window: u32,
    name: &[u8],
    kind: AtomEnum,
) -> Result<u32, String> {
    connection
        .get_property(false, window, atom(connection, name)?, kind, 0, 1)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?
        .value32()
        .and_then(|mut v| v.next())
        .ok_or_else(|| "Window identity is unavailable".into())
}
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
pub fn start(events: async_channel::Sender<Command>) -> Result<Arc<Mutex<Target>>, String> {
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
    let target = Arc::new(Mutex::new(Target::default()));
    let observed = target.clone();
    std::thread::spawn(move || {
        loop {
            if let Ok(window) = property(&connection, root, b"_NET_ACTIVE_WINDOW", AtomEnum::WINDOW)
                && let Ok(pid) = property(&connection, window, b"_NET_WM_PID", AtomEnum::CARDINAL)
                && pid != 0
                && pid != std::process::id()
                && let Ok(mut state) = observed.lock()
            {
                state.recent = Some((window, pid));
            }
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
    Ok(target)
}
pub fn capture(target: &Mutex<Target>) -> Result<(), String> {
    let mut state = target.lock().map_err(|e| e.to_string())?;
    state.captured = state.recent;
    if state.captured.is_none() {
        return Err("No previous application is available for paste".into());
    }
    Ok(())
}
pub fn validate(target: &Mutex<Target>) -> Result<(), String> {
    let (window, pid) = target
        .lock()
        .map_err(|e| e.to_string())?
        .captured
        .ok_or("No paste target was captured")?;
    let (connection, _) = x11rb::connect(None).map_err(|e| e.to_string())?;
    if pid == std::process::id()
        || property(&connection, window, b"_NET_WM_PID", AtomEnum::CARDINAL)? != pid
    {
        return Err("Previous application is no longer a valid paste target".into());
    }
    Ok(())
}
pub fn paste(target: &Mutex<Target>) -> Result<(), String> {
    validate(target)?;
    let (window, _) = target
        .lock()
        .map_err(|e| e.to_string())?
        .captured
        .ok_or("No paste target was captured")?;
    let (connection, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let root = connection.setup().roots[screen].root;
    let event = ClientMessageEvent::new(
        32,
        window,
        atom(&connection, b"_NET_ACTIVE_WINDOW")?,
        [2, CURRENT_TIME, 0, 0, 0],
    );
    connection
        .send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )
        .map_err(|e| e.to_string())?
        .check()
        .map_err(|e| e.to_string())?;
    connection.flush().map_err(|e| e.to_string())?;
    for _ in 0..100 {
        let modifiers = connection
            .query_pointer(root)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?
            .mask;
        if property(&connection, root, b"_NET_ACTIVE_WINDOW", AtomEnum::WINDOW)? == window
            && u16::from(modifiers)
                & u16::from(ModMask::CONTROL | ModMask::SHIFT | ModMask::M1 | ModMask::M4)
                == 0
        {
            validate(target)?;
            let ctrl = key(&connection, 0xffe3)?;
            let v = key(&connection, u32::from(b'v'))?;
            let send = |kind, code| -> Result<(), String> {
                connection
                    .xtest_fake_input(kind, code, CURRENT_TIME, root, 0, 0, 0)
                    .map_err(|e| e.to_string())?
                    .check()
                    .map_err(|e| e.to_string())
            };
            send(xproto::KEY_PRESS_EVENT, ctrl)?;
            let press = send(xproto::KEY_PRESS_EVENT, v);
            let release_v = send(xproto::KEY_RELEASE_EVENT, v);
            let release_ctrl = send(xproto::KEY_RELEASE_EVENT, ctrl);
            press.and(release_v).and(release_ctrl)?;
            return connection.flush().map_err(|e| e.to_string());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err("The desktop did not restore the paste target or modifier keys are still held".into())
}
