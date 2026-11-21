//! Place the mapped GTK tray workbench beside its StatusNotifier activation point.
#[cfg(target_os = "linux")]
mod x11 {
    use gtk4::{Window, gdk, prelude::*};
    use libloading::os::unix::Library;
    use nsh_core::element::{blocking, run_on_main_thread};
    use std::ffi::c_ulong;
    use x11rb::{
        connection::Connection,
        protocol::xproto::{ClientMessageEvent, ConnectionExt, EventMask},
    };

    struct Placement {
        display: String,
        window: u32,
        x: i32,
        y: i32,
    }

    fn placement(x: i32, y: i32) -> Result<Option<Placement>, String> {
        let Some(display) = gdk::Display::default() else {
            return Ok(None);
        };
        if display.type_().name() != "GdkX11Display" {
            return Ok(None);
        }
        let Some(window) = Window::list_toplevels()
            .into_iter()
            .filter_map(|widget| widget.downcast::<Window>().ok())
            .find(|win| win.is_mapped() && win.title().as_deref() == Some("BibCiTeX Tray"))
        else {
            return Ok(None);
        };
        let Some(surface) = window.surface() else {
            return Ok(None);
        };
        let monitors = display.monitors();
        let Some(monitor) = (0..monitors.n_items())
            .filter_map(|index| monitors.item(index)?.downcast::<gdk::Monitor>().ok())
            .find(|monitor| {
                let rect = monitor.geometry();
                let scale = monitor.scale_factor().max(1);
                rect.contains_point(x.div_euclid(scale), y.div_euclid(scale))
            })
        else {
            // The panel can send an unlocatable point when a monitor disappears.
            return Ok(None);
        };

        // Resolve the optional GTK X11 backend only when it is in use. A GTK
        // build with Wayland alone need not export either of these functions.
        let gtk = Library::this();
        type GetXid = unsafe extern "C" fn(*mut gdk::ffi::GdkSurface) -> c_ulong;
        type Workarea =
            unsafe extern "C" fn(*mut gdk::ffi::GdkMonitor, *mut gdk::ffi::GdkRectangle);
        // SAFETY: GDK's X11 header declares both functions with these signatures.
        // Objects belong to the current X11 display and are held on GTK's thread.
        let (xid, area) = unsafe {
            let xid = gtk
                .get::<GetXid>(b"gdk_x11_surface_get_xid\0")
                .map_err(|e| e.to_string())?;
            let workarea = gtk
                .get::<Workarea>(b"gdk_x11_monitor_get_workarea\0")
                .map_err(|e| e.to_string())?;
            let mut area = std::mem::MaybeUninit::uninit();
            workarea(monitor.as_ptr(), area.as_mut_ptr());
            (xid(surface.as_ptr()), area.assume_init())
        };
        let scale = monitor.scale_factor().max(1);
        let left = area.x * scale;
        let top = area.y * scale;
        let right = left + area.width * scale;
        let bottom = top + area.height * scale;
        let width = surface.width() * surface.scale_factor();
        let height = surface.height() * surface.scale_factor();
        let gap = 8 * scale;
        let below = y + gap;
        let preferred_y = if below + height <= bottom {
            below
        } else {
            y - gap - height
        };
        Ok(Some(Placement {
            display: display.name().to_string(),
            window: u32::try_from(xid).map_err(|e| e.to_string())?,
            x: (x - width / 2).clamp(left, (right - width).max(left)),
            y: preferred_y.clamp(top, (bottom - height).max(top)),
        }))
    }

    fn request_move(placement: Placement) -> Result<(), Box<dyn std::error::Error>> {
        let (connection, screen) = x11rb::connect(Some(&placement.display))?;
        let root = connection.setup().roots[screen].root;
        let atom = connection
            .intern_atom(false, b"_NET_MOVERESIZE_WINDOW")?
            .reply()?
            .atom;
        // EWMH: NorthWestGravity, x/y supplied, source = normal application.
        let flags = 1 | (1 << 8) | (1 << 9) | (1 << 12);
        let event = ClientMessageEvent::new(
            32,
            placement.window,
            atom,
            [flags, placement.x as u32, placement.y as u32, 0, 0],
        );
        connection
            .send_event(
                false,
                root,
                EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                event,
            )?
            .check()?;
        connection.flush()?;
        Ok(())
    }

    pub(super) async fn place(x: i32, y: i32) -> Result<bool, String> {
        let Some(target) = run_on_main_thread(move || placement(x, y))
            .await
            .map_err(|e| e.to_string())??
        else {
            return Ok(false);
        };
        blocking(move || request_move(target).map_err(|e| e.to_string()))
            .await
            .map_err(|e| e.to_string())??;
        Ok(true)
    }
}

/// Return true when a move request was sent to the X11 window manager.
pub(crate) async fn place_tray(x: i32, y: i32) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        x11::place(x, y).await
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (x, y);
        Ok(false)
    }
}
