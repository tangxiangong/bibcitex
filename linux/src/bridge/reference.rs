//! Search results and display-ready field projections shared by the three search surfaces.
use super::{query::Offsets, tags::TagDto};
use bibcitex_service::{ChunkKind, ChunkRecord, ReferenceRecord, SearchEvidence, SearchMatchRange};
use nsh_core::prelude::*;
use std::borrow::Cow;

/// How a run of bibliography text is to be displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum TextChunkKind {
    /// Plain bibliography text.
    Normal,
    /// Literal text preserving capitalization and punctuation.
    Verbatim,
    /// A LaTeX formula without its enclosing delimiters.
    Math,
}

/// One contiguous source-text run.
#[derive(Debug, Clone, JsObject)]
pub struct TextChunkDto {
    /// Interpretation of the source text.
    pub kind: TextChunkKind,
    /// Original text, including TeX commands for math runs.
    pub text: String,
}

/// An editor of a bibliography entry.
#[derive(Debug, Clone, JsObject)]
pub struct EditorDto {
    /// Formatted editor name.
    pub name: String,
    /// Editorial role as recorded in the bibliography.
    pub role: String,
}

/// One positive match inside an original field, in JavaScript UTF-16 code units.
#[derive(Debug, Clone, JsObject)]
pub struct MatchRangeDto {
    /// Canonical field name.
    pub field: String,
    /// Zero-based nonempty entry within a repeated field.
    pub entry_index: u32,
    /// Inclusive start in the individual field entry.
    pub start: u32,
    /// Exclusive end in the individual field entry.
    pub end: u32,
}

/// A rendered bibliography record with stable identity and its original BibTeX.
#[derive(Debug, Clone, JsObject)]
pub struct ReferenceDto {
    /// Stable reference UUID returned by the tag registry.
    pub record_uid: String,
    /// Stable UUID of the registered library.
    pub library_uid: String,
    /// Original citation key.
    pub cite_key: String,
    /// Lowercase type used by the search catalog.
    pub type_key: String,
    /// Original BibTeX entry type, including unknown types.
    pub display_type: String,
    /// Title with its text and math runs.
    pub title_chunks: Vec<TextChunkDto>,
    /// Author names in source order.
    pub authors: Vec<String>,
    /// Publication year when specified.
    pub year: Option<i32>,
    /// Publication month.
    pub month: Option<String>,
    /// Journal name or abbreviation.
    pub journal: Option<String>,
    /// Full journal name.
    pub full_journal: Option<String>,
    /// Volume number as a lossless decimal string.
    pub volume: Option<String>,
    /// Issue or report number.
    pub number: Option<String>,
    /// Page range in the macOS detail format.
    pub pages_text: String,
    /// Total book pagination.
    pub book_pages: Option<String>,
    /// Publisher names in source order.
    pub publishers: Vec<String>,
    /// Edition number as a lossless decimal string.
    pub edition: Option<String>,
    /// Publication series.
    pub series: Option<String>,
    /// Editors with their original roles.
    pub editors: Vec<EditorDto>,
    /// Degree-granting school.
    pub school: Option<String>,
    /// Place of publication.
    pub address: Option<String>,
    /// Contributing organizations.
    pub organizations: Vec<String>,
    /// Issuing institution.
    pub institution: Option<String>,
    /// Digital object identifier as written.
    pub doi: Option<String>,
    /// Validated HTTP or HTTPS destination for the DOI action.
    pub doi_url: Option<String>,
    /// International Standard Book Number.
    pub isbn: Option<String>,
    /// Mathematical Reviews subject classification.
    pub mrclass: Option<String>,
    /// Original URL field.
    pub url: Option<String>,
    /// Original attachment field, resolved relative to the bibliography.
    pub file: Option<String>,
    /// Electronic preprint identifier.
    pub eprint: Option<String>,
    /// Preprint archive name.
    pub archive_prefix: Option<String>,
    /// Primary arXiv subject class.
    pub arxiv_primary_class: Option<String>,
    /// Original publication method.
    pub how_published: Option<String>,
    /// Preferred venue label using the macOS workbench precedence.
    pub venue_text: String,
    /// Abstract runs.
    pub abstract_chunks: Vec<TextChunkDto>,
    /// Containing book title runs.
    pub book_title_chunks: Vec<TextChunkDto>,
    /// Issue title runs.
    pub issue_chunks: Vec<TextChunkDto>,
    /// Bibliographic note runs.
    pub note_chunks: Vec<TextChunkDto>,
    /// Assigned tags with stable identities and colors.
    pub tags: Vec<TagDto>,
    /// Original BibTeX text for copying.
    pub source: String,
    /// UTF-16 highlights within each matched field entry.
    pub match_ranges: Vec<MatchRangeDto>,
    /// Localized tooltip describing positive matches and spelling supplementation.
    pub match_evidence: String,
}

