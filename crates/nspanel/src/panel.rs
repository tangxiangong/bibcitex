// Copyright (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// This file contains code derived from tauri-nspanel by Victor Aremu (ahkohd)
// Original source: https://github.com/ahkohd/tauri-nspanel
// Copyright (c) 2023 - Present Victor Aremu
// Licensed under MIT OR MIT/Apache-2.0

use super::{FromWindow, Panel};
use objc2::{
    ClassType, DeclaredClass, define_class, msg_send, rc::Retained, runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSPanel, NSResponder, NSView, NSWindowCollectionBehavior,
    NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSSize};
use std::cell::Cell;

// Ivars for the custom panel
struct SpotlightPanelIvars {
    event_handler: Cell<*const std::ffi::c_void>,
}

// Define the custom NSPanel subclass for Spotlight-style panel
define_class!(
    #[unsafe(super = NSPanel)]
    #[name = "SpotlightPanel"]
    #[ivars = SpotlightPanelIvars]
    struct RawSpotlightPanel;

    unsafe impl NSObjectProtocol for RawSpotlightPanel {}

    impl RawSpotlightPanel {
        // Override canBecomeKeyWindow to return false for spotlight behavior
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key_window(&self) -> bool {
            false
        }

        // Override canBecomeMainWindow to return false
        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main_window(&self) -> bool {
            false
        }
    }
);

/// A Spotlight-style panel for macOS
pub struct SpotlightPanel {
    panel: Retained<RawSpotlightPanel>,
    label: String,
}

// SAFETY: While NSPanel must only be used on the main thread, we implement Send + Sync
// to allow passing references through async contexts. Users must ensure
// actual panel operations happen on the main thread.
unsafe impl Send for SpotlightPanel {}
unsafe impl Sync for SpotlightPanel {}

impl SpotlightPanel {
    fn with_label(panel: Retained<RawSpotlightPanel>, label: String) -> Self {
        Self { panel, label }
    }
}

