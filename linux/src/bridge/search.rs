//! Per-window search results and context-checked reference actions.
use super::{
    paste::{self, PastePlan},
    query::{AdvancedSearchRequestDto, QueryAnalysisDto, QueryDocumentDto, SearchContextDto},
    reference::{ReferenceDto, doi_url, web_url},
    tags::TagTargetDto,
};
use bibcitex_service as service;
use nsh_core::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

const STALE: &str = "Search results changed; search again before continuing";

/// The surface that owns an independent query and active result set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum SearchSurface {
    /// The main three-pane workbench.
    Main,
    /// The compact cross-application paste helper.
    Helper,
    /// The tray's independent workbench.
    Tray,
}

/// What to put on the clipboard for selected references.
#[derive(Debug, Clone, Copy, PropEnum)]
pub enum CopyKind {
    /// Format citation keys with the current citation preferences.
    Citation,
    /// Copy the original entries, separated by a blank line.
    Bibtex,
}

/// The reference field to open with its native handler.
#[derive(Debug, Clone, Copy, PropEnum)]
pub enum ReferenceLink {
    /// Open a DOI in the default browser.
    Doi,
    /// Open the entry's HTTP(S) URL.
    Url,
    /// Open its local attachment relative to the bibliography.
    File,
}

/// Stable identities selected in a particular search result.
#[derive(Debug, Clone, JsObject)]
pub struct SelectionDto {
    /// Exact context returned by search.
    pub context: SearchContextDto,
    /// Selected library/record pairs. Actions preserve result-list order.
    pub targets: Vec<TagTargetDto>,
}

/// Analysis and rows returned by one search generation.
#[derive(Debug, Clone, JsObject)]
pub struct SearchReplyDto {
    /// Effective analysis, including any refreshed bound-tag names.
    pub analysis: QueryAnalysisDto,
    /// References in deterministic result-list order.
    pub references: Vec<ReferenceDto>,
    /// Identity for actions, absent for invalid or stale results.
    pub context: Option<SearchContextDto>,
}

#[derive(Clone)]
struct ActionRow {
    library_uid: String,
    record_uid: String,
    cite_key: String,
    source: String,
    doi: Option<String>,
    url: Option<String>,
    file: Option<String>,
}

impl From<&service::ReferenceRecord> for ActionRow {
    fn from(r: &service::ReferenceRecord) -> Self {
        Self {
            library_uid: r.library_uid.clone(),
            record_uid: r.id.clone(),
            cite_key: r.cite_key.clone(),
            source: r.source.clone(),
            doi: r.doi.clone(),
            url: r.url.clone(),
            file: r.file.clone(),
        }
    }
}

struct Snapshot {
    context: service::SearchContext,
    rows: Vec<ActionRow>,
}

impl Snapshot {
    fn selected(&self, selection: &SelectionDto) -> Result<Vec<ActionRow>, String> {
        if SearchContextDto::from(self.context.clone()) != selection.context {
            return Err(STALE.into());
        }
        let mut requested: HashSet<_> = selection
            .targets
            .iter()
            .map(|t| (t.library_uid.as_str(), t.record_uid.as_str()))
            .collect();
        if requested.is_empty() {
            return Err("Select at least one reference".into());
        }
        let rows = self
            .rows
            .iter()
            .filter(|row| requested.remove(&(row.library_uid.as_str(), row.record_uid.as_str())))
            .cloned()
            .collect();
        if !requested.is_empty() {
            return Err(STALE.into());
        }
        Ok(rows)
    }
}

