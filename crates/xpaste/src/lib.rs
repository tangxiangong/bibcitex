//! Copyright (c) EcoPasteHub
//!
//! License: Apache-2.0
//!
//! Modified by tangxiangong (2025) for [bibcitex](https://github.com/tangxiangong/bibcitex).
//!
//! # Note
//!
//! This crate is forked from the [EcoPasteHub/EcoPaste](https://github.com/EcoPasteHub/EcoPaste), which is licensed under [Apache 2.0](https://github.com/EcoPasteHub/EcoPaste/blob/master/LICENSE).

pub(crate) static MAIN_WINDOW_TITLE: &str = "BibCiTeX";

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::{focus_previous_window, observe_app};
#[cfg(target_os = "windows")]
pub use windows::{focus_previous_window, observe_app};
