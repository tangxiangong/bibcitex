use super::{BridgeError, TextRequest, invoke, request};
use bibcitex_service::*;
use interoptopus::{ffi, wire::Wire};
/// Path and exact cite key for a metadata snapshot; an empty path requests the global catalog.
#[ffi]
pub struct TagSnapshotRequest {
    /// Stable bibliography identity for assignment reads.
    pub library_uid: String,
    /// Stable record identity for assignment reads.
    pub record_uid: String,
    /// Bibliography path; empty requests the global catalog.
    pub path: String,
    /// Original case-sensitive cite key for single-record assignments.
    pub cite_key: String,
}
/// Whether a result still refers to the current file and metadata revisions.
#[ffi]
pub struct ContextValidity {
    /// True only while file and metadata versions still match the result.
    pub valid: bool,
}
/// Owned typed result for AdvancedSearchResponse; operation failures use error, query diagnostics remain in the payload.
#[ffi]
pub struct AdvancedSearchResponseResult {
    /// Owned operation result; absent when the operation failed.
    pub payload: Option<AdvancedSearchResponse>,
    /// Storage or request failure; query diagnostics remain inside a successful payload.
    pub error: Option<String>,
}
/// Owned typed result for QueryAnalysis; operation failures use error, query diagnostics remain in the payload.
#[ffi]
pub struct QueryAnalysisResult {
    /// Owned operation result; absent when the operation failed.
    pub payload: Option<QueryAnalysis>,
    /// Storage or request failure; query diagnostics remain inside a successful payload.
    pub error: Option<String>,
}
/// Owned typed result for SearchCatalog; operation failures use error, query diagnostics remain in the payload.
#[ffi]
pub struct SearchCatalogResult {
    /// Owned operation result; absent when the operation failed.
    pub payload: Option<SearchCatalog>,
    /// Storage or request failure; query diagnostics remain inside a successful payload.
    pub error: Option<String>,
}
/// Owned typed result for TagSnapshot; operation failures use error, query diagnostics remain in the payload.
#[ffi]
pub struct TagSnapshotResult {
    /// Owned operation result; absent when the operation failed.
    pub payload: Option<TagSnapshot>,
    /// Storage or request failure; query diagnostics remain inside a successful payload.
    pub error: Option<String>,
}
/// Owned typed result for ContextValidity; operation failures use error, query diagnostics remain in the payload.
#[ffi]
pub struct ContextValidityResult {
    /// Owned operation result; absent when the operation failed.
    pub payload: Option<ContextValidity>,
    /// Storage or request failure; query diagnostics remain inside a successful payload.
    pub error: Option<String>,
}
/// Generated-wire entry point for the shared `advanced_search` operation.
#[ffi]
pub fn advanced_search(
    wire: Wire<AdvancedSearchRequest>,
) -> ffi::Result<Wire<AdvancedSearchResponseResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(match bibcitex_service::advanced_search(input) {
            Ok(payload) => AdvancedSearchResponseResult {
                payload: Some(payload),
                error: None,
            },
            Err(e) => AdvancedSearchResponseResult {
                payload: None,
                error: Some(e.to_string()),
            },
        })
    })
}
/// Generated-wire entry point for the shared `analyze_query` operation.
#[ffi]
pub fn analyze_query(
    wire: Wire<TextRequest>,
) -> ffi::Result<Wire<QueryAnalysisResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(QueryAnalysisResult {
            payload: Some(bibcitex_service::analyze_query(input.text)),
            error: None,
        })
    })
}
/// Generated-wire entry point for the shared `render_query` operation.
#[ffi]
pub fn render_query(
    wire: Wire<QueryDocumentRecord>,
) -> ffi::Result<Wire<QueryAnalysisResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(QueryAnalysisResult {
            payload: Some(bibcitex_service::render_query(input)),
            error: None,
        })
    })
}
/// Generated-wire entry point for the shared `edit_query` operation.
#[ffi]
pub fn edit_query(wire: Wire<QueryEdit>) -> ffi::Result<Wire<QueryAnalysisResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(QueryAnalysisResult {
            payload: Some(bibcitex_service::edit_query(input)),
            error: None,
        })
    })
}
/// Generated-wire entry point for the shared `build_condition` operation.
#[ffi]
pub fn build_condition(
    wire: Wire<QueryCondition>,
) -> ffi::Result<Wire<QueryAnalysisResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(QueryAnalysisResult {
            payload: Some(bibcitex_service::build_condition(input)),
            error: None,
        })
    })
}
/// Generated-wire entry point for the shared `refresh_query` operation.
#[ffi]
pub fn refresh_query(
    wire: Wire<QueryDocumentRecord>,
) -> ffi::Result<Wire<QueryAnalysisResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(match bibcitex_service::refresh_query(input) {
            Ok(payload) => QueryAnalysisResult {
                payload: Some(payload),
                error: None,
            },
            Err(e) => QueryAnalysisResult {
                payload: None,
                error: Some(e.to_string()),
            },
        })
    })
}
/// Generated-wire entry point for the shared `edit_query_text` operation.
#[ffi]
pub fn edit_query_text(
    wire: Wire<QueryTextEdit>,
) -> ffi::Result<Wire<QueryAnalysisResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(QueryAnalysisResult {
            payload: Some(bibcitex_service::edit_query_text(input)),
            error: None,
        })
    })
}
/// Generated-wire entry point for the shared `search_catalog` operation.
#[ffi]
pub fn search_catalog() -> ffi::Result<Wire<SearchCatalogResult>, BridgeError> {
    invoke(|| {
        Ok(SearchCatalogResult {
            payload: Some(bibcitex_service::search_catalog()),
            error: None,
        })
    })
}
/// Generated-wire entry point for the shared `tag_snapshot` operation.
#[ffi]
pub fn tag_snapshot(
    wire: Wire<TagSnapshotRequest>,
) -> ffi::Result<Wire<TagSnapshotResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(
            match if input.library_uid.is_empty() {
                bibcitex_service::tag_snapshot(input.path, input.cite_key)
            } else {
                bibcitex_service::tag_snapshot_uid(input.library_uid, input.record_uid)
            } {
                Ok(payload) => TagSnapshotResult {
                    payload: Some(payload),
                    error: None,
                },
                Err(e) => TagSnapshotResult {
                    payload: None,
                    error: Some(e.to_string()),
                },
            },
        )
    })
}
/// Generated-wire entry point for the shared `mutate_tag` operation.
#[ffi]
pub fn mutate_tag(wire: Wire<TagMutation>) -> ffi::Result<Wire<TagSnapshotResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(match bibcitex_service::mutate_tag(input) {
            Ok(payload) => TagSnapshotResult {
                payload: Some(payload),
                error: None,
            },
            Err(e) => TagSnapshotResult {
                payload: None,
                error: Some(e.to_string()),
            },
        })
    })
}
/// Generated-wire entry point for the shared `validate_search_context` operation.
#[ffi]
pub fn validate_search_context(
    wire: Wire<SearchContext>,
) -> ffi::Result<Wire<ContextValidityResult>, BridgeError> {
    invoke(|| {
        let input = request(wire)?;
        Ok(
            match bibcitex_service::validate_search_context(input)
                .map(|valid| ContextValidity { valid })
            {
                Ok(payload) => ContextValidityResult {
                    payload: Some(payload),
                    error: None,
                },
                Err(e) => ContextValidityResult {
                    payload: None,
                    error: Some(e.to_string()),
                },
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_query_diagnostics_and_bound_tag_arenas_roundtrip_without_json() {
        let result = analyze_query(Wire::from(TextRequest {
            text: "year:abc author:".into(),
        }))
        .unwrap()
        .unwire();
        assert!(result.error.is_none());
        let payload = result.payload.unwrap();
        assert_eq!(payload.status, QueryStatus::Invalid);
        assert!(!payload.diagnostics.is_empty());
        for _ in 0..64 {
            let built = build_condition(Wire::from(QueryCondition {
                field_name: "tag".into(),
                text_value: "machine learning".into(),
                phrase: false,
                exclude: true,
                tag_id: Some("stable-tag-id".into()),
            }))
            .unwrap()
            .unwire()
            .payload
            .unwrap();
            assert_eq!(built.status, QueryStatus::Valid);
            let doc = built.document.unwrap();
            let output = render_query(Wire::from(doc))
                .unwrap()
                .unwire()
                .payload
                .unwrap();
            assert!(
                output
                    .document
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|n| n.tag_id.as_deref() == Some("stable-tag-id"))
            );
        }
    }
    #[test]
    fn catalog_and_source_records_preserve_optional_values_and_ownership() {
        let snapshot = TagSnapshot {
            library_uid: String::new(),
            record_uid: String::new(),
            revision: 42,
            tags: vec![TagRecord {
                color: "#64748B".into(),
                id: "id".into(),
                name: "Straße".into(),
            }],
            assigned_ids: vec!["id".into()],
            pending: vec![PendingAssociation {
                record_id: "record".into(),
                source_id: "source".into(),
                path: "/tmp/reference.bib".into(),
                cite_key: "Key".into(),
                tags: vec!["id".into()],
            }],
            sources: vec![SearchSourceRecord {
                id: "source".into(),
                path: "/tmp/reference.bib".into(),
                available: false,
            }],
        };
        let copied = Wire::from(snapshot).unwire();
        assert_eq!(copied.tags[0].name, "Straße");
        assert!(!copied.sources[0].available);
        assert_eq!(copied.pending[0].cite_key, "Key");
    }
}
