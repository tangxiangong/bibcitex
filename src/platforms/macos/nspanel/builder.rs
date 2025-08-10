// Copyright (c) 2025 BibCiTeX Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// This file contains code derived from tauri-nspanel by Victor Aremu (ahkohd)
// Original source: https://github.com/ahkohd/tauri-nspanel
// Copyright (c) 2023 - Present Victor Aremu
// Licensed under MIT OR MIT/Apache-2.0

#![allow(unused)]

use super::{FromWindow, Panel};
use objc2_app_kit::{NSTrackingAreaOptions, NSWindowCollectionBehavior, NSWindowStyleMask};
use objc2_foundation::MainThreadMarker;
use std::sync::Arc;

/// Window level constants for NSPanel
/// Based on NSWindow.Level constants from macOS
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelLevel {
    /// Normal window level (0)
    Normal,
    /// Submenu window level (3)
    Submenu,
    /// Torn-off menu window level (3)
    TornOffMenu,
    /// Floating window level (4)
    Floating,
    /// Modal panel window level (8)
    ModalPanel,
    /// Utility window level (19)
    Utility,
    /// Dock window level (20)
    Dock,
    /// Main menu window level (24)
    MainMenu,
    /// Status window level (25)
    Status,
    /// Pop-up menu window level (101)
    PopUpMenu,
    /// Screen saver window level (1000)
    ScreenSaver,
    /// Custom level value
    Custom(i32),
}

impl PanelLevel {
    /// Convert to the raw i64 value used by NSWindow
    pub fn value(&self) -> i64 {
        match self {
            PanelLevel::Normal => 0,
            PanelLevel::Submenu => 3,
            PanelLevel::TornOffMenu => 3,
            PanelLevel::Floating => 4,
            PanelLevel::ModalPanel => 8,
            PanelLevel::Utility => 19,
            PanelLevel::Dock => 20,
            PanelLevel::MainMenu => 24,
            PanelLevel::Status => 25,
            PanelLevel::PopUpMenu => 101,
            PanelLevel::ScreenSaver => 1000,
            PanelLevel::Custom(value) => *value as i64,
        }
    }
}

impl From<PanelLevel> for i64 {
    fn from(level: PanelLevel) -> Self {
        level.value()
    }
}

impl From<i32> for PanelLevel {
    fn from(value: i32) -> Self {
        PanelLevel::Custom(value)
    }
}

impl From<i64> for PanelLevel {
    fn from(value: i64) -> Self {
        PanelLevel::Custom(value as i32)
    }
}

/// Window collection behavior builder for NSPanel
///
/// Allows combining multiple collection behaviors using the builder pattern.
/// Collection behaviors control how a window participates in Spaces, Exposé, and fullscreen mode.
///
/// # Example
/// ```rust
/// use tauri_nspanel::CollectionBehavior;
///
/// // Create a panel that appears on all spaces and ignores Cmd+Tab cycling
/// let behavior = CollectionBehavior::new()
///     .can_join_all_spaces()
///     .ignores_cycle();
///
/// // Use with PanelBuilder
/// PanelBuilder::new(&app, "my-panel")
///     .collection_behavior(behavior)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollectionBehavior(objc2_app_kit::NSWindowCollectionBehavior);

impl CollectionBehavior {
    /// Create an empty collection behavior
    pub fn new() -> Self {
        Self(objc2_app_kit::NSWindowCollectionBehavior::empty())
    }

    /// Window can be shown on another space
    pub fn can_join_all_spaces(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::CanJoinAllSpaces;
        self
    }

    /// Window appears in all spaces
    pub fn move_to_active_space(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::MoveToActiveSpace;
        self
    }

    /// Window is managed by Spaces
    pub fn managed(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::Managed;
        self
    }

    /// Window participates in Spaces and Expose
    pub fn transient(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::Transient;
        self
    }

    /// Window does not participate in Spaces or Expose
    pub fn stationary(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::Stationary;
        self
    }

    /// Window participates in cycling
    pub fn participates_in_cycle(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::ParticipatesInCycle;
        self
    }

    /// Window ignores cycling commands
    pub fn ignores_cycle(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::IgnoresCycle;
        self
    }

