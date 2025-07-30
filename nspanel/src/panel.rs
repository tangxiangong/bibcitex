// Re-export commonly used types
pub use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSTrackingAreaOptions, NSWindowCollectionBehavior, NSWindowStyleMask,
};

/// Macro to create a custom NSPanel class
///
/// This macro generates a custom NSPanel subclass with the specified configuration.
/// The first parameter is the name of your custom panel class.
///
/// **Implementation Details**:
/// - The macro generates an internal `Raw{ClassName}` Objective-C class
/// - A public `{ClassName}` wrapper type that implements `Send` and `Sync`
/// - All methods are implemented on the wrapper type
///
/// **Thread Safety**: The wrapper type implements `Send` and `Sync` to allow
/// passing references through Tauri's command system. However, all actual panel
/// operations must be performed on the main thread.
///
/// ## Sections:
/// - `config`: Override NSPanel methods that return boolean values
/// - `with`: Optional configurations (tracking_area, etc.)
///
/// ## Mouse Tracking:
/// When you enable tracking_area in the `with` section, mouse event callbacks become available
/// on your event handler. You can set callbacks for:
/// - `on_mouse_entered()` - Called when mouse enters the panel
/// - `on_mouse_exited()` - Called when mouse exits the panel
/// - `on_mouse_moved()` - Called when mouse moves within the panel
/// - `on_cursor_update()` - Called when cursor needs to be updated
///
/// ## Usage:
/// ```rust
/// use tauri_nspanel::{panel, panel_event};
/// use tauri_nspanel::event::EventReturn;
///
/// // Define your custom panel class
/// panel!(MyCustomPanel {
///     // Config overrides - these affect compile-time behavior
///     config: {
///         canBecomeKeyWindow: true,
///         canBecomeMainWindow: false,
///     },
///     // Optional configurations
///     with: {
///         tracking_area: {
///             options: NSTrackingAreaOptions::NSTrackingActiveAlways
///                    | NSTrackingAreaOptions::NSTrackingMouseEnteredAndExited
///                    | NSTrackingAreaOptions::NSTrackingMouseMoved,
///             auto_resize: true,
///         }
///     }
/// });
///
/// // In your Tauri app:
/// fn create_panel(window: tauri::WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
///     // Convert existing Tauri window to your custom panel
///     let panel = MyCustomPanel::from_window(window)?;
///
///     // Use control methods
///     panel.show();
///     panel.set_level(5i64); // NSStatusWindowLevel
///     panel.set_floating_panel(true);
///
///     // Create and attach an event handler
///     let handler = MyPanelEventHandler::new();
///     handler.window_did_become_key(|args| {
///         println!("Panel became key window");
///         None
///     });
///
///     // If tracking_area is enabled, you can set mouse event callbacks
///     handler.on_mouse_entered(|event| {
///         println!("Mouse entered the panel");
///     });
///
///     handler.on_mouse_moved(|event| {
///         let location = unsafe { event.locationInWindow() };
///         println!("Mouse moved to: x={}, y={}", location.x, location.y);
///     });
///
///     panel.set_event_handler(Some(handler.as_protocol_object()));
///
///     Ok(())
/// }
/// ```
///
/// ## Available Methods:
/// - `show()`, `hide()`, `close()`
/// - `make_key_window()`, `resign_key_window()`
/// - `set_level()`, `set_alpha_value()`, `set_content_size()`
/// - `set_floating_panel()`, `set_has_shadow()`, `set_opaque()`
/// - `set_accepts_mouse_moved_events()`, `set_ignores_mouse_events()`
/// - And many more...
/// ```
#[macro_export]
macro_rules! panel {
    (
        $class_name:ident {
            $(config: {
                $($method:ident: $value:expr),* $(,)?
            })?
            $(with: {
                $(tracking_area: {
                    options: $tracking_options:expr,
                    auto_resize: $auto_resize:expr $(,)?
                })?
            })?
        }
    ) => {
        $crate::pastey::paste! {
            struct [<$class_name Ivars>] {
                event_handler: std::cell::Cell<*const std::ffi::c_void>,
            }

            $crate::objc2::define_class!(
                #[unsafe(super = $crate::objc2_app_kit::NSPanel)]
                #[name = stringify!($class_name)]
                #[ivars = [<$class_name Ivars>]]

                struct [<Raw $class_name>];

                unsafe impl NSObjectProtocol for [<Raw $class_name>] {}

                impl [<Raw $class_name>] {
                    $($(
                        #[doc = " Returns whether panels of this class " $method]
                        #[unsafe(method($method))]
                        fn [<__ $method:snake>]() -> bool {
                            $value
                        }

                        #[doc = " Returns whether this specific panel instance " $method]
                        #[unsafe(method($method))]
                        fn [<__ $method:snake _instance>](&self) -> bool {
                            $value
                        }
                    )*)?

                    // Mouse tracking methods - forward to delegate if set
                    #[unsafe(method(mouseEntered:))]
                    fn __mouse_entered(&self, event: &$crate::objc2_app_kit::NSEvent) {
                        unsafe {
                            let ivars = self.ivars();
                            let delegate_ptr = ivars.event_handler.get();
                            if !delegate_ptr.is_null() {
                                let delegate = delegate_ptr as *const $crate::objc2_foundation::NSObject;
                                // Check if delegate responds to selector before calling
                                let selector = $crate::objc2::sel!(mouseEntered:);
                                let responds: bool = $crate::objc2::msg_send![delegate, respondsToSelector: selector];
                                if responds {
                                    let _: () = $crate::objc2::msg_send![delegate, mouseEntered: event];
                                }
                            }
                        }
                    }

                    #[unsafe(method(mouseExited:))]
                    fn __mouse_exited(&self, event: &$crate::objc2_app_kit::NSEvent) {
                        unsafe {
                            let ivars = self.ivars();
                            let delegate_ptr = ivars.event_handler.get();
                            if !delegate_ptr.is_null() {
                                let delegate = delegate_ptr as *const $crate::objc2_foundation::NSObject;
                                // Check if delegate responds to selector before calling
                                let selector = $crate::objc2::sel!(mouseExited:);
                                let responds: bool = $crate::objc2::msg_send![delegate, respondsToSelector: selector];
                                if responds {
                                    let _: () = $crate::objc2::msg_send![delegate, mouseExited: event];
                                }
                            }
                        }
                    }

                    #[unsafe(method(mouseMoved:))]
                    fn __mouse_moved(&self, event: &$crate::objc2_app_kit::NSEvent) {
                        unsafe {
                            let ivars = self.ivars();
                            let delegate_ptr = ivars.event_handler.get();
                            if !delegate_ptr.is_null() {
                                let delegate = delegate_ptr as *const $crate::objc2_foundation::NSObject;
                                // Check if delegate responds to selector before calling
                                let selector = $crate::objc2::sel!(mouseMoved:);
                                let responds: bool = $crate::objc2::msg_send![delegate, respondsToSelector: selector];
                                if responds {
                                    let _: () = $crate::objc2::msg_send![delegate, mouseMoved: event];
                                }
                            }
                        }
                    }

                    #[unsafe(method(cursorUpdate:))]
                    fn __cursor_update(&self, event: &$crate::objc2_app_kit::NSEvent) {
                        unsafe {
                            let ivars = self.ivars();
                            let delegate_ptr = ivars.event_handler.get();
                            if !delegate_ptr.is_null() {
                                let delegate = delegate_ptr as *const $crate::objc2_foundation::NSObject;
                                // Check if delegate responds to selector before calling
                                let selector = $crate::objc2::sel!(cursorUpdate:);
                                let responds: bool = $crate::objc2::msg_send![delegate, respondsToSelector: selector];
                                if responds {
                                    let _: () = $crate::objc2::msg_send![delegate, cursorUpdate: event];
                                }
                            }
                        }
                    }
                }
            );

            #[doc = " A public wrapper for `Raw" $class_name "` "]
            pub struct $class_name {
                panel: $crate::objc2::rc::Retained<[<Raw $class_name>]>,
                label: String,
            }

            // SAFETY: While NSPanel must only be used on the main thread, we implement Send + Sync
            // to allow passing references through Tauri's command system. Users must ensure
            // actual panel operations happen on the main thread.
            unsafe impl Send for $class_name {}
            unsafe impl Sync for $class_name {}

            impl $class_name {
                fn with_label(panel: $crate::objc2::rc::Retained<[<Raw $class_name>]>, label: String) -> Self {
                    Self { panel, label }
                }

                /// Convert a Tauri window to this panel type (convenience method)
                pub fn from_window<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) -> tauri::Result<Self> {
                    let label = window.label().to_string();
                    <Self as $crate::FromWindow<R>>::from_window(window.clone(), label)
                }

            }

            // Implement Panel trait
            impl $crate::Panel for $class_name {
                fn show(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, orderFrontRegardless];
                    }
                }

                fn hide(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, orderOut: $crate::objc2::ffi::nil];
                    }
                }

                fn close(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, close];
                    }
                }

                fn as_panel(&self) -> &$crate::objc2_app_kit::NSPanel {
                    // SAFETY: Raw class inherits from NSPanel
                    unsafe { &*(&*self.panel as *const [<Raw $class_name>] as *const $crate::objc2_app_kit::NSPanel) }
                    // Cast the retained Raw panel to NSPanel reference
                }

                fn label(&self) -> &str {
                    &self.label
                }

                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }

                fn set_event_handler(
                    &self,
                    handler: Option<&$crate::objc2::runtime::ProtocolObject<dyn $crate::objc2_app_kit::NSWindowDelegate>>,
                ) {
                    unsafe {
                        let ivars = (*self.panel).ivars();

                        // Release old event handler if any
                        let old_ptr = ivars.event_handler.get();
                        if !old_ptr.is_null() {
                            let _: () = $crate::objc2::msg_send![old_ptr as *const $crate::objc2_foundation::NSObject, release];
                        }

                        match handler {
                            Some(h) => {
                                // Retain the new event handler
                                let retained = h.retain();
                                let obj_ptr = &*retained as *const _ as *const std::ffi::c_void;

                                // Store the retained event handler pointer
                                ivars.event_handler.set(obj_ptr);

                                // Forget the retained object so it won't be dropped
                                std::mem::forget(retained);

                                // Set as window delegate
                                let _: () = $crate::objc2::msg_send![&*self.panel, setDelegate: h];
                            }
                            None => {
                                // Clear stored delegate
                                ivars.event_handler.set(std::ptr::null());

                                // Remove window delegate
                                let _: () = $crate::objc2::msg_send![&*self.panel, setDelegate: $crate::objc2::ffi::nil];
                            }
                        }
                    }
                }

                // Query methods
                fn is_visible(&self) -> bool {
                    unsafe {
                        $crate::objc2::msg_send![&*self.panel, isVisible]
                    }
                }

                fn is_floating_panel(&self) -> bool {
                    unsafe {
                        $crate::objc2::msg_send![&*self.panel, isFloatingPanel]
                    }
                }

                fn becomes_key_only_if_needed(&self) -> bool {
                    unsafe {
                        $crate::objc2::msg_send![&*self.panel, becomesKeyOnlyIfNeeded]
                    }
                }

                fn can_become_key_window(&self) -> bool {
                    unsafe {
                        $crate::objc2::msg_send![&*self.panel, canBecomeKeyWindow]
                    }
                }

                fn can_become_main_window(&self) -> bool {
                    unsafe {
                        $crate::objc2::msg_send![&*self.panel, canBecomeMainWindow]
                    }
                }

                // Window state methods
                fn make_key_window(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, makeKeyWindow];
                    }
                }

                fn make_main_window(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, makeMainWindow];
                    }
                }

                fn resign_key_window(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, resignKeyWindow];
                    }
                }

                fn make_key_and_order_front(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, makeKeyAndOrderFront: $crate::objc2::ffi::nil];
                    }
                }

                fn order_front_regardless(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, orderFrontRegardless];
                    }
                }

                fn show_and_make_key(&self) {
                    unsafe {
                        let content_view: $crate::objc2::rc::Retained<$crate::objc2_app_kit::NSView> =
                            $crate::objc2::msg_send![&*self.panel, contentView];
                        let _: bool = $crate::objc2::msg_send![&*self.panel, makeFirstResponder: &*content_view];
                        let _: () = $crate::objc2::msg_send![&*self.panel, orderFrontRegardless];
                        let _: () = $crate::objc2::msg_send![&*self.panel, makeKeyWindow];
                    }
                }

                // Configuration methods
                fn set_level(&self, level: i64) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setLevel: level];
                    }
                }

                fn set_floating_panel(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setFloatingPanel: value];
                    }
                }

                fn set_becomes_key_only_if_needed(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setBecomesKeyOnlyIfNeeded: value];
                    }
                }

                fn set_hides_on_deactivate(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setHidesOnDeactivate: value];
                    }
                }

                fn set_works_when_modal(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setWorksWhenModal: value];
                    }
                }

                fn set_alpha_value(&self, value: f64) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setAlphaValue: value];
                    }
                }

                fn set_released_when_closed(&self, released: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setReleasedWhenClosed: released];
                    }
                }

                fn set_content_size(&self, width: f64, height: f64) {
                    unsafe {
                        let size = $crate::objc2_foundation::NSSize::new(width, height);
                        let _: () = $crate::objc2::msg_send![&*self.panel, setContentSize: size];
                    }
                }

                fn set_has_shadow(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setHasShadow: value];
                    }
                }

                fn set_opaque(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setOpaque: value];
                    }
                }

                fn set_accepts_mouse_moved_events(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setAcceptsMouseMovedEvents: value];
                    }
                }

                fn set_ignores_mouse_events(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setIgnoresMouseEvents: value];
                    }
                }

                fn set_movable_by_window_background(&self, value: bool) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setMovableByWindowBackground: value];
                    }
                }

                fn set_collection_behavior(&self, behavior: $crate::objc2_app_kit::NSWindowCollectionBehavior) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setCollectionBehavior: behavior];
                    }
                }

                fn content_view(&self) -> $crate::objc2::rc::Retained<$crate::objc2_app_kit::NSView> {
                    unsafe {
                        $crate::objc2::msg_send![&*self.panel, contentView]
                    }
                }

                fn resign_main_window(&self) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, resignMainWindow];
                    }
                }

                fn set_style_mask(&self, style_mask: $crate::objc2_app_kit::NSWindowStyleMask) {
                    unsafe {
                        let _: () = $crate::objc2::msg_send![&*self.panel, setStyleMask: style_mask];
                    }
                }

                fn make_first_responder(&self, responder: Option<&$crate::objc2_app_kit::NSResponder>) -> bool {
                    unsafe {
                        let result: bool = match responder {
                            Some(resp) => $crate::objc2::msg_send![&*self.panel, makeFirstResponder: resp],
                            None => $crate::objc2::msg_send![&*self.panel, makeFirstResponder: $crate::objc2::ffi::nil],
                        };
                        result
                    }
                }
            }

            // Implement FromWindow trait
            impl<R: tauri::Runtime> $crate::FromWindow<R> for $class_name {
                fn from_window(window: tauri::WebviewWindow<R>, label: String) -> tauri::Result<Self> {
                    let ns_window = window.ns_window().map_err(|e| {
                        tauri::Error::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to get NSWindow: {:?}", e),
                        ))
                    })?;

                    unsafe {
                        // Use object_setClass from the runtime
                        extern "C" {
                            fn object_setClass(
                                obj: *mut $crate::objc2_foundation::NSObject,
                                cls: *const $crate::objc2::runtime::AnyClass,
                            ) -> *const $crate::objc2::runtime::AnyClass;
                        }

                        // Change the window class to our custom panel class
                        object_setClass(
                            ns_window as *mut $crate::objc2_foundation::NSObject,
                            [<Raw $class_name>]::class(),
                        );

                        // Now cast to our panel type
                        let panel_ptr = ns_window as *mut [<Raw $class_name>];

                        // Create a Retained from the raw pointer
                        let panel = $crate::objc2::rc::Retained::retain(panel_ptr).ok_or_else(|| {
                            tauri::Error::Io(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to retain panel",
                            ))
                        })?;

                        // Add tracking area if configured
                        $($(
                            Self::add_tracking_area(&panel, $tracking_options, $auto_resize);
                        )?)?

                        // Enable auto-resizing for all subviews
                        let content_view: $crate::objc2::rc::Retained<$crate::objc2_app_kit::NSView> =
                            $crate::objc2::msg_send![&panel, contentView];
                        let subviews: $crate::objc2::rc::Retained<$crate::objc2_foundation::NSArray<$crate::objc2_app_kit::NSView>> =
                            $crate::objc2::msg_send![&content_view, subviews];
                        let count: usize = $crate::objc2::msg_send![&subviews, count];

                        let resize_mask = $crate::objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                            | $crate::objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable;

                        for i in 0..count {
                            let view: $crate::objc2::rc::Retained<$crate::objc2_app_kit::NSView> =
                                $crate::objc2::msg_send![&subviews, objectAtIndex: i];
                            let _: () = $crate::objc2::msg_send![&view, setAutoresizingMask: resize_mask];
                        }

                        Ok($class_name::with_label(panel, label))
                    }
                }
            }

            // Implement Drop to clean up the retained delegate
            impl Drop for $class_name {
                fn drop(&mut self) {
                    unsafe {
                        let ivars = (*self.panel).ivars();
                        let delegate_ptr = ivars.event_handler.get();
                        if !delegate_ptr.is_null() {
                            let _: () = $crate::objc2::msg_send![delegate_ptr as *const $crate::objc2_foundation::NSObject, release];
                        }
                    }
                }
            }

            // Add tracking area helper
            impl $class_name {
                #[allow(unused)]
                fn add_tracking_area(panel: &$crate::objc2_app_kit::NSPanel, options: impl Into<$crate::objc2_app_kit::NSTrackingAreaOptions>, auto_resize: bool) {
                    unsafe {
                        let content_view: $crate::objc2::rc::Retained<$crate::objc2_app_kit::NSView> =
                            $crate::objc2::msg_send![panel, contentView];
                        let bounds: $crate::objc2_foundation::NSRect =
                            $crate::objc2::msg_send![&content_view, bounds];

                        // Create tracking area
                        let tracking_area: $crate::objc2::rc::Retained<$crate::objc2_app_kit::NSTrackingArea> = {
                            let alloc: *mut $crate::objc2_app_kit::NSTrackingArea = $crate::objc2::msg_send![
                                $crate::objc2_app_kit::NSTrackingArea::class(),
                                alloc
                            ];
                            let area: *mut $crate::objc2_app_kit::NSTrackingArea = $crate::objc2::msg_send![
                                alloc,
                                initWithRect: bounds,
                                options: options.into(),
                                owner: &*content_view,
                                userInfo: $crate::objc2::ffi::nil
                            ];
                            $crate::objc2::rc::Retained::from_raw(area).unwrap()
                        };

                        // Set auto-resizing if requested
                        if auto_resize {
                            let resize_mask = $crate::objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                                | $crate::objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable;
                            let _: () = $crate::objc2::msg_send![&content_view, setAutoresizingMask: resize_mask];
                        }

                        // Add tracking area
                        let _: () = $crate::objc2::msg_send![&content_view, addTrackingArea: &*tracking_area];
                    }
                }
            }
        }
    };
}