fn chunks(records: Vec<ChunkRecord>) -> Vec<TextChunkDto> {
    records
        .into_iter()
        .map(|record| TextChunkDto {
            kind: match record.kind {
                ChunkKind::Normal => TextChunkKind::Normal,
                ChunkKind::Verbatim => TextChunkKind::Verbatim,
                ChunkKind::Math => TextChunkKind::Math,
            },
            text: record.text,
        })
        .collect()
}

fn plain(chunks: &[ChunkRecord]) -> String {
    chunks.iter().map(|run| run.text.as_str()).collect()
}

fn venue(record: &ReferenceRecord) -> String {
    for field in [&record.full_journal, &record.journal]
        .into_iter()
        .flatten()
    {
        if !field.is_empty() {
            return field.clone();
        }
    }
    if !record.book_title.is_empty() {
        return plain(&record.book_title);
    }
    if !record.publisher.is_empty() {
        return record.publisher.join(", ");
    }
    for field in [&record.school, &record.institution].into_iter().flatten() {
        if !field.is_empty() {
            return field.clone();
        }
    }
    if !record.organization.is_empty() {
        return record.organization.join(", ");
    }
    record.how_published.clone().unwrap_or_default()
}

pub(super) fn web_url(value: &str) -> Option<String> {
    let url = url::Url::parse(value.trim()).ok()?;
    (matches!(url.scheme(), "https" | "http") && url.has_host()).then(|| url.into())
}

pub(super) fn doi_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if url::Url::parse(value).is_ok() {
        return web_url(value);
    }
    web_url(&format!("https://doi.org/{value}"))
}

// Match the ordering and empty-entry filtering in AdvancedIndex::new. In particular,
// full_journal is the second "journal" entry only if the abbreviated one exists.
fn field_text<'a>(record: &'a ReferenceRecord, field: &str, index: usize) -> Option<Cow<'a, str>> {
    let value = match field {
        "key" => Cow::Borrowed(record.cite_key.as_str()),
        "title" => Cow::Owned(plain(&record.title)),
        "abstract" => Cow::Owned(plain(&record.abstract_text)),
        "booktitle" => Cow::Owned(plain(&record.book_title)),
        "note" => Cow::Owned(plain(&record.note)),
        "year" => Cow::Owned(record.year?.to_string()),
        "school" => Cow::Borrowed(record.school.as_deref()?),
        "institution" => Cow::Borrowed(record.institution.as_deref()?),
        "doi" => Cow::Borrowed(record.doi.as_deref()?),
        "isbn" => Cow::Borrowed(record.isbn.as_deref()?),
        _ => {
            let entry = match field {
                "editor" => record
                    .editor
                    .iter()
                    .map(|e| e.name.as_str())
                    .filter(|s| !s.is_empty())
                    .nth(index),
                "journal" => [record.journal.as_deref(), record.full_journal.as_deref()]
                    .into_iter()
                    .flatten()
                    .filter(|s| !s.is_empty())
                    .nth(index),
                _ => {
                    let multi = match field {
                        "author" => &record.author,
                        "publisher" => &record.publisher,
                        "organization" => &record.organization,
                        _ => return None,
                    };
                    multi
                        .iter()
                        .map(String::as_str)
                        .filter(|s| !s.is_empty())
                        .nth(index)
                }
            };
            return entry.map(Cow::Borrowed);
        }
    };
    (index == 0).then_some(value)
}

fn highlights(
    record: &ReferenceRecord,
    ranges: &[SearchMatchRange],
) -> Result<Vec<MatchRangeDto>, String> {
    let mut result = Vec::with_capacity(ranges.len());
    // The shared index sorts ranges by field and entry. Reuse concatenated text
    // and its UTF-16 conversion for repeated highlights in long abstracts.
    for group in
        ranges.chunk_by(|a, b| a.field_name == b.field_name && a.entry_index == b.entry_index)
    {
        let first = &group[0];
        let source = field_text(record, &first.field_name, first.entry_index as usize)
            .ok_or_else(|| format!("Unknown evidence field: {}", first.field_name))?;
        let offsets = Offsets::new(&source);
        for range in group {
            result.push(MatchRangeDto {
                field: range.field_name.clone(),
                entry_index: range.entry_index,
                start: offsets.units(range.start)?,
                end: offsets.units(range.end)?,
            });
        }
    }
    Ok(result)
}

