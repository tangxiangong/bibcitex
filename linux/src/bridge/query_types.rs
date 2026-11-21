use bibcitex_service as svc;
use nsh_core::prelude::*;

/// Selects ordinary compatibility search or the shared advanced query language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum SearchModeDto {
    /// Preserve the existing whitespace-separated ordinary search contract.
    Ordinary,
    /// Enable the complete typed advanced-search grammar.
    Advanced,
}
impl From<SearchModeDto> for svc::SearchMode {
    fn from(value: SearchModeDto) -> Self {
        match value {
            SearchModeDto::Ordinary => Self::Ordinary,
            SearchModeDto::Advanced => Self::Advanced,
        }
    }
}
/// Selects raw name-based text or a typed tree retaining stable tag identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum QueryInputKindDto {
    /// Parse raw text; tag references resolve by current name.
    Text,
    /// Execute the supplied validated tree, preserving selected tag IDs.
    Bound,
}
impl From<QueryInputKindDto> for svc::QueryInputKind {
    fn from(value: QueryInputKindDto) -> Self {
        match value {
            QueryInputKindDto::Text => Self::Text,
            QueryInputKindDto::Bound => Self::Bound,
        }
    }
}
/// Query state independent of storage errors; only Valid permits actionable results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum QueryStatusDto {
    /// Complete query eligible for execution.
    Valid,
    /// Only a required suffix, operand, or closing delimiter is missing.
    Incomplete,
    /// A definite lexical, semantic, structural, or budget error prevents execution.
    Invalid,
    /// Expected metadata or result snapshot has changed; refresh before execution.
    StaleQuery,
}
impl From<svc::QueryStatus> for QueryStatusDto {
    fn from(value: svc::QueryStatus) -> Self {
        match value {
            svc::QueryStatus::Valid => Self::Valid,
            svc::QueryStatus::Incomplete => Self::Incomplete,
            svc::QueryStatus::Invalid => Self::Invalid,
            svc::QueryStatus::StaleQuery => Self::StaleQuery,
        }
    }
}
/// Node operation in a flat, ordered, single-parent query tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum QueryNodeKindDto {
    /// Ordinary normalized text value.
    Word,
    /// Strict contiguous phrase value.
    Phrase,
    /// All ordered children must match.
    And,
    /// At least one ordered child must match.
    Or,
    /// Logical complement of the single child within external candidates.
    Not,
    /// Apply the named field to implicit descendants.
    Scope,
}
impl From<QueryNodeKindDto> for svc::QueryNodeKind {
    fn from(value: QueryNodeKindDto) -> Self {
        match value {
            QueryNodeKindDto::Word => Self::Word,
            QueryNodeKindDto::Phrase => Self::Phrase,
            QueryNodeKindDto::And => Self::And,
            QueryNodeKindDto::Or => Self::Or,
            QueryNodeKindDto::Not => Self::Not,
            QueryNodeKindDto::Scope => Self::Scope,
        }
    }
}
impl From<svc::QueryNodeKind> for QueryNodeKindDto {
    fn from(value: svc::QueryNodeKind) -> Self {
        match value {
            svc::QueryNodeKind::Word => Self::Word,
            svc::QueryNodeKind::Phrase => Self::Phrase,
            svc::QueryNodeKind::And => Self::And,
            svc::QueryNodeKind::Or => Self::Or,
            svc::QueryNodeKind::Not => Self::Not,
            svc::QueryNodeKind::Scope => Self::Scope,
        }
    }
}
/// Structural edit applied to an explicit target node or the document root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum QueryEditActionDto {
    /// Combine the target and operand with AND.
    AppendAnd,
    /// Combine the target and operand with OR.
    AppendOr,
    /// Negate the target subtree.
    Negate,
    /// Remove the target and prune empty ancestors.
    Remove,
    /// Replace the target with the supplied operand.
    Replace,
}
impl From<QueryEditActionDto> for svc::QueryEditAction {
    fn from(value: QueryEditActionDto) -> Self {
        match value {
            QueryEditActionDto::AppendAnd => Self::AppendAnd,
            QueryEditActionDto::AppendOr => Self::AppendOr,
            QueryEditActionDto::Negate => Self::Negate,
            QueryEditActionDto::Remove => Self::Remove,
            QueryEditActionDto::Replace => Self::Replace,
        }
    }
}
/// One flat query node. Child IDs index the containing nodes array; spans are UTF-16 code units.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QueryNodeDto {
    /// Operation represented by this node.
    pub kind: QueryNodeKindDto,
    /// Decoded literal value, or canonical field name for a Scope node.
    pub text_value: String,
    /// Stable selected tag ID; None denotes a name-based condition.
    pub tag_id: Option<String>,
    /// Ordered child indices into the containing node arena.
    pub children: Vec<u32>,
    /// Inclusive UTF-16 code-unit offset in the original query.
    pub start: u32,
    /// Exclusive UTF-16 code-unit offset in the original query.
    pub end: u32,
}
/// Ordered query arena. An absent root represents the empty query; unreachable and shared nodes are rejected.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QueryDocumentDto {
    /// Exact input whose UTF-16 indices the nodes reference.
    pub text: String,
    /// Flat node arena, validated for cycles, reachability, single parents, and resource budgets.
    pub nodes: Vec<QueryNodeDto>,
    /// Root node index; None denotes an empty query.
    pub root: Option<u32>,
}
/// Language-neutral diagnostic code with a half-open original UTF-16 code-unit range.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QueryDiagnosticDto {
    /// Stable error or warning key for localization.
    pub code: String,
    /// Inclusive UTF-16 code-unit offset in the original query.
    pub start: u32,
    /// Exclusive UTF-16 code-unit offset in the original query.
    pub end: u32,
    /// True for non-blocking information such as an unknown tag name.
    pub warning: bool,
}
/// Selection in the exact returned query string, measured in UTF-16 code units.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QuerySelectionDto {
    /// Inclusive selection start, or caret position for an empty selection.
    pub start: u32,
    /// Exclusive selection end.
    pub end: u32,
}

