//! Report whether the current Linux display can composite a window with per-pixel alpha.
#[cfg(target_os = "linux")]
pub(crate) async fn supported() -> bool {
    use gtk4::gdk::{Display, prelude::DisplayExt};
    nsh_core::element::run_on_main_thread(|| {
        Display::default().is_some_and(|display| display.is_rgba() && display.is_composited())
    })
    .await
    .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
pub(crate) async fn supported() -> bool {
    false
}
