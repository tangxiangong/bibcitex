//! Windows binding declarations. Interoptopus owns the ABI and wire encoding;
//! all application operations and records remain in `bibcitex-service`.

use bibcitex_service::{CoreError, LibraryRecord, ReferenceRecord};
use interoptopus::inventory::RustInventory;
use interoptopus::lang::types::{TypeInfo, WireIO};
use interoptopus::pattern::result::panic_to_result;
use interoptopus::wire::Wire;
use interoptopus::{builtins_wire, ffi, function};

#[ffi]
pub enum BridgeError {
    Serialization,
}

#[ffi]
pub struct AddLibraryRequest {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

#[ffi]
pub struct UpdateLibraryRequest {
    pub name: String,
    pub new_name: String,
    pub path: Option<String>,
    pub description: Option<String>,
}
#[ffi]
pub struct PinLibraryRequest {
    pub name: String,
    pub pinned: bool,
}
#[ffi]
pub struct NameRequest {
    pub name: String,
}

#[ffi]
pub struct SearchRequest {
    pub path: String,
    pub query: String,
    pub search_field: String,
    pub type_filter: String,
}

#[ffi]
pub struct SelectLibraryRequest {
    pub name: String,
    pub path: String,
}

#[ffi]
pub struct TextRequest {
    pub text: String,
}

#[ffi]
pub struct StatusResponse {
    pub error: Option<String>,
}

#[ffi]
pub struct LibrariesResponse {
    pub payload: Vec<LibraryRecord>,
    pub error: Option<String>,
}

#[ffi]
pub struct LibraryResponse {
    pub payload: Option<LibraryRecord>,
    pub error: Option<String>,
}

#[ffi]
pub struct SearchResponse {
    pub payload: Vec<ReferenceRecord>,
    pub error: Option<String>,
}

fn invoke<T: TypeInfo + WireIO>(
    work: impl FnOnce() -> Result<T, BridgeError>,
) -> ffi::Result<Wire<T>, BridgeError> {
    // Use the library's panic boundary; no Rust unwind reaches the CLR.
    panic_to_result(|| {
        work()
            .and_then(|value| Wire::try_from(value).map_err(|_| BridgeError::Serialization))
            .into()
    })
}

fn request<T: TypeInfo + WireIO>(mut wire: Wire<T>) -> Result<T, BridgeError> {
    wire.try_unwire().map_err(|_| BridgeError::Serialization)
}

fn status(result: Result<(), CoreError>) -> StatusResponse {
    StatusResponse {
        error: result.err().map(|error| error.to_string()),
    }
}

fn library(result: Result<Option<LibraryRecord>, CoreError>) -> LibraryResponse {
    match result {
        Ok(value) => LibraryResponse {
            payload: value,
            error: None,
        },
        Err(error) => LibraryResponse {
            payload: None,
            error: Some(error.to_string()),
        },
    }
}

#[ffi]
pub fn initialize() -> ffi::Result<Wire<StatusResponse>, BridgeError> {
    invoke(|| {
        bibcitex_service::initialize();
        Ok(StatusResponse { error: None })
    })
}

#[ffi]
pub fn libraries() -> ffi::Result<Wire<LibrariesResponse>, BridgeError> {
    invoke(|| {
        Ok(match bibcitex_service::libraries() {
            Ok(value) => LibrariesResponse {
                payload: value,
                error: None,
            },
            Err(error) => LibrariesResponse {
                payload: Vec::new(),
                error: Some(error.to_string()),
            },
        })
    })
}

#[ffi]
pub fn add_library(
    input: Wire<AddLibraryRequest>,
) -> ffi::Result<Wire<LibraryResponse>, BridgeError> {
    invoke(|| {
        let input = request(input)?;
        Ok(library(
            bibcitex_service::add_library(input.name, input.path, input.description).map(Some),
        ))
    })
}

#[ffi]
pub fn update_library(
    input: Wire<UpdateLibraryRequest>,
) -> ffi::Result<Wire<LibraryResponse>, BridgeError> {
    invoke(|| {
        let input = request(input)?;
        Ok(library(
            bibcitex_service::update_library(
                input.name,
                input.new_name,
                input.path,
                input.description,
            )
            .map(Some),
        ))
    })
}
#[ffi]
pub fn set_library_pinned(
    input: Wire<PinLibraryRequest>,
) -> ffi::Result<Wire<StatusResponse>, BridgeError> {
    invoke(|| {
        let input = request(input)?;
        Ok(status(bibcitex_service::set_library_pinned(
            input.name,
            input.pinned,
        )))
    })
}
#[ffi]
pub fn remove_library(input: Wire<NameRequest>) -> ffi::Result<Wire<StatusResponse>, BridgeError> {
    invoke(|| {
        Ok(status(bibcitex_service::remove_library(
            request(input)?.name,
        )))
    })
}

#[ffi]
pub fn search(input: Wire<SearchRequest>) -> ffi::Result<Wire<SearchResponse>, BridgeError> {
    invoke(|| {
        let input = request(input)?;
        Ok(
            match bibcitex_service::search(
                input.path,
                input.query,
                input.search_field,
                input.type_filter,
            ) {
                Ok(value) => SearchResponse {
                    payload: value,
                    error: None,
                },
                Err(error) => SearchResponse {
                    payload: Vec::new(),
                    error: Some(error.to_string()),
                },
            },
        )
    })
}

#[ffi]
pub fn helper_current() -> ffi::Result<Wire<LibraryResponse>, BridgeError> {
    invoke(|| Ok(library(bibcitex_service::helper_current())))
}

#[ffi]
pub fn helper_select(
    input: Wire<SelectLibraryRequest>,
) -> ffi::Result<Wire<LibraryResponse>, BridgeError> {
    invoke(|| {
        let input = request(input)?;
        Ok(library(
            bibcitex_service::helper_select(input.name, input.path).map(Some),
        ))
    })
}

#[ffi]
pub fn capture_paste_target() -> ffi::Result<Wire<StatusResponse>, BridgeError> {
    invoke(|| Ok(status(bibcitex_service::capture_paste_target())))
}

#[ffi]
pub fn copy(input: Wire<TextRequest>) -> ffi::Result<Wire<StatusResponse>, BridgeError> {
    invoke(|| Ok(status(bibcitex_service::copy(request(input)?.text))))
}

#[ffi]
pub fn paste(input: Wire<TextRequest>) -> ffi::Result<Wire<StatusResponse>, BridgeError> {
    invoke(|| Ok(status(bibcitex_service::paste(request(input)?.text))))
}

pub fn inventory() -> RustInventory {
    RustInventory::new()
        .register(builtins_wire!())
        .register(function!(initialize))
        .register(function!(libraries))
        .register(function!(add_library))
        .register(function!(remove_library))
        .register(function!(update_library))
        .register(function!(set_library_pinned))
        .register(function!(search))
        .register(function!(helper_current))
        .register(function!(helper_select))
        .register(function!(capture_paste_target))
        .register(function!(copy))
        .register(function!(paste))
        .validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bibcitex_service::ChunkKind;

    #[test]
    fn wire_search_preserves_unicode_math_and_errors() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("文献.bib");
        std::fs::write(
            &path,
            "@article{example,title={中文 $x^2$ result},author={Doe, Jane},year={2026}}",
        )
        .unwrap();
        let make_request = |field: &str| {
            Wire::from(SearchRequest {
                path: path.to_string_lossy().into_owned(),
                query: String::new(),
                search_field: field.to_owned(),
                type_filter: "all".into(),
            })
        };
        let response = search(make_request("all")).unwrap().unwire();
        assert!(response.error.is_none());
        assert_eq!(response.payload.len(), 1);
        let reference = &response.payload[0];
        assert_eq!(reference.cite_key, "example");
        assert_eq!(reference.year, Some(2026));
        assert!(reference.pages.is_none());
        assert!(
            reference
                .title
                .iter()
                .any(|chunk| chunk.text.contains("中文"))
        );
        assert!(
            reference
                .title
                .iter()
                .any(|chunk| { matches!(chunk.kind, ChunkKind::Math) && chunk.text == "x^2" })
        );

        let invalid = search(make_request("not-a-field")).unwrap().unwire();
        assert!(invalid.payload.is_empty());
        assert_eq!(invalid.error.as_deref(), Some("Unknown search field"));
    }

    #[test]
    fn library_panic_boundary_returns_a_result() {
        let result = invoke::<StatusResponse>(|| panic!("boundary regression"));
        assert!(matches!(result, ffi::Result::Panic));
    }
}
