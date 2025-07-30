//! Module for macOS-specific platform NSPanel
//!
//! The codes are copied from the [tauri-nspanel](https://github.com/ahkohd/tauri-nspanel), which licensed under the MIT License or Apache License, Version 2.0.

pub mod builder;
pub mod common;
pub mod event;
pub mod panel;

// Re-export for macro usage
#[doc(hidden)]
pub use objc2;
#[doc(hidden)]
pub use objc2_app_kit;
#[doc(hidden)]
pub use objc2_foundation;
#[doc(hidden)]
pub use pastey;

use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use objc2::runtime::ProtocolObject;
use objc2_app_kit::NSWindowDelegate;
use tauri::{
    Manager, Runtime, WebviewWindow,
    plugin::{Builder, TauriPlugin},
};

pub use builder::{CollectionBehavior, PanelBuilder, PanelLevel, StyleMask, TrackingAreaOptions};

// Re-export commonly used types for convenience
pub use objc2::runtime::AnyObject;
pub use objc2_app_kit::{NSPanel, NSResponder, NSView, NSWindow};
pub use objc2_foundation::{NSNotification, NSObject, NSPoint, NSRect, NSSize};

/// Trait for event handlers that can be used with panels
pub trait EventHandler {
    /// Get the NSWindowDelegate protocol object
    fn as_delegate(&self) -> ProtocolObject<dyn NSWindowDelegate>;
}

/// Common trait for all panel types
pub trait Panel: Send + Sync {
    /// Show the panel
    fn show(&self);

    /// Hide the panel
    fn hide(&self);

    /// Close the panel
    fn close(&self);

    /// Get a reference to the underlying NSPanel
    fn as_panel(&self) -> &objc2_app_kit::NSPanel;

    /// Get the panel label
    fn label(&self) -> &str;

    /// Downcast to concrete type
    fn as_any(&self) -> &dyn Any;

    /// Set the event handler (window delegate)
    /// Pass `None` to remove the current delegate
    fn set_event_handler(&self, handler: Option<&ProtocolObject<dyn NSWindowDelegate>>);

    // Query methods
    /// Check if the panel is visible
    fn is_visible(&self) -> bool;

    /// Check if this is a floating panel
    fn is_floating_panel(&self) -> bool;

    /// Check if panel becomes key only if needed
    fn becomes_key_only_if_needed(&self) -> bool;

    /// Check if panel can become key window
    fn can_become_key_window(&self) -> bool;

    /// Check if panel can become main window
    fn can_become_main_window(&self) -> bool;

    // Window state methods
    /// Make the panel key window
    fn make_key_window(&self);

    /// Make the panel main window
    fn make_main_window(&self);

    /// Resign key window status
    fn resign_key_window(&self);

    /// Make key and order front
    fn make_key_and_order_front(&self);

    /// Order front regardless
    fn order_front_regardless(&self);

    /// Show and make key
    fn show_and_make_key(&self);

    // Configuration methods
    /// Set the window level
    fn set_level(&self, level: i64);

    /// Set whether this is a floating panel
    fn set_floating_panel(&self, value: bool);

    /// Set whether panel becomes key only if needed
    fn set_becomes_key_only_if_needed(&self, value: bool);

    /// Set whether panel hides on deactivate
    fn set_hides_on_deactivate(&self, value: bool);

    /// Set whether panel works when modal
    fn set_works_when_modal(&self, value: bool);

    /// Set the alpha value
    fn set_alpha_value(&self, value: f64);

    /// Set whether the panel should be released when closed
    fn set_released_when_closed(&self, released: bool);

    /// Set the content size
    fn set_content_size(&self, width: f64, height: f64);

    /// Set whether panel has shadow
    fn set_has_shadow(&self, value: bool);

    /// Set whether panel is opaque
    fn set_opaque(&self, value: bool);

    /// Set whether panel accepts mouse moved events
    fn set_accepts_mouse_moved_events(&self, value: bool);

    /// Set whether panel ignores mouse events
    fn set_ignores_mouse_events(&self, value: bool);

    /// Set whether panel is movable by window background
    fn set_movable_by_window_background(&self, value: bool);

    /// Set the collection behavior
    fn set_collection_behavior(&self, behavior: objc2_app_kit::NSWindowCollectionBehavior);

    /// Get the content view
    fn content_view(&self) -> objc2::rc::Retained<objc2_app_kit::NSView>;

    /// Resign main window status
    fn resign_main_window(&self);

    /// Set the style mask
    fn set_style_mask(&self, style_mask: objc2_app_kit::NSWindowStyleMask);

    /// Make a view the first responder
    fn make_first_responder(&self, responder: Option<&objc2_app_kit::NSResponder>) -> bool;
}

/// Trait for panels that can be created from a window
pub trait FromWindow<R: Runtime>: Panel + Sized {
    /// Create panel from a Tauri window
    fn from_window(window: WebviewWindow<R>, label: String) -> tauri::Result<Self>;
}

#[derive(Default)]
pub struct Store {
    panels: HashMap<String, Arc<dyn Panel>>,
}

pub struct WebviewPanelManager(pub Mutex<Store>);

impl Default for WebviewPanelManager {
    fn default() -> Self {
        Self(Mutex::new(Store::default()))
    }
}

pub trait ManagerExt<R: Runtime> {
    fn get_webview_panel(&self, label: &str) -> Result<Arc<dyn Panel>, Error>;
}

#[derive(Debug)]
pub enum Error {
    PanelNotFound,
}

impl<R: Runtime, T: Manager<R>> ManagerExt<R> for T {
    fn get_webview_panel(&self, label: &str) -> Result<Arc<dyn Panel>, Error> {
        let manager = self.state::<self::WebviewPanelManager>();
        let manager = manager.0.lock().unwrap();

        match manager.panels.get(label) {
            Some(panel) => Ok(panel.clone()),
            None => Err(Error::PanelNotFound),
        }
    }
}

pub trait WebviewWindowExt<R: Runtime> {
    /// Convert window to specific panel type
    fn to_panel<P: FromWindow<R> + 'static>(&self) -> tauri::Result<Arc<dyn Panel>>;
}

impl<R: Runtime> WebviewWindowExt<R> for WebviewWindow<R> {
    fn to_panel<P: FromWindow<R> + 'static>(&self) -> tauri::Result<Arc<dyn Panel>> {
        let label = self.label().to_string();
        let panel = P::from_window(self.clone(), label.clone())?;
        let arc_panel = Arc::new(panel) as Arc<dyn Panel>;

        // Register with manager
        let manager = self.state::<WebviewPanelManager>();
        manager
            .0
            .lock()
            .unwrap()
            .panels
            .insert(label, arc_panel.clone());

        Ok(arc_panel)
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("nspanel")
        .setup(|app, _api| {
            app.manage(self::WebviewPanelManager::default());

            Ok(())
        })
        .build()
}
