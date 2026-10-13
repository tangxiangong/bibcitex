use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use x11rb::{
    CURRENT_TIME,
    connection::Connection,
    protocol::{
        xproto::{self, AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, ModMask},
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
};

#[derive(Default, Clone)]
pub(super) struct Target {
    recent: Option<(u32, u32)>,
    captured: Option<(u32, u32)>,
    pub(super) captured_name: Option<String>,
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
fn external_target(connection: &RustConnection, root: u32) -> Option<(u32, u32)> {
    let window = property(connection, root, b"_NET_ACTIVE_WINDOW", AtomEnum::WINDOW).ok()?;
    let pid = property(connection, window, b"_NET_WM_PID", AtomEnum::CARDINAL).ok()?;
    (pid != 0 && pid != std::process::id()).then_some((window, pid))
}
pub(super) fn start(
    report_error: impl Fn(String) + Send + 'static,
) -> Result<Arc<Mutex<Target>>, String> {
    let (connection, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let root = connection.setup().roots[screen].root;
    let target = Arc::new(Mutex::new(Target::default()));
    // The observer must not keep a discarded paste session alive.
    let observed = Arc::downgrade(&target);
    std::thread::spawn(move || {
        while let Some(observed) = observed.upgrade() {
            if let Some(recent) = external_target(&connection, root)
                && let Ok(mut state) = observed.lock()
            {
                state.recent = Some(recent);
            }
            if let Err(error) = connection.poll_for_event() {
                report_error(error.to_string());
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    Ok(target)
}
pub(super) fn capture(target: &Mutex<Target>) -> Result<(), String> {
    let mut state = target.lock().map_err(|e| e.to_string())?;
    // Shortcut handling uses a separate connection. Refresh before freezing the
    // target so a shortcut cannot beat the observer's next polling iteration.
    if let Ok((connection, screen)) = x11rb::connect(None)
        && let Some(recent) = external_target(&connection, connection.setup().roots[screen].root)
    {
        state.recent = Some(recent);
    }
    state.captured = state.recent;
    state.captured_name = state
        .captured
        .and_then(|(window, pid)| application_name(window, pid));
    if state.captured.is_none() {
        return Err("No previous application is available for paste".into());
    }
    Ok(())
}
fn application_name(window: u32, pid: u32) -> Option<String> {
    let class = (|| {
        let (connection, _) = x11rb::connect(None).ok()?;
        let reply = connection
            .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)
            .ok()?
            .reply()
            .ok()?;
        let mut names = reply.value.split(|byte| *byte == 0);
        let instance = names.next()?;
        let class = names
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or(instance);
        let name = String::from_utf8_lossy(class).trim().to_owned();
        (!name.is_empty()).then_some(name)
    })();
    class.or_else(|| {
        std::fs::read_link(format!("/proc/{pid}/exe"))
            .ok()?
            .file_stem()
            .map(|name| name.to_string_lossy().into_owned())
    })
}

pub(super) fn validate(target: &Mutex<Target>) -> Result<(), String> {
    let captured = captured_target(target)?;
    let (connection, _) = x11rb::connect(None).map_err(|e| e.to_string())?;
    validate_window(&connection, captured)
}
fn captured_target(target: &Mutex<Target>) -> Result<(u32, u32), String> {
    target
        .lock()
        .map_err(|e| e.to_string())?
        .captured
        .ok_or_else(|| "No paste target was captured".into())
}
fn validate_window(connection: &RustConnection, (window, pid): (u32, u32)) -> Result<(), String> {
    crate::validate_target_process(
        pid,
        property(connection, window, b"_NET_WM_PID", AtomEnum::CARDINAL)?,
    )
}
pub(super) fn paste(target: &Mutex<Target>) -> Result<(), String> {
    // Capture can run on the host thread while input is queued on the worker.
    // Verify the same immutable identity before focus restoration and injection.
    let captured = captured_target(target)?;
    let (window, _) = captured;
    let (connection, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
    validate_window(&connection, captured)?;
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
            validate_window(&connection, captured)?;
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