    /// Window can be shown in full screen
    pub fn full_screen_primary(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::FullScreenPrimary;
        self
    }

    /// Window can be shown alongside full screen window
    pub fn full_screen_auxiliary(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::FullScreenAuxiliary;
        self
    }

    /// Window does not allow full screen
    pub fn full_screen_none(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::FullScreenNone;
        self
    }

    /// Window can be shown in full screen for this space only
    pub fn full_screen_allows_tiling(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::FullScreenAllowsTiling;
        self
    }

    /// Window does not allow full screen and hides on app deactivation
    pub fn full_screen_disallows_tiling(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowCollectionBehavior::FullScreenDisallowsTiling;
        self
    }

    /// Create from raw NSWindowCollectionBehavior flags
    pub fn from_raw(flags: objc2_app_kit::NSWindowCollectionBehavior) -> Self {
        Self(flags)
    }

    /// Get the raw NSWindowCollectionBehavior flags
    pub fn value(&self) -> objc2_app_kit::NSWindowCollectionBehavior {
        self.0
    }
}

impl Default for CollectionBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl From<CollectionBehavior> for objc2_app_kit::NSWindowCollectionBehavior {
    fn from(behavior: CollectionBehavior) -> Self {
        behavior.0
    }
}

impl From<objc2_app_kit::NSWindowCollectionBehavior> for CollectionBehavior {
    fn from(value: objc2_app_kit::NSWindowCollectionBehavior) -> Self {
        CollectionBehavior(value)
    }
}

/// Tracking area options builder for NSPanel
///
/// Allows combining multiple tracking area options using the builder pattern.
/// Tracking areas enable mouse event tracking within a specific region of a view.
///
/// # Example
/// ```rust
/// use tauri_nspanel::TrackingAreaOptions;
///
/// // Track mouse movement and enter/exit events, active in any application state
/// let options = TrackingAreaOptions::new()
///     .active_always()
///     .mouse_entered_and_exited()
///     .mouse_moved();
///
/// // Use with panel macro
/// panel!(MyPanel {
///     with: {
///         tracking_area: {
///             options: options,
///             auto_resize: true
///         }
///     }
/// });
///
/// // Or use with PanelBuilder
/// PanelBuilder::new(&app, "my-panel")
///     .tracking_area(options, true)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackingAreaOptions(objc2_app_kit::NSTrackingAreaOptions);

impl TrackingAreaOptions {
    /// Create empty tracking area options
    pub fn new() -> Self {
        Self(objc2_app_kit::NSTrackingAreaOptions::empty())
    }

    /// Track mouse moved events
    pub fn mouse_moved(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::MouseMoved;
        self
    }

    /// Track mouse entered and exited events
    pub fn mouse_entered_and_exited(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::MouseEnteredAndExited;
        self
    }

    /// Track when mouse is active in any application
    pub fn active_always(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::ActiveAlways;
        self
    }

    /// Track when mouse is active in this application
    pub fn active_in_active_app(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::ActiveInActiveApp;
        self
    }

    /// Track when mouse is active in key window
    pub fn active_in_key_window(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::ActiveInKeyWindow;
        self
    }

    /// Track when window is key
    pub fn active_when_first_responder(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::ActiveWhenFirstResponder;
        self
    }

    /// Assumes tracking area is active
    pub fn assume_inside(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::AssumeInside;
        self
    }

    /// Tracking area is in visibleRect coordinates
    pub fn in_visible_rect(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::InVisibleRect;
        self
    }

    /// Enable cursor update events
    pub fn cursor_update(mut self) -> Self {
        self.0 |= objc2_app_kit::NSTrackingAreaOptions::CursorUpdate;
        self
    }

    /// Create from raw NSTrackingAreaOptions flags
    pub fn from_raw(flags: objc2_app_kit::NSTrackingAreaOptions) -> Self {
        Self(flags)
    }

    /// Get the raw NSTrackingAreaOptions flags
    pub fn value(&self) -> objc2_app_kit::NSTrackingAreaOptions {
        self.0
    }
}

impl Default for TrackingAreaOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl From<TrackingAreaOptions> for objc2_app_kit::NSTrackingAreaOptions {
    fn from(options: TrackingAreaOptions) -> Self {
        options.0
    }
}

