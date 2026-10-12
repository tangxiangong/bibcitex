use crate::records::CoreError;
use crate::search_records::*;
/// Typed Swift entry point for `bibcitex_service::advanced_search`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn advanced_search(
    request: AdvancedSearchRequest,
) -> Result<AdvancedSearchResponse, CoreError> {
    bibcitex_service::advanced_search(request)
}
/// Typed Swift entry point for `bibcitex_service::analyze_query`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn analyze_query(text: String) -> QueryAnalysis {
    bibcitex_service::analyze_query(text)
}
/// Typed Swift entry point for `bibcitex_service::render_query`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn render_query(document: QueryDocumentRecord) -> QueryAnalysis {
    bibcitex_service::render_query(document)
}
/// Typed Swift entry point for `bibcitex_service::edit_query`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn edit_query(edit: QueryEdit) -> QueryAnalysis {
    bibcitex_service::edit_query(edit)
}
/// Typed Swift entry point for `bibcitex_service::build_condition`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn build_condition(condition: QueryCondition) -> QueryAnalysis {
    bibcitex_service::build_condition(condition)
}
/// Typed Swift entry point for `bibcitex_service::refresh_query`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn refresh_query(document: QueryDocumentRecord) -> Result<QueryAnalysis, CoreError> {
    bibcitex_service::refresh_query(document)
}
/// Typed Swift entry point for `bibcitex_service::edit_query_text`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn edit_query_text(edit: QueryTextEdit) -> QueryAnalysis {
    bibcitex_service::edit_query_text(edit)
}
/// Typed Swift entry point for `bibcitex_service::search_catalog`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn search_catalog() -> SearchCatalog {
    bibcitex_service::search_catalog()
}
/// Typed Swift entry point for `bibcitex_service::tag_snapshot`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn tag_snapshot(path: String, cite_key: String) -> Result<TagSnapshot, CoreError> {
    bibcitex_service::tag_snapshot(path, cite_key)
}
/// Typed Swift entry point for `bibcitex_service::mutate_tag`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn mutate_tag(mutation: TagMutation) -> Result<TagSnapshot, CoreError> {
    bibcitex_service::mutate_tag(mutation)
}
/// Typed Swift entry point for `bibcitex_service::validate_search_context`; preserves shared validation and diagnostics.
#[uniffi::export]
pub fn validate_search_context(context: SearchContext) -> Result<bool, CoreError> {
    bibcitex_service::validate_search_context(context)
}

/// Read tag assignments using stable identities.
#[uniffi::export]
pub fn tag_snapshot_uid(library_uid: String, record_uid: String) -> Result<TagSnapshot, CoreError> {
    bibcitex_service::tag_snapshot_uid(library_uid, record_uid)
}
