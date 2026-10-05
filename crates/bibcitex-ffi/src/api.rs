use crate::records::{CoreError, LibraryRecord, ReferenceRecord};
#[uniffi::export]
pub fn initialize() {
    bibcitex_service::initialize();
}
#[uniffi::export]
pub fn libraries() -> Result<Vec<LibraryRecord>, CoreError> {
    bibcitex_service::libraries()
}
#[uniffi::export]
pub fn add_library(
    name: String,
    path: String,
    description: Option<String>,
) -> Result<LibraryRecord, CoreError> {
    bibcitex_service::add_library(name, path, description)
}
#[uniffi::export]
pub fn update_library(
    name: String,
    new_name: String,
    path: Option<String>,
    description: Option<String>,
) -> Result<LibraryRecord, CoreError> {
    bibcitex_service::update_library(name, new_name, path, description)
}
#[uniffi::export]
pub fn set_library_pinned(name: String, pinned: bool) -> Result<(), CoreError> {
    bibcitex_service::set_library_pinned(name, pinned)
}
#[uniffi::export]
pub fn remove_library(name: String) -> Result<(), CoreError> {
    bibcitex_service::remove_library(name)
}
#[uniffi::export]
pub fn search(
    path: String,
    query: String,
    field: String,
    type_filter: String,
) -> Result<Vec<ReferenceRecord>, CoreError> {
    bibcitex_service::search(path, query, field, type_filter)
}
#[uniffi::export]
pub fn helper_current() -> Result<Option<LibraryRecord>, CoreError> {
    bibcitex_service::helper_current()
}
#[uniffi::export]
pub fn helper_select(name: String, path: String) -> Result<LibraryRecord, CoreError> {
    bibcitex_service::helper_select(name, path)
}
#[uniffi::export]
pub fn capture_paste_target() -> Result<(), CoreError> {
    bibcitex_service::capture_paste_target()
}
#[uniffi::export]
pub fn copy(text: String) -> Result<(), CoreError> {
    bibcitex_service::copy(text)
}
#[uniffi::export]
pub fn paste(text: String) -> Result<(), CoreError> {
    bibcitex_service::paste(text)
}
