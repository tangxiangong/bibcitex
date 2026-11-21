use bibcitex_service as svc;
use nsh_core::prelude::*;

/// Metadata mutation intent. All mutations require optimistic revision validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum TagActionDto {
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
impl From<TagActionDto> for svc::TagAction {
    fn from(value: TagActionDto) -> Self {
        match value {
            TagActionDto::Recolor => Self::Recolor,
            TagActionDto::Create => Self::Create,
            TagActionDto::Rename => Self::Rename,
            TagActionDto::Delete => Self::Delete,
            TagActionDto::Merge => Self::Merge,
            TagActionDto::Assign => Self::Assign,
            TagActionDto::Unassign => Self::Unassign,
            TagActionDto::Relink => Self::Relink,
            TagActionDto::Relocate => Self::Relocate,
        }
    }
}
/// Stable tag identity and user-facing display name. Renames preserve the ID.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct TagDto {
    /// Shared sRGB color in #RRGGBB format.
    pub color: String,
    /// Immutable tag or source identifier.
    pub id: String,
    /// User-facing tag display name.
    pub name: String,
}
impl From<svc::TagRecord> for TagDto {
    fn from(value: svc::TagRecord) -> Self {
        Self {
            color: value.color,
            id: value.id,
            name: value.name,
        }
    }
}
/// A record selected by its stable library and record UUIDs.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct TagTargetDto {
    /// Stable bibliography UUID.
    pub library_uid: String,
    /// Stable reference UUID within that bibliography.
    pub record_uid: String,
}
impl From<TagTargetDto> for svc::TagTarget {
    fn from(value: TagTargetDto) -> Self {
        Self {
            library_uid: value.library_uid,
            record_uid: value.record_uid,
        }
    }
}
/// Revision-checked metadata command. Relocate uses a source ID; Relink uses a pending record ID.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct TagMutationDto {
    /// Stable bibliography identity for record associations.
    pub library_uid: String,
    /// Stable record identity for record associations.
    pub record_uid: String,
    /// Optional color when creating or recoloring a tag.
    pub color: Option<String>,
    /// Requested semantic edit or metadata operation.
    pub action: TagActionDto,
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
    pub expected_revision: String,
}
impl TryFrom<TagMutationDto> for svc::TagMutation {
    type Error = String;
    fn try_from(value: TagMutationDto) -> Result<Self, String> {
        Ok(Self {
            library_uid: value.library_uid,
            record_uid: value.record_uid,
            color: value.color,
            action: value.action.into(),
            id: value.id,
            name: value.name,
            target_id: value.target_id,
            path: value.path,
            cite_key: value.cite_key,
            expected_revision: value
                .expected_revision
                .parse::<u64>()
                .map_err(|_| "InvalidRevision".to_owned())?,
        })
    }
}
/// One atomic assignment operation across all selected records.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct BatchTagMutationDto {
    /// Exact selected identities; duplicate pairs are ignored.
    pub targets: Vec<TagTargetDto>,
    /// Only Create, Assign and Unassign are accepted.
    pub action: TagActionDto,
    /// Existing tag UUID for Assign or Unassign.
    pub id: String,
    /// Display name for Create (creates and assigns in one transaction).
    pub name: String,
    /// Revision observed by the batch editor.
    pub expected_revision: String,
}
impl TryFrom<BatchTagMutationDto> for svc::BatchTagMutation {
    type Error = String;
    fn try_from(value: BatchTagMutationDto) -> Result<Self, String> {
        Ok(Self {
            targets: value.targets.into_iter().map(Into::into).collect(),
            action: value.action.into(),
            id: value.id,
            name: value.name,
            expected_revision: value
                .expected_revision
                .parse::<u64>()
                .map_err(|_| "InvalidRevision".to_owned())?,
        })
    }
}
/// An absent bibliography record whose tag associations await explicit reconnection.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct PendingAssociationDto {
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
impl From<svc::PendingAssociation> for PendingAssociationDto {
    fn from(value: svc::PendingAssociation) -> Self {
        Self {
            record_id: value.record_id,
            source_id: value.source_id,
            path: value.path,
            cite_key: value.cite_key,
            tags: value.tags,
        }
    }
}
/// Stable bibliography source available for explicit relocation without guessing identity.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct SearchSourceDto {
    /// Immutable tag or source identifier.
    pub id: String,
    /// Bibliography path identifying the current source.
    pub path: String,
    /// Whether the source path exists when the catalog is read.
    pub available: bool,
}
impl From<svc::SearchSourceRecord> for SearchSourceDto {
    fn from(value: svc::SearchSourceRecord) -> Self {
        Self {
            id: value.id,
            path: value.path,
            available: value.available,
        }
    }
}
/// Consistent catalog, assignments, sources, and pending records at one metadata revision.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct TagSnapshotDto {
    /// Selected stable bibliography identity, empty for the global catalog.
    pub library_uid: String,
    /// Selected stable record identity, empty for catalog-only requests.
    pub record_uid: String,
    /// Monotonically increasing metadata revision for optimistic concurrency.
    pub revision: String,
    /// Tag catalog or associated stable tag IDs, according to the enclosing record.
    pub tags: Vec<TagDto>,
    /// Tags assigned to the requested current record.
    pub assigned_ids: Vec<String>,
    /// Absent records with retained associations that need explicit reconnection.
    pub pending: Vec<PendingAssociationDto>,
    /// Registered metadata sources available for relocation.
    pub sources: Vec<SearchSourceDto>,
}
impl From<svc::TagSnapshot> for TagSnapshotDto {
    fn from(value: svc::TagSnapshot) -> Self {
        Self {
            library_uid: value.library_uid,
            record_uid: value.record_uid,
            revision: value.revision.to_string(),
            tags: value.tags.into_iter().map(Into::into).collect(),
            assigned_ids: value.assigned_ids,
            pending: value.pending.into_iter().map(Into::into).collect(),
            sources: value.sources.into_iter().map(Into::into).collect(),
        }
    }
}
/// Consistent tag state across a deduplicated selection.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct BatchTagSnapshotDto {
    /// Metadata revision required by the next mutation.
    pub revision: String,
    /// Global tag catalog.
    pub tags: Vec<TagDto>,
    /// Tags present on every selected reference.
    pub assigned_ids: Vec<String>,
    /// Tags present on some, but not all, selected references.
    pub partial_ids: Vec<String>,
    /// Number of distinct selected references.
    pub count: u64,
}
impl From<svc::BatchTagSnapshot> for BatchTagSnapshotDto {
    fn from(value: svc::BatchTagSnapshot) -> Self {
        Self {
            revision: value.revision.to_string(),
            tags: value.tags.into_iter().map(Into::into).collect(),
            assigned_ids: value.assigned_ids,
            partial_ids: value.partial_ids,
            count: value.count,
        }
    }
}

