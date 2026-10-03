//! Shared desktop operations used by platform-specific binding libraries.
pub use bibcitex_core::core;
mod api;
mod records;
mod registry;
pub use api::*;
pub use records::*;