impl From<objc2_app_kit::NSTrackingAreaOptions> for TrackingAreaOptions {
    fn from(value: objc2_app_kit::NSTrackingAreaOptions) -> Self {
        TrackingAreaOptions(value)
    }
}

/// Window style mask builder for NSPanel
///
/// Allows combining multiple style masks using the builder pattern.
/// Style masks control the appearance and behavior of the window frame.
///
/// # Example
/// ```rust
/// use tauri_nspanel::StyleMask;
///
/// // Create a borderless panel that doesn't activate the app
/// let style = StyleMask::new()
///     .borderless()
///     .nonactivating_panel();
///
/// // Use with PanelBuilder
/// PanelBuilder::new(&app, "my-panel")
///     .style_mask(style)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleMask(objc2_app_kit::NSWindowStyleMask);

impl StyleMask {
    /// Create with default style mask (Titled | Closable | Miniaturizable | Resizable)
    pub fn new() -> Self {
        Self(
            objc2_app_kit::NSWindowStyleMask::Titled
                | objc2_app_kit::NSWindowStyleMask::Closable
                | objc2_app_kit::NSWindowStyleMask::Miniaturizable
                | objc2_app_kit::NSWindowStyleMask::Resizable,
        )
    }

    /// Create an empty style mask
    pub fn empty() -> Self {
        Self(objc2_app_kit::NSWindowStyleMask::empty())
    }

    /// Window has a title bar
    pub fn titled(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::Titled;
        self
    }

    /// Window has a close button
    pub fn closable(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::Closable;
        self
    }

    /// Window has a minimize button
    pub fn miniaturizable(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::Miniaturizable;
        self
    }

    /// Window can be resized
    pub fn resizable(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::Resizable;
        self
    }

    /// Window uses unified title and toolbar
    pub fn unified_title_and_toolbar(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::UnifiedTitleAndToolbar;
        self
    }

    /// Window uses full size content view
    pub fn full_size_content_view(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::FullSizeContentView;
        self
    }

    /// Window is a utility window
    pub fn utility_window(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::UtilityWindow;
        self
    }

    /// Window is a HUD window
    pub fn hud_window(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::HUDWindow;
        self
    }

    /// Window is a non-activating panel
    pub fn nonactivating_panel(mut self) -> Self {
        self.0 |= objc2_app_kit::NSWindowStyleMask::NonactivatingPanel;
        self
    }

    /// Window has no title bar or border
    pub fn borderless(mut self) -> Self {
        self.0 = objc2_app_kit::NSWindowStyleMask::Borderless;
        self
    }

    /// Create from raw NSWindowStyleMask flags
    pub fn from_raw(flags: objc2_app_kit::NSWindowStyleMask) -> Self {
        Self(flags)
    }

    /// Get the raw NSWindowStyleMask flags
    pub fn value(&self) -> objc2_app_kit::NSWindowStyleMask {
        self.0
    }
}

impl Default for StyleMask {
    fn default() -> Self {
        Self::new()
    }
}

impl From<StyleMask> for objc2_app_kit::NSWindowStyleMask {
    fn from(mask: StyleMask) -> Self {
        mask.0
    }
}

impl From<objc2_app_kit::NSWindowStyleMask> for StyleMask {
    fn from(value: objc2_app_kit::NSWindowStyleMask) -> Self {
        StyleMask(value)
    }
}

#[derive(Default)]
pub(crate) struct PanelConfig {
    pub floating: Option<bool>,
    pub level: Option<PanelLevel>,
    pub has_shadow: Option<bool>,
    pub opaque: Option<bool>,
    pub alpha_value: Option<f64>,
    pub hides_on_deactivate: Option<bool>,
    pub becomes_key_only_if_needed: Option<bool>,
    pub accepts_mouse_moved_events: Option<bool>,
    pub ignores_mouse_events: Option<bool>,
    pub movable_by_window_background: Option<bool>,
    pub released_when_closed: Option<bool>,
    pub works_when_modal: Option<bool>,
    // pub content_size: Option<Size>,
    pub style_mask: Option<StyleMask>,
    pub collection_behavior: Option<CollectionBehavior>,
    pub no_activate: Option<bool>,
}
