//! Query editor operations shared by all desktop surfaces.
use bibcitex_service as svc;
use nsh_core::{element::blocking, prelude::*};

#[path = "query_types.rs"]
mod types;
pub use types::*;

// A query may have hundreds of spans. Index its character boundaries once
// instead of scanning the query from its start for every node.
pub(super) struct Offsets(Vec<(usize, usize)>);

impl Offsets {
    pub(super) fn new(text: &str) -> Self {
        let mut units = 0;
        let boundaries = text.char_indices().map(|(byte, ch)| {
            let entry = (byte, units);
            units += ch.len_utf16();
            entry
        });
        let mut offsets: Vec<_> = boundaries.collect();
        offsets.push((text.len(), units));
        Self(offsets)
    }

    fn byte(&self, utf16: u32) -> Result<u32, String> {
        let index = self
            .0
            .binary_search_by_key(&(utf16 as usize), |&(_, units)| units)
            .map_err(|_| "InvalidEditRange".to_owned())?;
        u32::try_from(self.0[index].0).map_err(|_| "InvalidEditRange".to_owned())
    }

    pub(super) fn units(&self, byte: u32) -> Result<u32, String> {
        let index = self
            .0
            .binary_search_by_key(&(byte as usize), |&(byte, _)| byte)
            .map_err(|_| "InvalidUtf8Boundary".to_owned())?;
        u32::try_from(self.0[index].1).map_err(|_| "InvalidUtf8Boundary".to_owned())
    }
}