impl ReferenceDto {
    pub(super) fn from_record(
        r: ReferenceRecord,
        evidence: Option<SearchEvidence>,
        spelling_label: &str,
    ) -> Result<Self, String> {
        let ranges = evidence.as_ref().map_or(&[][..], |e| e.ranges.as_slice());
        let match_ranges = highlights(&r, ranges)?;
        Ok(Self {
            venue_text: venue(&r),
            doi_url: r.doi.as_deref().and_then(doi_url),
            record_uid: r.id,
            library_uid: r.library_uid,
            cite_key: r.cite_key,
            type_key: r.entry_type.to_ascii_lowercase(),
            display_type: r.entry_type,
            title_chunks: chunks(r.title),
            authors: r.author,
            year: r.year,
            month: r.month,
            journal: r.journal,
            full_journal: r.full_journal,
            volume: r.volume.map(|n| n.to_string()),
            number: r.number,
            pages_text: r
                .pages
                .map(|p| format!("{}-{}", p.start, p.end))
                .unwrap_or_default(),
            book_pages: r.book_pages,
            publishers: r.publisher,
            edition: r.edition.map(|n| n.to_string()),
            series: r.series,
            editors: r
                .editor
                .into_iter()
                .map(|e| EditorDto {
                    name: e.name,
                    role: e.role,
                })
                .collect(),
            school: r.school,
            address: r.address,
            organizations: r.organization,
            institution: r.institution,
            doi: r.doi,
            isbn: r.isbn,
            mrclass: r.mrclass,
            url: r.url,
            file: r.file,
            eprint: r.eprint,
            archive_prefix: r.archive_prefix,
            arxiv_primary_class: r.arxiv_primary_class,
            how_published: r.how_published,
            abstract_chunks: chunks(r.abstract_text),
            book_title_chunks: chunks(r.book_title),
            issue_chunks: chunks(r.issue),
            note_chunks: chunks(r.note),
            tags: r.tags.into_iter().map(Into::into).collect(),
            source: r.source,
            match_evidence: evidence
                .map(|e| {
                    let fields = e.fields.join(", ");
                    if e.fuzzy_terms.is_empty() {
                        return fields;
                    }
                    let spelling = format!("{spelling_label}: {}", e.fuzzy_terms.join(", "));
                    if fields.is_empty() {
                        spelling
                    } else {
                        format!("{fields} · {spelling}")
                    }
                })
                .unwrap_or_default(),
            match_ranges,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn article() -> ReferenceRecord {
        ReferenceRecord {
            library_uid: String::new(),
            tags: vec![],
            id: String::new(),
            cite_key: String::new(),
            source: String::new(),
            entry_type: "Article".into(),
            author: vec!["A".into(), "李🚀".into()],
            title: vec![
                ChunkRecord {
                    kind: ChunkKind::Normal,
                    text: "中🚀".into(),
                },
                ChunkRecord {
                    kind: ChunkKind::Math,
                    text: "x".into(),
                },
            ],
            journal: Some("J".into()),
            year: None,
            full_journal: Some("🚀期刊".into()),
            volume: None,
            number: None,
            pages: Some(bibcitex_service::PageRange { start: 3, end: 9 }),
            note: vec![],
            doi: None,
            mrclass: None,
            publisher: vec![],
            isbn: None,
            series: None,
            url: None,
            file: None,
            abstract_text: vec![],
            edition: Some(i64::MAX),
            issue: vec![],
            book_pages: None,
            school: None,
            address: None,
            book_title: vec![],
            editor: vec![],
            month: None,
            organization: vec![],
            institution: None,
            eprint: None,
            archive_prefix: None,
            arxiv_primary_class: None,
            how_published: None,
        }
    }
    #[test]
    fn unicode_highlights_use_the_original_field_and_entry() {
        let row = article();
        let ranges = vec![
            SearchMatchRange {
                field_name: "author".into(),
                entry_index: 1,
                start: 3,
                end: 7,
            },
            SearchMatchRange {
                field_name: "journal".into(),
                entry_index: 1,
                start: 4,
                end: 10,
            },
            SearchMatchRange {
                field_name: "title".into(),
                entry_index: 0,
                start: 7,
                end: 8,
            },
        ];
        let evidence = SearchEvidence {
            ranges,
            reference_id: String::new(),
            fields: vec!["title".into()],
            fuzzy: false,
            fuzzy_terms: vec![],
        };
        let dto = ReferenceDto::from_record(row, Some(evidence), "拼写补充").unwrap();
        assert_eq!(
            dto.match_ranges
                .iter()
                .map(|m| (m.start, m.end))
                .collect::<Vec<_>>(),
            [(1, 3), (2, 4), (3, 4)]
        );
        assert_eq!(dto.match_evidence, "title");
        assert_eq!(dto.venue_text, "🚀期刊");
        assert_eq!(dto.title_chunks[1].kind, TextChunkKind::Math);
        assert_eq!(dto.edition.as_deref(), Some("9223372036854775807"));
        assert_eq!(dto.pages_text, "3-9");
    }
    #[test]
    fn doi_and_url_actions_only_open_web_addresses() {
        assert_eq!(
            doi_url("10.1/论文").as_deref(),
            Some("https://doi.org/10.1/%E8%AE%BA%E6%96%87")
        );
        assert_eq!(
            doi_url("https://doi.org/10.1/a").as_deref(),
            Some("https://doi.org/10.1/a")
        );
        assert_eq!(web_url("file:///etc/hosts"), None);
        assert_eq!(doi_url("javascript:alert(1)"), None);
        assert_eq!(doi_url(" "), None);
    }
}