// Implement Panel trait
impl Panel for SpotlightPanel {
    fn show(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, orderFrontRegardless];
        }
    }

    fn hide(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, orderOut: objc2::ffi::nil];
        }
    }

    fn close(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, close];
        }
    }

    fn as_panel(&self) -> &NSPanel {
        // SAFETY: RawSpotlightPanel inherits from NSPanel
        unsafe { &*(&*self.panel as *const RawSpotlightPanel as *const NSPanel) }
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn set_event_handler(&self, handler: Option<&ProtocolObject<dyn NSWindowDelegate>>) {
        unsafe {
            let ivars = self.panel.ivars();

            // Release old event handler if any
            let old_ptr = ivars.event_handler.get();
            if !old_ptr.is_null() {
                let _: () = msg_send![old_ptr as *const NSObject, release];
            }

            match handler {
                Some(h) => {
                    // Retain the new event handler by cloning the reference
                    let retained = h as *const ProtocolObject<dyn NSWindowDelegate>;
                    let obj_ptr = retained as *const std::ffi::c_void;

                    // Store the retained event handler pointer
                    ivars.event_handler.set(obj_ptr);

                    // Set as window delegate
                    let _: () = msg_send![&*self.panel, setDelegate: h];
                }
                None => {
                    // Clear stored delegate
                    ivars.event_handler.set(std::ptr::null());

                    // Remove window delegate
                    let _: () = msg_send![&*self.panel, setDelegate: objc2::ffi::nil];
                }
            }
        }
    }

    // Query methods
    fn is_visible(&self) -> bool {
        unsafe { msg_send![&*self.panel, isVisible] }
    }

    fn is_floating_panel(&self) -> bool {
        unsafe { msg_send![&*self.panel, isFloatingPanel] }
    }

    fn becomes_key_only_if_needed(&self) -> bool {
        unsafe { msg_send![&*self.panel, becomesKeyOnlyIfNeeded] }
    }

    fn can_become_key_window(&self) -> bool {
        unsafe { msg_send![&*self.panel, canBecomeKeyWindow] }
    }

    fn can_become_main_window(&self) -> bool {
        unsafe { msg_send![&*self.panel, canBecomeMainWindow] }
    }

    // Window state methods
    fn make_key_window(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, makeKeyWindow];
        }
    }

    fn make_main_window(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, makeMainWindow];
        }
    }

    fn resign_key_window(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, resignKeyWindow];
        }
    }

    fn make_key_and_order_front(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, makeKeyAndOrderFront: objc2::ffi::nil];
        }
    }

    fn order_front_regardless(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, orderFrontRegardless];
        }
    }

    fn show_and_make_key(&self) {
        unsafe {
            let content_view: Retained<NSView> = msg_send![&*self.panel, contentView];
            let _: bool = msg_send![&*self.panel, makeFirstResponder: &*content_view];
            let _: () = msg_send![&*self.panel, orderFrontRegardless];
            let _: () = msg_send![&*self.panel, makeKeyWindow];
        }
    }

    // Configuration methods
    fn set_level(&self, level: i64) {
        unsafe {
            let _: () = msg_send![&*self.panel, setLevel: level];
        }
    }

    fn set_floating_panel(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setFloatingPanel: value];
        }
    }

    fn set_becomes_key_only_if_needed(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setBecomesKeyOnlyIfNeeded: value];
        }
    }

    fn set_hides_on_deactivate(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setHidesOnDeactivate: value];
        }
    }

    fn set_works_when_modal(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setWorksWhenModal: value];
        }
    }

    fn set_alpha_value(&self, value: f64) {
        unsafe {
            let _: () = msg_send![&*self.panel, setAlphaValue: value];
        }
    }

    fn set_released_when_closed(&self, released: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setReleasedWhenClosed: released];
        }
    }

    fn set_content_size(&self, width: f64, height: f64) {
        unsafe {
            let size = NSSize::new(width, height);
            let _: () = msg_send![&*self.panel, setContentSize: size];
        }
    }

    fn set_has_shadow(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setHasShadow: value];
        }
    }

    fn set_opaque(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setOpaque: value];
        }
    }

    fn set_accepts_mouse_moved_events(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setAcceptsMouseMovedEvents: value];
        }
    }

    fn set_ignores_mouse_events(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setIgnoresMouseEvents: value];
        }
    }

    fn set_movable_by_window_background(&self, value: bool) {
        unsafe {
            let _: () = msg_send![&*self.panel, setMovableByWindowBackground: value];
        }
    }

    fn set_collection_behavior(&self, behavior: NSWindowCollectionBehavior) {
        unsafe {
            let _: () = msg_send![&*self.panel, setCollectionBehavior: behavior];
        }
    }

    fn content_view(&self) -> Retained<NSView> {
        unsafe { msg_send![&*self.panel, contentView] }
    }

    fn resign_main_window(&self) {
        unsafe {
            let _: () = msg_send![&*self.panel, resignMainWindow];
        }
    }

    fn set_style_mask(&self, style_mask: NSWindowStyleMask) {
        unsafe {
            let _: () = msg_send![&*self.panel, setStyleMask: style_mask];
        }
    }

    fn make_first_responder(&self, responder: Option<&NSResponder>) -> bool {
        unsafe {
            let result: bool = match responder {
                Some(resp) => msg_send![&*self.panel, makeFirstResponder: resp],
                None => msg_send![&*self.panel, makeFirstResponder: objc2::ffi::nil],
            };
            result
        }
    }
}

// Implement FromWindow trait
impl FromWindow for SpotlightPanel {
    fn from_window(ns_window: *mut std::ffi::c_void, label: String) -> Result<Self, String> {
        // Use object_setClass from the runtime
        unsafe extern "C" {
            fn object_setClass(
                obj: *mut NSObject,
                cls: *const objc2::runtime::AnyClass,
            ) -> *const objc2::runtime::AnyClass;
        }

        unsafe {
            // Change the window class to our custom panel class
            object_setClass(ns_window as *mut NSObject, RawSpotlightPanel::class());

            // Now cast to our panel type
            let panel_ptr = ns_window as *mut RawSpotlightPanel;

            // Create a Retained from the raw pointer
            let panel =
                Retained::retain(panel_ptr).ok_or_else(|| "Failed to retain panel".to_string())?;

            // Enable auto-resizing for all subviews
            let content_view: Retained<NSView> = msg_send![&panel, contentView];
            let subviews: Retained<NSArray<NSView>> = msg_send![&content_view, subviews];
            let count: usize = msg_send![&subviews, count];

            let resize_mask = NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable;

            for i in 0..count {
                let view: Retained<NSView> = msg_send![&subviews, objectAtIndex: i];
                let _: () = msg_send![&view, setAutoresizingMask: resize_mask];
            }

            Ok(SpotlightPanel::with_label(panel, label))
        }
    }
}

// Implement Drop to clean up the retained delegate
impl Drop for SpotlightPanel {
    fn drop(&mut self) {
        unsafe {
            let ivars = self.panel.ivars();
            let delegate_ptr = ivars.event_handler.get();
            if !delegate_ptr.is_null() {
                let _: () = msg_send![delegate_ptr as *const NSObject, release];
            }
        }
    }
}
