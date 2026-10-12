use crate::records::ReferenceRecord;
pub use bibcitex_service::{
    AdvancedSearchRequest, AdvancedSearchResponse, BatchTagMutation, BatchTagSnapshot,
    PendingAssociation, QueryAnalysis, QueryCondition, QueryDiagnostic, QueryDocumentRecord,
    QueryEdit, QueryEditAction, QueryInputKind, QueryNodeKind, QueryNodeRecord, QueryStatus,
    QueryTextEdit, SearchCatalog, SearchContext, SearchEvidence, SearchFieldDescriptor,
    SearchMatchRange, SearchMode, SearchSourceRecord, SearchTypeDescriptor, TagAction, TagMutation,
    TagRecord, TagSnapshot, TagTarget,
};
// Platform-neutral advanced search and metadata contracts.
/// Selects ordinary compatibility search or the shared advanced query language.
#[uniffi::remote(Enum)]
pub enum SearchMode {
    /// Preserve the existing whitespace-separated ordinary search contract.
    Ordinary,
    /// Enable the complete typed advanced-search grammar.
    Advanced,
}
/// Selects raw name-based text or a typed tree retaining stable tag identities.
#[uniffi::remote(Enum)]
pub enum QueryInputKind {
    /// Parse raw text; tag references resolve by current name.
    Text,
    /// Execute the supplied validated tree, preserving selected tag IDs.
    Bound,
}
/// Query state independent of storage errors; only Valid permits actionable results.
#[uniffi::remote(Enum)]
pub enum QueryStatus {
    /// Complete query eligible for execution.
    Valid,
    /// Only a required suffix, operand, or closing delimiter is missing.
    Incomplete,
    /// A definite lexical, semantic, structural, or budget error prevents execution.
    Invalid,
    /// Expected metadata or result snapshot has changed; refresh before execution.
    StaleQuery,
}
/// Node operation in a flat, ordered, single-parent query tree.
#[uniffi::remote(Enum)]
pub enum QueryNodeKind {
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
/// One flat query node. Child IDs index the containing nodes array; spans are UTF-8 bytes.
#[uniffi::remote(Record)]
pub struct QueryNodeRecord {
    /// Operation represented by this node.
    pub kind: QueryNodeKind,
    /// Decoded literal value, or canonical field name for a Scope node.
    pub text_value: String,
    /// Stable selected tag ID; None denotes a name-based condition.
    pub tag_id: Option<String>,
    /// Ordered child indices into the containing node arena.
    pub children: Vec<u32>,
    /// Inclusive UTF-8 byte offset in the original query.
    pub start: u32,
    /// Exclusive UTF-8 byte offset in the original query.
    pub end: u32,
}
/// Ordered query arena. An absent root represents the empty query; unreachable and shared nodes are rejected.
#[uniffi::remote(Record)]
pub struct QueryDocumentRecord {
    /// Flat node arena, validated for cycles, reachability, single parents, and resource budgets.
    pub nodes: Vec<QueryNodeRecord>,
    /// Root node index; None denotes an empty query.
    pub root: Option<u32>,
}
/// Language-neutral diagnostic code with a half-open original UTF-8 byte range.
#[uniffi::remote(Record)]
pub struct QueryDiagnostic {
    /// Stable error or warning key for localization.
    pub code: String,
    /// Inclusive UTF-8 byte offset in the original query.
    pub start: u32,
    /// Exclusive UTF-8 byte offset in the original query.
    pub end: u32,
    /// True for non-blocking information such as an unknown tag name.
    pub warning: bool,
}
/// Strict analysis or edit result. Invalid and incomplete trees cannot be executed.
#[uniffi::remote(Record)]
pub struct QueryAnalysis {
    /// Strict query validity or stale-snapshot state.
    pub status: QueryStatus,
    /// Bounded structured diagnostics; warnings do not prevent execution.
    pub diagnostics: Vec<QueryDiagnostic>,
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: Option<QueryDocumentRecord>,
    /// Original query text or canonical projection produced by an editor operation.
    pub text: String,
    /// Metadata revision used by this result; absent when metadata is not required.
    pub tag_revision: Option<u64>,
}
/// Explicit-mode request. Bound tag IDs require an expected metadata revision and a matching text projection.
#[uniffi::remote(Record)]
pub struct AdvancedSearchRequest {
    /// Bibliography path identifying the current source.
    pub path: String,
    /// Explicit ordinary or advanced execution mode.
    pub mode: SearchMode,
    /// Whether execution uses raw text or the supplied bound tree.
    pub input_kind: QueryInputKind,
    /// Original query text or canonical projection produced by an editor operation.
    pub text: String,
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: Option<QueryDocumentRecord>,
    /// Required revision for ById leaves; a mismatch returns StaleQuery without rows.
    pub expected_tag_revision: Option<u64>,
    /// Default scope for implicit terms: all, title, author, journal, or year.
    pub default_field: String,
    /// Independent legacy type filter, intersected with the entire expression.
    pub type_filter: String,
    /// Caller generation echoed to reject out-of-order responses.
    pub query_version: u64,
}
/// Immutable result identity. Revalidate before copy, paste, or attachment actions.
#[uniffi::remote(Record)]
pub struct SearchContext {
    /// Bibliography path identifying the current source.
    pub path: String,
    /// Content digest and filesystem incarnation of the exact bibliography snapshot.
    pub file_version: String,
    /// Metadata revision used by this result; absent when metadata is not required.
    pub tag_revision: Option<u64>,
    /// Caller generation echoed to reject out-of-order responses.
    pub query_version: u64,
}
/// Positive match evidence for one source record, excluding negated conditions.
#[uniffi::remote(Record)]
pub struct SearchEvidence {
    /// Original UTF-8 ranges within individual field entries; never normalized offsets.
    pub ranges: Vec<SearchMatchRange>,
    /// Persistent record UUID shared with the returned reference.
    pub reference_id: String,
    /// Canonical fields supporting the successful positive branch.
    pub fields: Vec<String>,
    /// Whether this row belongs to the spelling-supplement batch.
    pub fuzzy: bool,
    /// Positive query terms that required spelling supplementation, in contribution order.
    pub fuzzy_terms: Vec<String>,
}
/// Search results and diagnostics from one bibliography and metadata snapshot.
#[uniffi::remote(Record)]
pub struct AdvancedSearchResponse {
    /// Query status, source spans, and the authoritative typed representation.
    pub analysis: QueryAnalysis,
    /// Matching records in deterministic relevance order; empty on non-valid states.
    pub references: Vec<ReferenceRecord>,
    /// Positive matching information aligned by reference identity.
    pub evidence: Vec<SearchEvidence>,
    /// Action-validation context, absent when results are not actionable.
    pub context: Option<SearchContext>,
}
/// Canonical field and type values shared by all native editors and candidate lists.
#[uniffi::remote(Record)]
pub struct SearchCatalog {
    /// Typed field metadata, including shared localization and matching capabilities.
    pub field_descriptors: Vec<SearchFieldDescriptor>,
    /// Typed bibliography values and their shared localization keys.
    pub type_descriptors: Vec<SearchTypeDescriptor>,
    /// Canonical fields supporting the successful positive branch.
    pub fields: Vec<String>,
    /// Canonical lower-case bibliography type operands.
    pub types: Vec<String>,
}
/// Stable tag identity and user-facing display name. Renames preserve the ID.
#[uniffi::remote(Record)]
pub struct TagRecord {
    /// Shared sRGB color in #RRGGBB format.
    pub color: String,
    /// Immutable tag or source identifier.
    pub id: String,
    /// User-facing tag display name.
    pub name: String,
}
/// An absent bibliography record whose tag associations await explicit reconnection.
#[uniffi::remote(Record)]
pub struct PendingAssociation {
    /// Stable ID of the absent record to reconnect explicitly.
    pub record_id: String,
    /// Stable identity of the original bibliography source.
    pub source_id: String,
    /// Bibliography path identifying the current source.
    pub path: String,
    /// Original case-sensitive cite key used for association lookup.
    pub cite_key: String,
    /// Tag catalog or associated stable tag IDs, according to the enclosing record.
    pub tags: Vec<String>,
}
/// Consistent catalog, assignments, sources, and pending records at one metadata revision.
#[uniffi::remote(Record)]
pub struct TagSnapshot {
    /// Selected stable bibliography identity, empty for the global catalog.
    pub library_uid: String,
    /// Selected stable record identity, empty for catalog-only requests.
    pub record_uid: String,
    /// Monotonically increasing metadata revision for optimistic concurrency.
    pub revision: u64,
    /// Tag catalog or associated stable tag IDs, according to the enclosing record.
    pub tags: Vec<TagRecord>,
    /// Tags assigned to the requested current record.
    pub assigned_ids: Vec<String>,
    /// Absent records with retained associations that need explicit reconnection.
    pub pending: Vec<PendingAssociation>,
    /// Registered metadata sources available for relocation.
    pub sources: Vec<SearchSourceRecord>,
}
/// Metadata mutation intent. All mutations require optimistic revision validation.
#[uniffi::remote(Enum)]
pub enum TagAction {
    /// Update a tag color without changing identity.
    Recolor,
    /// Create or reuse a normalized same-name tag.
    Create,
    /// Rename an existing tag without changing its identity.
    Rename,
    /// Delete a tag and its assignments.
    Delete,
    /// Merge assignments into an explicitly selected destination tag.
    Merge,
    /// Add a tag to the exact current record idempotently.
    Assign,
    /// Remove a tag assignment idempotently.
    Unassign,
    /// Move pending associations to an explicitly selected current record.
    Relink,
    /// Relocate an explicitly selected stable source to a new file path.
    Relocate,
}
/// Revision-checked metadata command. Relocate uses a source ID; Relink uses a pending record ID.
#[uniffi::remote(Record)]
pub struct TagMutation {
    /// Stable bibliography identity for record associations.
    pub library_uid: String,
    /// Stable record identity for record associations.
    pub record_uid: String,
    /// Optional color when creating or recoloring a tag.
    pub color: Option<String>,
    /// Requested semantic edit or metadata operation.
    pub action: TagAction,
    /// Immutable tag or source identifier.
    pub id: String,
    /// User-facing tag display name.
    pub name: String,
    /// Destination tag ID for an explicit merge.
    pub target_id: String,
    /// Destination file path for explicit relocation only; associations use UUIDs.
    pub path: String,
    /// Legacy display context; associations use record_uid.
    pub cite_key: String,
    /// Revision observed by the editor; stale mutations fail without writing.
    pub expected_revision: u64,
}
/// Structural edit applied to an explicit target node or the document root.
#[uniffi::remote(Enum)]
pub enum QueryEditAction {
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
/// Typed edit transaction. Append and Replace require an operand; Remove also prunes empty groups.
#[uniffi::remote(Record)]
pub struct QueryEdit {
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: QueryDocumentRecord,
    /// Requested semantic edit or metadata operation.
    pub action: QueryEditAction,
    /// Node index to edit; None selects the root.
    pub target: Option<u32>,
    /// Validated subtree supplied for append or replacement.
    pub operand: Option<QueryDocumentRecord>,
}
/// Validated leaf or same-field term group constructed by the shared service.
#[uniffi::remote(Record)]
pub struct QueryCondition {
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
/// Actual native input replacement using UTF-8 offsets; untouched tag bindings retain their IDs.
#[uniffi::remote(Record)]
pub struct QueryTextEdit {
    /// Typed query tree. Non-valid text-edit results may carry non-executable binding anchors for the next edit.
    pub document: QueryDocumentRecord,
    /// Original query text or canonical projection produced by an editor operation.
    pub text: String,
    /// Inclusive UTF-8 byte offset in the original query.
    pub start: u32,
    /// Exclusive UTF-8 byte offset in the original query.
    pub end: u32,
    /// Native input inserted into the supplied byte range.
    pub replacement: String,
    /// True for whole-query replacement; all selected tags become name-based input.
    pub full_replace: bool,
}

/// Stable bibliography source available for explicit relocation without guessing identity.
#[uniffi::remote(Record)]
pub struct SearchSourceRecord {
    /// Immutable tag or source identifier.
    pub id: String,
    /// Bibliography path identifying the current source.
    pub path: String,
    /// Whether the source path exists when the catalog is read.
    pub available: bool,
}

/// Original field location for a positive matching condition, before Unicode normalization.
#[uniffi::remote(Record)]
pub struct SearchMatchRange {
    /// Canonical field name, such as title or author.
    pub field_name: String,
    /// Zero-based entry for repeated fields; zero for a single chunked field.
    pub entry_index: u32,
    /// Inclusive original UTF-8 byte offset across that entry's chunks.
    pub start: u32,
    /// Exclusive original UTF-8 byte offset across that entry's chunks.
    pub end: u32,
}

/// Shared presentation and matching contract for one advanced field.
#[uniffi::remote(Record)]
pub struct SearchFieldDescriptor {
    /// Canonical syntax name accepted by condition builders.
    pub name: String,
    /// Localization key owned by the shared catalog.
    pub label_key: String,
    /// Matching family: text, identifier, year, type, or tag.
    pub match_mode: String,
    /// Whether strict phrases are a distinct matching mode for this field.
    pub supports_phrase: bool,
}
/// Canonical type operand and shared presentation label.
#[uniffi::remote(Record)]
pub struct SearchTypeDescriptor {
    /// Lower-case syntax value; thesis includes all supported degree variants.
    pub name: String,
    /// Localization key owned by the shared catalog.
    pub label_key: String,
}

/// A record selected by its stable library and record UUIDs.
#[uniffi::remote(Record)]
pub struct TagTarget {
    /// Stable bibliography UUID.
    pub library_uid: String,
    /// Stable reference UUID within that bibliography.
    pub record_uid: String,
}
/// Consistent tag state across a deduplicated selection.
#[uniffi::remote(Record)]
pub struct BatchTagSnapshot {
    /// Metadata revision required by the next mutation.
    pub revision: u64,
    /// Global tag catalog.
    pub tags: Vec<TagRecord>,
    /// Tags present on every selected reference.
    pub assigned_ids: Vec<String>,
    /// Tags present on some, but not all, selected references.
    pub partial_ids: Vec<String>,
    /// Number of distinct selected references.
    pub count: u64,
}
/// One atomic assignment operation across all selected records.
#[uniffi::remote(Record)]
pub struct BatchTagMutation {
    /// Exact selected identities; duplicate pairs are ignored.
    pub targets: Vec<TagTarget>,
    /// Only Create, Assign and Unassign are accepted.
    pub action: TagAction,
    /// Existing tag UUID for Assign or Unassign.
    pub id: String,
    /// Display name for Create (creates and assigns in one transaction).
    pub name: String,
    /// Revision observed by the batch editor.
    pub expected_revision: u64,
}