/// Manage the shared tag catalog, assignments and pending reference associations.
#[module]
pub struct Tags;

#[module]
impl Tags {
    /// Read the global tag catalog, metadata revision, sources and pending associations.
    pub async fn catalog() -> Result<TagSnapshotDto, String> {
        nsh_core::element::blocking(|| {
            svc::tag_snapshot(String::new(), String::new())
                .map(Into::into)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Read the selected record's assigned tag IDs together with the current catalog.
    pub async fn snapshot(target: TagTargetDto) -> Result<TagSnapshotDto, String> {
        nsh_core::element::blocking(move || {
            svc::tag_snapshot_uid(target.library_uid, target.record_uid)
                .map(Into::into)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Apply a revision-checked tag change. Rename, recolor, merge and delete use the tag ID.
    /// Relink moves a pending record's assignments to the selected library and record UUIDs.
    pub async fn mutate(input: TagMutationDto) -> Result<TagSnapshotDto, String> {
        let relocates_source = matches!(input.action, TagActionDto::Relocate);
        let result = nsh_core::element::blocking(move || {
            svc::mutate_tag(input.try_into()?)
                .map(Into::into)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())??;
        super::events::emit("tags");
        if relocates_source {
            super::events::emit("libraries");
        }
        Ok(result)
    }

    /// Read common and partial assignments across stable records from any registered library.
    pub async fn batch_snapshot(targets: Vec<TagTargetDto>) -> Result<BatchTagSnapshotDto, String> {
        nsh_core::element::blocking(move || {
            svc::batch_tag_snapshot(targets.into_iter().map(Into::into).collect())
                .map(Into::into)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Atomically create, assign or unassign a tag for a revision-checked selection.
    pub async fn mutate_batch(input: BatchTagMutationDto) -> Result<BatchTagSnapshotDto, String> {
        let result = nsh_core::element::blocking(move || {
            svc::mutate_batch_tag(input.try_into()?)
                .map(Into::into)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())??;
        super::events::emit("tags");
        Ok(result)
    }
}
