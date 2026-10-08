//! Consume a compositor-issued activation token for a live GPUI Wayland surface.
use gpui_kit::Window;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_surface::WlSurface},
};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;

struct Activation;
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Activation {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
wayland_client::delegate_noop!(Activation: XdgActivationV1);

pub fn activate(window: &Window, token: &str) -> Result<(), String> {
    let display_guard = window.display_handle().map_err(|e| e.to_string())?;
    let window_guard = HasWindowHandle::window_handle(window).map_err(|e| e.to_string())?;
    let (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(surface)) =
        (display_guard.as_raw(), window_guard.as_raw())
    else {
        window.activate_window();
        return Ok(());
    };
    // SAFETY: GPUI's Linux backend uses libwayland-client's system backend.
    // Both borrowed raw-handle guards and &Window outlive the temporary backend,
    // which does not own/disconnect the foreign display. No handles escape.
    let backend = unsafe {
        wayland_backend::client::Backend::from_foreign_display(display.display.as_ptr().cast())
    };
    let connection = Connection::from_backend(backend);
    let (globals, queue) =
        registry_queue_init::<Activation>(&connection).map_err(|e| e.to_string())?;
    let activation: XdgActivationV1 = globals
        .bind(&queue.handle(), 1..=1, ())
        .map_err(|e| e.to_string())?;
    // SAFETY: the live WindowHandle identifies a wl_surface on this same display.
    // Reconstituting a proxy only borrows it; we never destroy GPUI's surface.
    let id = unsafe {
        wayland_backend::client::ObjectId::from_ptr(
            WlSurface::interface(),
            surface.surface.as_ptr().cast(),
        )
    }
    .map_err(|e| e.to_string())?;
    let surface = WlSurface::from_id(&connection, id).map_err(|e| e.to_string())?;
    activation.activate(token.to_owned(), &surface);
    activation.destroy();
    connection.flush().map_err(|e| e.to_string())
}

/// Check the compositor's advertised protocol before requesting a layer surface.
pub fn supports_layer_shell() -> bool {
    static SUPPORTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SUPPORTED.get_or_init(|| {
        let Ok(connection) = Connection::connect_to_env() else {
            return false;
        };
        let Ok((globals, _queue)) = registry_queue_init::<Activation>(&connection) else {
            return false;
        };
        globals.contents().with_list(|globals| {
            globals
                .iter()
                .any(|global| global.interface == "zwlr_layer_shell_v1")
        })
    })
}
