//! Swift bindings over the shared desktop service.
mod api;
mod records;
pub use api::*;
pub use records::*;
uniffi::setup_scaffolding!();
