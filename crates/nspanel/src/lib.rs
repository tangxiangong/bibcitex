//! Copyright (c) 2023 - Present Victor Aremu
//!
//! License: MIT or MIT/Apache-2.0
//!
//! Modified by tangxiangong (2025) for [bibcitex](https://github.com/tangxiangong/bibcitex).
//!
//! # Note
//!
//! This module is forked from the ahkohd's project [tauri-nspanel](https://github.com/ahkohd/tauri-nspanel/tree/v2.1), which is licensed under [MIT](https://github.com/ahkohd/tauri-nspanel/blob/v2.1/LICENSE_MIT) or [MIT](https://github.com/ahkohd/tauri-nspanel/blob/v2.1/LICENSE_MIT)/[Apache 2.0](https://github.com/ahkohd/tauri-nspanel/blob/v2.1/LICENSE_APACHE-2.0).
//!

pub mod builder;
pub mod common;
pub mod event;
pub mod panel;

// Re-export for direct usage (no macros)
pub use objc2;
pub use objc2_app_kit;
pub use objc2_foundation;

use objc2::runtime::ProtocolObject;
use objc2_app_kit::NSWindowDelegate;
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use builder::{CollectionBehavior, PanelLevel, StyleMask, TrackingAreaOptions};
pub use panel::SpotlightPanel;

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
pub trait FromWindow: Panel + Sized {
    /// Create panel from a raw NSWindow pointer
    fn from_window(ns_window: *mut std::ffi::c_void, label: String) -> Result<Self, String>;
}

#[allow(dead_code)]
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

pub trait ManagerExt {
    fn get_webview_panel(&self, label: &str) -> Result<Arc<dyn Panel>, Error>;
    fn register_panel(&self, label: String, panel: Arc<dyn Panel>);
}

#[derive(Debug)]
pub enum Error {
    PanelNotFound,
}

impl ManagerExt for WebviewPanelManager {
    fn get_webview_panel(&self, label: &str) -> Result<Arc<dyn Panel>, Error> {
        let manager = self.0.lock().unwrap();
        match manager.panels.get(label) {
            Some(panel) => Ok(panel.clone()),
            None => Err(Error::PanelNotFound),
        }
    }

    fn register_panel(&self, label: String, panel: Arc<dyn Panel>) {
        let mut manager = self.0.lock().unwrap();
        manager.panels.insert(label, panel);
    }
}

/// Global panel manager instance
pub static PANEL_MANAGER: once_cell::sync::Lazy<WebviewPanelManager> =
    once_cell::sync::Lazy::new(WebviewPanelManager::default);
