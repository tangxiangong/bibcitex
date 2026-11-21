//! Library registration shared by every desktop surface.
use super::events;
use bibcitex_service as service;
use nsh_core::{element::blocking, prelude::*};

/// A registered bibliography and its current availability on disk.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct LibraryDto {
    /// Stable identifier across renames and relocations.
    pub uid: String,
    /// User-facing library name.
    pub name: String,
    /// Registered bibliography path.
    pub path: String,
    /// Registration timestamp from the shared registry.
    pub created_at: String,
    /// Last change timestamp from the shared registry.
    pub updated_at: String,
    /// User-supplied library description.
    pub description: Option<String>,
    /// Whether the library is pinned.
    pub pinned: bool,
    /// Whether the file existed when this record was read.
    pub available: bool,
}
impl From<service::LibraryRecord> for LibraryDto {
    fn from(record: service::LibraryRecord) -> Self {
        Self {
            uid: record.uid,
            name: record.name,
            path: record.path,
            created_at: record.created_at,
            updated_at: record.updated_at,
            description: record.description,
            pinned: record.pinned,
            available: record.available,
        }
    }
}

async fn registry<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, service::CoreError> + Send + 'static,
) -> Result<T, String> {
    blocking(operation)
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

async fn change<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, service::CoreError> + Send + 'static,
) -> Result<T, String> {
    let value = registry(operation).await?;
    events::emit("libraries");
    Ok(value)
}

/// Bibliography registry and persisted helper library selection.
#[module]
pub struct Catalog;

#[module]
impl Catalog {
    /// Lists libraries in registry order, with pins and file availability.
    pub async fn libraries() -> Result<Vec<LibraryDto>, String> {
        Ok(registry(service::libraries)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    /// Registers a bibliography at its original path.
    pub async fn add(
        name: String,
        path: String,
        description: Option<String>,
    ) -> Result<LibraryDto, String> {
        change(move || service::add_library(name, path, description))
            .await
            .map(Into::into)
    }
    /// Renames a library and updates its optional path and description.
    pub async fn update(
        old_name: String,
        new_name: String,
        path: Option<String>,
        description: Option<String>,
    ) -> Result<LibraryDto, String> {
        change(move || service::update_library(old_name, new_name, path, description))
            .await
            .map(Into::into)
    }
    /// Changes whether a library appears in the pinned group.
    pub async fn set_pinned(name: String, pinned: bool) -> Result<(), String> {
        change(move || service::set_library_pinned(name, pinned)).await
    }
    /// Removes a registration while preserving the bibliography file.
    pub async fn remove(name: String) -> Result<(), String> {
        change(move || service::remove_library(name)).await
    }
    /// Reads the compact helper's currently selected library.
    pub async fn helper_current() -> Result<Option<LibraryDto>, String> {
        Ok(registry(service::helper_current).await?.map(Into::into))
    }
    /// Selects the named library in the helper after verifying its registered path.
    pub async fn helper_select(name: String, path: String) -> Result<LibraryDto, String> {
        change(move || service::helper_select(name, path))
            .await
            .map(Into::into)
    }
}