impl QueryDocumentDto {
    fn into_record(self, offsets: &Offsets) -> Result<svc::QueryDocumentRecord, String> {
        let nodes = self
            .nodes
            .into_iter()
            .map(|node| {
                if node.start > node.end {
                    return Err("InvalidEditRange".into());
                }
                Ok(svc::QueryNodeRecord {
                    kind: node.kind.into(),
                    text_value: node.text_value,
                    tag_id: node.tag_id,
                    children: node.children,
                    start: offsets.byte(node.start)?,
                    end: offsets.byte(node.end)?,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(svc::QueryDocumentRecord {
            nodes,
            root: self.root,
        })
    }
}

impl TryFrom<QueryDocumentDto> for svc::QueryDocumentRecord {
    type Error = String;
    fn try_from(doc: QueryDocumentDto) -> Result<Self, String> {
        let offsets = Offsets::new(&doc.text);
        doc.into_record(&offsets)
    }
}

impl TryFrom<QueryTextEditDto> for svc::QueryTextEdit {
    type Error = String;
    fn try_from(edit: QueryTextEditDto) -> Result<Self, String> {
        if edit.document.text != edit.text || edit.start > edit.end {
            return Err("InvalidEditRange".into());
        }
        let offsets = Offsets::new(&edit.text);
        Ok(Self {
            document: edit.document.into_record(&offsets)?,
            start: offsets.byte(edit.start)?,
            end: offsets.byte(edit.end)?,
            replacement: edit.replacement,
            full_replace: edit.full_replace,
            text: edit.text,
        })
    }
}

impl TryFrom<svc::QueryAnalysis> for QueryAnalysisDto {
    type Error = String;
    fn try_from(value: svc::QueryAnalysis) -> Result<Self, String> {
        let offsets = Offsets::new(&value.text);
        let diagnostics = value
            .diagnostics
            .into_iter()
            .map(|d| {
                Ok(QueryDiagnosticDto {
                    code: d.code,
                    start: offsets.units(d.start)?,
                    end: offsets.units(d.end)?,
                    warning: d.warning,
                })
            })
            .collect::<Result<_, String>>()?;
        let document = value
            .document
            .map(|doc| {
                let nodes = doc
                    .nodes
                    .into_iter()
                    .map(|node| {
                        Ok(QueryNodeDto {
                            kind: node.kind.into(),
                            text_value: node.text_value,
                            tag_id: node.tag_id,
                            children: node.children,
                            start: offsets.units(node.start)?,
                            end: offsets.units(node.end)?,
                        })
                    })
                    .collect::<Result<_, String>>()?;
                Ok::<_, String>(QueryDocumentDto {
                    text: value.text.clone(),
                    root: doc.root,
                    nodes,
                })
            })
            .transpose()?;
        Ok(Self {
            status: value.status.into(),
            document,
            diagnostics,
            text: value.text,
            selection: None,
            tag_revision: value.tag_revision.map(|n| n.to_string()),
        })
    }
}

// On invalid/incomplete input the service keeps the old node arena as binding
// anchors. It shifts untouched nodes, but clears the binding on an overlapping
// node without rewriting that node's span. Project only those spans into the
// replacement text before handing the retained arena back to the editor.
fn edit_text(input: QueryTextEditDto) -> Result<QueryAnalysisDto, String> {
    let inserted_units =
        u32::try_from(input.replacement.encode_utf16().count()).map_err(|_| "ByteLimitExceeded")?;
    let caret = input
        .start
        .checked_add(inserted_units)
        .ok_or("ByteLimitExceeded")?;
    let edit: svc::QueryTextEdit = input.try_into()?;
    let mut touched = Vec::new();
    if !edit.full_replace {
        let replacement_len =
            u32::try_from(edit.replacement.len()).map_err(|_| "ByteLimitExceeded")?;
        for (index, node) in edit.document.nodes.iter().enumerate() {
            if node.end > edit.start && node.start < edit.end {
                let start = node.start.min(edit.start);
                let end = node.end.max(edit.end) - (edit.end - edit.start);
                touched.push((
                    index,
                    start,
                    end.checked_add(replacement_len)
                        .ok_or("ByteLimitExceeded")?,
                ));
            }
        }
    }
    let expected_len = edit.text.len() - (edit.end - edit.start) as usize + edit.replacement.len();
    let mut result = svc::edit_query_text(edit);
    if matches!(
        result.status,
        svc::QueryStatus::Invalid | svc::QueryStatus::Incomplete
    ) && let Some(anchors) = &mut result.document
    {
        for (index, start, end) in touched {
            let node = &mut anchors.nodes[index];
            node.start = start;
            node.end = end;
        }
    }
    let mut analysis = QueryAnalysisDto::try_from(result)?;
    if analysis.text.len() == expected_len {
        analysis.selection = Some(QuerySelectionDto {
            start: caret,
            end: caret,
        });
    }
    Ok(analysis)
}

/// Analyze and edit bibliography search expressions, with offsets in UTF-16 code units.
#[module]
pub struct Query;

#[module]
impl Query {
    /// Query field descriptors, matching modes and bibliography type selectors.
    pub fn catalog() -> SearchCatalogDto {
        svc::search_catalog().into()
    }

    /// Parse the complete expression and report diagnostics in the original text.
    pub async fn analyze(text: String) -> Result<QueryAnalysisDto, String> {
        blocking(move || svc::analyze_query(text).try_into())
            .await
            .map_err(|e| e.to_string())?
    }

    /// Construct a canonical expression for one typed condition or selected tag.
    pub async fn condition(input: QueryConditionDto) -> Result<QueryAnalysisDto, String> {
        blocking(move || svc::build_condition(input.into()).try_into())
            .await
            .map_err(|e| e.to_string())?
    }

    /// Apply a structural edit to a node while preserving bound tag identities.
    pub async fn edit(input: QueryEditDto) -> Result<QueryAnalysisDto, String> {
        blocking(move || svc::edit_query(input.try_into()?).try_into())
            .await
            .map_err(|e| e.to_string())?
    }

    /// Replace the half-open UTF-16 range, retaining tag identities outside that range.
    pub async fn edit_text(input: QueryTextEditDto) -> Result<QueryAnalysisDto, String> {
        blocking(move || edit_text(input))
            .await
            .map_err(|e| e.to_string())?
    }

    /// Render a canonical projection and recompute its UTF-16 token spans.
    pub async fn render(doc: QueryDocumentDto) -> Result<QueryAnalysisDto, String> {
        blocking(move || svc::render_query(doc.try_into()?).try_into())
            .await
            .map_err(|e| e.to_string())?
    }

    /// Resolve current names and explicit merges for previously bound tag identities.
    pub async fn refresh(doc: QueryDocumentDto) -> Result<QueryAnalysisDto, String> {
        blocking(move || {
            svc::refresh_query(doc.try_into()?)
                .map_err(|e| e.to_string())?
                .try_into()
        })
        .await
        .map_err(|e| e.to_string())?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_and_astral_boundaries_round_trip_and_reject_split_characters() {
        let text = "甲🦀e\u{301}";
        let offsets = Offsets::new(text);
        for (byte, units) in [(0, 0), (3, 1), (7, 3), (8, 4), (10, 5)] {
            assert_eq!(offsets.byte(units), Ok(byte));
            assert_eq!(offsets.units(byte), Ok(units));
        }
        assert!(offsets.byte(2).is_err());
        assert!(offsets.byte(6).is_err());
        assert!(offsets.units(4).is_err());
    }

    #[test]
    fn bridge_replaces_exact_utf16_selection_after_non_ascii_prefix() {
        let text = "甲🦀 tag:文献";
        let analysis = QueryAnalysisDto::try_from(svc::analyze_query(text.into())).unwrap();
        let mut doc = analysis.document.unwrap();
        let tag = doc
            .nodes
            .iter_mut()
            .find(|n| n.text_value == "文献")
            .unwrap();
        assert_eq!((tag.start, tag.end), (8, 10));
        tag.tag_id = Some("stable-id".into());
        let edit = QueryTextEditDto {
            text: text.into(),
            document: doc,
            start: 0,
            end: 3,
            replacement: "新".into(),
            full_replace: false,
        };
        let result = edit_text(edit).unwrap();
        assert_eq!(result.text, "新 tag:文献");
        assert_eq!(
            result.selection,
            Some(QuerySelectionDto { start: 1, end: 1 })
        );
        let doc = result.document.unwrap();
        let tag = doc.nodes.iter().find(|n| n.tag_id.is_some()).unwrap();
        assert_eq!((tag.start, tag.end), (6, 8));
    }

    #[test]
    fn incomplete_edit_preserves_valid_token_spans_and_corrects_the_touched_range() {
        let text = "tag:数学 title:🦀";
        let mut doc = QueryAnalysisDto::try_from(svc::analyze_query(text.into()))
            .unwrap()
            .document
            .unwrap();
        let tag = doc
            .nodes
            .iter_mut()
            .find(|n| n.text_value == "数学")
            .unwrap();
        tag.tag_id = Some("stable-tag".into());
        let result = edit_text(QueryTextEditDto {
            document: doc,
            text: text.into(),
            start: 13,
            end: 15,
            replacement: "\"".into(),
            full_replace: false,
        })
        .unwrap();
        let doc = result.document.unwrap();
        assert_eq!(doc.text, "tag:数学 title:\"");
        let tag = doc.nodes.iter().find(|n| n.tag_id.is_some()).unwrap();
        assert_eq!((tag.start, tag.end), (4, 6));
        let old_leaf = doc.nodes.iter().find(|n| n.text_value == "🦀").unwrap();
        assert_eq!((old_leaf.start, old_leaf.end), (13, 14));
        assert!(svc::QueryDocumentRecord::try_from(doc).is_ok());
    }

    #[test]
    fn analysis_keeps_decimal_revision_beyond_js_integer_precision() {
        let context = SearchContextDto::from(svc::SearchContext {
            path: "library.bib".into(),
            file_version: "immutable".into(),
            tag_revision: Some(u64::MAX),
            query_version: u64::MAX - 1,
        });
        assert_eq!(
            context.tag_revision.as_deref(),
            Some("18446744073709551615")
        );
        let restored: svc::SearchContext = context.try_into().unwrap();
        assert_eq!(restored.query_version, u64::MAX - 1);
    }
}