/// Strict analysis or edit result. Invalid and incomplete trees cannot be executed.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QueryAnalysisDto {
    /// Strict query validity or stale-snapshot state.
    pub status: QueryStatusDto,
    /// Bounded structured diagnostics; warnings do not prevent execution.
    pub diagnostics: Vec<QueryDiagnosticDto>,
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: Option<QueryDocumentDto>,
    /// Original query text or canonical projection produced by an editor operation.
    pub text: String,
    /// Collapsed selection following a text replacement; absent for non-editor operations.
    pub selection: Option<QuerySelectionDto>,
    /// Metadata revision used by this result; absent when metadata is not required.
    pub tag_revision: Option<String>,
}
/// Validated leaf or same-field term group constructed by the shared service.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QueryConditionDto {
    /// Canonical field; empty selects the default search scope.
    pub field_name: String,
    /// Decoded literal value, or canonical field name for a Scope node.
    pub text_value: String,
    /// Preserve strict contiguous phrase semantics rather than ordinary word matching.
    pub phrase: bool,
    /// Wrap the constructed condition in logical negation.
    pub exclude: bool,
    /// Stable selected tag ID; None denotes a name-based condition.
    pub tag_id: Option<String>,
}
impl From<QueryConditionDto> for svc::QueryCondition {
    fn from(value: QueryConditionDto) -> Self {
        Self {
            field_name: value.field_name,
            text_value: value.text_value,
            phrase: value.phrase,
            exclude: value.exclude,
            tag_id: value.tag_id,
        }
    }
}
/// Actual native input replacement using UTF-16 offsets; untouched tag bindings retain their IDs.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QueryTextEditDto {
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: QueryDocumentDto,
    /// Original query text or canonical projection produced by an editor operation.
    pub text: String,
    /// Inclusive UTF-16 code-unit offset in the original query.
    pub start: u32,
    /// Exclusive UTF-16 code-unit offset in the original query.
    pub end: u32,
    /// Native input inserted into the supplied UTF-16 range.
    pub replacement: String,
    /// True for whole-query replacement; all selected tags become name-based input.
    pub full_replace: bool,
}
/// Typed edit transaction. Append and Replace require an operand; Remove also prunes empty groups.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct QueryEditDto {
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: QueryDocumentDto,
    /// Requested semantic edit or metadata operation.
    pub action: QueryEditActionDto,
    /// Node index to edit; None selects the root.
    pub target: Option<u32>,
    /// Validated subtree supplied for append or replacement.
    pub operand: Option<QueryDocumentDto>,
}
impl TryFrom<QueryEditDto> for svc::QueryEdit {
    type Error = String;
    fn try_from(value: QueryEditDto) -> Result<Self, String> {
        Ok(Self {
            document: value.document.try_into()?,
            action: value.action.into(),
            target: value.target,
            operand: value.operand.map(TryInto::try_into).transpose()?,
        })
    }
}
/// Shared presentation and matching contract for one advanced field.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct SearchFieldDescriptorDto {
    /// Canonical syntax name accepted by condition builders.
    pub name: String,
    /// Localization key owned by the shared catalog.
    pub label_key: String,
    /// Matching family: text, identifier, year, type, or tag.
    pub match_mode: String,
    /// Whether strict phrases are a distinct matching mode for this field.
    pub supports_phrase: bool,
}
impl From<svc::SearchFieldDescriptor> for SearchFieldDescriptorDto {
    fn from(value: svc::SearchFieldDescriptor) -> Self {
        Self {
            name: value.name,
            label_key: value.label_key,
            match_mode: value.match_mode,
            supports_phrase: value.supports_phrase,
        }
    }
}
/// Canonical type operand and shared presentation label.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct SearchTypeDescriptorDto {
    /// Lower-case syntax value; thesis includes all supported degree variants.
    pub name: String,
    /// Localization key owned by the shared catalog.
    pub label_key: String,
}
impl From<svc::SearchTypeDescriptor> for SearchTypeDescriptorDto {
    fn from(value: svc::SearchTypeDescriptor) -> Self {
        Self {
            name: value.name,
            label_key: value.label_key,
        }
    }
}
/// Canonical field and type values shared by all native editors and candidate lists.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct SearchCatalogDto {
    /// Typed field metadata, including shared localization and matching capabilities.
    pub field_descriptors: Vec<SearchFieldDescriptorDto>,
    /// Typed bibliography values and their shared localization keys.
    pub type_descriptors: Vec<SearchTypeDescriptorDto>,
    /// Canonical fields supporting the successful positive branch.
    pub fields: Vec<String>,
    /// Canonical lower-case bibliography type operands.
    pub types: Vec<String>,
}
impl From<svc::SearchCatalog> for SearchCatalogDto {
    fn from(value: svc::SearchCatalog) -> Self {
        Self {
            field_descriptors: value
                .field_descriptors
                .into_iter()
                .map(Into::into)
                .collect(),
            type_descriptors: value.type_descriptors.into_iter().map(Into::into).collect(),
            fields: value.fields,
            types: value.types,
        }
    }
}
/// Explicit-mode request. Bound tag IDs require an expected metadata revision and a matching text projection.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct AdvancedSearchRequestDto {
    /// Bibliography path identifying the current source.
    pub path: String,
    /// Explicit ordinary or advanced execution mode.
    pub mode: SearchModeDto,
    /// Whether execution uses raw text or the supplied bound tree.
    pub input_kind: QueryInputKindDto,
    /// Original query text or canonical projection produced by an editor operation.
    pub text: String,
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: Option<QueryDocumentDto>,
    /// Required revision for ById leaves; a mismatch returns StaleQuery without rows.
    pub expected_tag_revision: Option<String>,
    /// Default scope for implicit terms: all, title, author, journal, or year.
    pub default_field: String,
    /// Independent legacy type filter, intersected with the entire expression.
    pub type_filter: String,
    /// Caller generation echoed to reject out-of-order responses.
    pub query_version: String,
}
impl TryFrom<AdvancedSearchRequestDto> for svc::AdvancedSearchRequest {
    type Error = String;
    fn try_from(value: AdvancedSearchRequestDto) -> Result<Self, String> {
        Ok(Self {
            path: value.path,
            mode: value.mode.into(),
            input_kind: value.input_kind.into(),
            text: value.text,
            document: value.document.map(TryInto::try_into).transpose()?,
            expected_tag_revision: value
                .expected_tag_revision
                .map(|s| s.parse::<u64>())
                .transpose()
                .map_err(|_| "InvalidRevision".to_owned())?,
            default_field: value.default_field,
            type_filter: value.type_filter,
            query_version: value
                .query_version
                .parse::<u64>()
                .map_err(|_| "InvalidRevision".to_owned())?,
        })
    }
}
/// Immutable result identity. Revalidate before copy, paste, or attachment actions.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct SearchContextDto {
    /// Bibliography path identifying the current source.
    pub path: String,
    /// Content digest and filesystem incarnation of the exact bibliography snapshot.
    pub file_version: String,
    /// Metadata revision used by this result; absent when metadata is not required.
    pub tag_revision: Option<String>,
    /// Caller generation echoed to reject out-of-order responses.
    pub query_version: String,
}
impl TryFrom<SearchContextDto> for svc::SearchContext {
    type Error = String;
    fn try_from(value: SearchContextDto) -> Result<Self, String> {
        Ok(Self {
            path: value.path,
            file_version: value.file_version,
            tag_revision: value
                .tag_revision
                .map(|s| s.parse::<u64>())
                .transpose()
                .map_err(|_| "InvalidRevision".to_owned())?,
            query_version: value
                .query_version
                .parse::<u64>()
                .map_err(|_| "InvalidRevision".to_owned())?,
        })
    }
}
impl From<svc::SearchContext> for SearchContextDto {
    fn from(value: svc::SearchContext) -> Self {
        Self {
            path: value.path,
            file_version: value.file_version,
            tag_revision: value.tag_revision.map(|n| n.to_string()),
            query_version: value.query_version.to_string(),
        }
    }
}