fn execute(
    request: service::AdvancedSearchRequest,
    filter: Option<service::QueryDocumentRecord>,
    spelling_label: String,
) -> Result<(SearchReplyDto, Option<Snapshot>), String> {
    let search = |request| {
        if let Some(filter) = &filter {
            service::filtered_search(request, filter.clone())
        } else {
            service::advanced_search(request)
        }
    };
    let mut response = search(request.clone()).map_err(|e| e.to_string())?;
    if response.analysis.status == service::QueryStatus::StaleQuery
        && let Some(document) = request.document.clone()
    {
        let updated = service::refresh_query(document).map_err(|e| e.to_string())?;
        if updated.status == service::QueryStatus::Valid {
            response = search(service::AdvancedSearchRequest {
                text: updated.text,
                document: updated.document,
                expected_tag_revision: updated.tag_revision,
                ..request
            })
            .map_err(|e| e.to_string())?;
        } else {
            response.analysis = updated;
            response.references.clear();
            response.evidence.clear();
            response.context = None;
        }
    }
    let snapshot = response.context.clone().map(|context| Snapshot {
        context,
        rows: response.references.iter().map(ActionRow::from).collect(),
    });
    let mut evidence: HashMap<_, _> = response
        .evidence
        .into_iter()
        .map(|e| (e.reference_id.clone(), e))
        .collect();
    let references = response
        .references
        .into_iter()
        .map(|r| {
            let evidence = evidence.remove(&r.id);
            ReferenceDto::from_record(r, evidence, &spelling_label)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        SearchReplyDto {
            analysis: response.analysis.try_into()?,
            references,
            context: response.context.map(Into::into),
        },
        snapshot,
    ))
}

/// One search surface's immutable action snapshot and in-flight generation.
#[class]
pub struct SearchSession {
    surface: SearchSurface,
    generation: u64,
    snapshot: Option<Snapshot>,
}

#[class]
impl SearchSession {
    /// Creates a search session for one workbench or compact helper.
    pub fn new(surface: SearchSurface) -> Self {
        Self {
            surface,
            generation: 0,
            snapshot: None,
        }
    }
    /// Drops the previous action context and rejects any in-flight search.
    pub fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if let Some(snapshot) = self.snapshot.take()
            && self.surface == SearchSurface::Helper
        {
            paste::invalidate(&snapshot.context);
        }
    }
    /// Runs a search with an optional independent tag filter. A superseded result rejects.
    pub async fn search(
        this: Instance<Self>,
        input: AdvancedSearchRequestDto,
        filter: Option<QueryDocumentDto>,
    ) -> Result<SearchReplyDto, String> {
        let generation = this.with_mut(|session| {
            session.invalidate();
            session.generation
        });
        let spelling_label = nsh_core::t!(&crate::locale_key::key("拼写补充"));
        let (reply, snapshot) = blocking(move || {
            execute(
                input.try_into()?,
                filter.map(TryInto::try_into).transpose()?,
                spelling_label,
            )
        })
        .await
        .map_err(|e| e.to_string())??;
        ensure_generation(&this, generation)?;
        this.with_mut(|session| session.snapshot = snapshot);
        Ok(reply)
    }
    /// Revalidates a selection and copies citations or complete BibTeX entries.
    pub async fn copy(
        this: Instance<Self>,
        selection: SelectionDto,
        kind: CopyKind,
    ) -> Result<(), String> {
        let (_, text) = formatted(&this, selection, kind).await?;
        paste::copy_text(text).await
    }
    /// Revalidates a single reference before opening its DOI, URL, or local attachment.
    pub async fn open_reference(
        this: Instance<Self>,
        selection: SelectionDto,
        kind: ReferenceLink,
    ) -> Result<(), String> {
        let (generation, context, rows) = selected(&this, &selection)?;
        let [row] = rows.as_slice() else {
            return Err("Select one reference".into());
        };
        validate(&this, generation, context.clone()).await?;
        if let ReferenceLink::File = kind {
            let file = row.file.as_deref().ok_or("Missing attachment")?;
            let path = crate::files::attachment_path(
                file,
                Path::new(&context.path),
                dirs::home_dir().as_deref(),
            )?;
            open_path(path).await
        } else {
            let url = match kind {
                ReferenceLink::Doi => row.doi.as_deref().and_then(doi_url),
                _ => row.url.as_deref().and_then(web_url),
            }
            .ok_or("Invalid reference URL")?;
            if open_url(url).await {
                Ok(())
            } else {
                Err("Unable to open reference URL".into())
            }
        }
    }
    /// Checks the captured external target and prepares a one-time paste of this selection.
    pub async fn prepare_paste(
        this: Instance<Self>,
        selection: SelectionDto,
        target: String,
    ) -> Result<PastePlan, String> {
        if !this.with(|s| s.surface == SearchSurface::Helper) {
            return Err("Paste requires a helper search session".into());
        }
        let (context, text) = formatted(&this, selection, CopyKind::Citation).await?;
        paste::prepare(target, text, context).await
    }
}

fn selected(
    this: &Instance<SearchSession>,
    selection: &SelectionDto,
) -> Result<(u64, service::SearchContext, Vec<ActionRow>), String> {
    this.with(|session| {
        let snapshot = session.snapshot.as_ref().ok_or(STALE)?;
        Ok((
            session.generation,
            snapshot.context.clone(),
            snapshot.selected(selection)?,
        ))
    })
}

fn ensure_generation(this: &Instance<SearchSession>, generation: u64) -> Result<(), String> {
    if this.is_disposed() || this.with(|s| s.generation != generation) {
        Err(STALE.into())
    } else {
        Ok(())
    }
}

async fn validate(
    this: &Instance<SearchSession>,
    generation: u64,
    context: service::SearchContext,
) -> Result<(), String> {
    let current = blocking(move || service::validate_search_context(context))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    if !current {
        return Err(STALE.into());
    }
    ensure_generation(this, generation)
}

async fn formatted(
    this: &Instance<SearchSession>,
    selection: SelectionDto,
    kind: CopyKind,
) -> Result<(service::SearchContext, String), String> {
    let (generation, context, rows) = selected(this, &selection)?;
    let text = blocking(move || -> Result<String, String> {
        match kind {
            CopyKind::Bibtex => Ok(rows
                .into_iter()
                .map(|r| r.source)
                .collect::<Vec<_>>()
                .join("\n\n")),
            CopyKind::Citation => {
                let p = crate::preferences::Preferences::load()?;
                let keys = rows.into_iter().map(|r| r.cite_key).collect::<Vec<_>>();
                Ok(service::format_citation(
                    &keys,
                    &p.citation_format,
                    &p.citation_mode,
                ))
            }
        }
    })
    .await
    .map_err(|e| e.to_string())??;
    validate(this, generation, context.clone()).await?;
    Ok((context, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(id: &str) -> ActionRow {
        ActionRow {
            library_uid: "library".into(),
            record_uid: id.into(),
            cite_key: "same".into(),
            source: id.into(),
            doi: None,
            url: None,
            file: None,
        }
    }
    fn target(id: &str) -> TagTargetDto {
        TagTargetDto {
            library_uid: "library".into(),
            record_uid: id.into(),
        }
    }
    #[test]
    fn selection_orders_by_result_identity_and_rejects_stale_inputs() {
        let snapshot = Snapshot {
            context: service::SearchContext {
                path: "library.bib".into(),
                file_version: "digest".into(),
                tag_revision: Some(123),
                query_version: 10,
            },
            rows: vec![row("one"), row("two")],
        };
        let mut selected = SelectionDto {
            context: snapshot.context.clone().into(),
            targets: vec![target("two"), target("one"), target("one")],
        };
        assert_eq!(
            snapshot
                .selected(&selected)
                .unwrap()
                .iter()
                .map(|r| r.record_uid.as_str())
                .collect::<Vec<_>>(),
            ["one", "two"]
        );
        selected.targets[0].library_uid = "other library".into();
        assert!(snapshot.selected(&selected).is_err());
        selected.targets = vec![target("one")];
        selected.context.query_version = "11".into();
        assert!(snapshot.selected(&selected).is_err());
    }
}
