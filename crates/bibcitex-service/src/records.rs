use crate::core::bib::Reference;
use biblatex::{Chunk, EntryType};

#[cfg_attr(feature = "csharp", interoptopus::ffi)]
#[derive(Clone, Debug, serde::Deserialize)]
pub struct LibraryRecord {
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: Option<String>,
}
#[cfg_attr(feature = "csharp", interoptopus::ffi)]
#[derive(Clone, Debug)]
pub enum ChunkKind {
    Normal,
    Verbatim,
    Math,
}
#[cfg_attr(feature = "csharp", interoptopus::ffi)]
#[derive(Clone, Debug)]
pub struct ChunkRecord {
    pub kind: ChunkKind,
    pub text: String,
}
#[cfg_attr(feature = "csharp", interoptopus::ffi)]
#[derive(Clone, Debug)]
pub struct EditorRecord {
    pub name: String,
    pub role: String,
}
#[cfg_attr(feature = "csharp", interoptopus::ffi)]
#[derive(Clone, Debug)]
pub struct PageRange {
    pub start: u32,
    pub end: u32,
}
#[cfg_attr(feature = "csharp", interoptopus::ffi)]
#[derive(Clone, Debug)]
pub struct ReferenceRecord {
    pub id: String,
    pub cite_key: String,
    pub source: String,
    pub entry_type: String,
    pub author: Vec<String>,
    pub title: Vec<ChunkRecord>,
    pub journal: Option<String>,
    pub year: Option<i32>,
    pub full_journal: Option<String>,
    pub volume: Option<i64>,
    pub number: Option<String>,
    pub pages: Option<PageRange>,
    pub note: Vec<ChunkRecord>,
    pub doi: Option<String>,
    pub mrclass: Option<String>,
    pub publisher: Vec<String>,
    pub isbn: Option<String>,
    pub series: Option<String>,
    pub url: Option<String>,
    pub file: Option<String>,
    pub abstract_text: Vec<ChunkRecord>,
    pub edition: Option<i64>,
    pub issue: Vec<ChunkRecord>,
    pub book_pages: Option<String>,
    pub school: Option<String>,
    pub address: Option<String>,
    pub book_title: Vec<ChunkRecord>,
    pub editor: Vec<EditorRecord>,
    pub month: Option<String>,
    pub organization: Vec<String>,
    pub institution: Option<String>,
    pub eprint: Option<String>,
    pub archive_prefix: Option<String>,
    pub arxiv_primary_class: Option<String>,
    pub how_published: Option<String>,
}

fn chunks(value: &Option<Vec<Chunk>>) -> Vec<ChunkRecord> {
    value
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|chunk| ChunkRecord {
            kind: match chunk {
                Chunk::Normal(_) => ChunkKind::Normal,
                Chunk::Verbatim(_) => ChunkKind::Verbatim,
                Chunk::Math(_) => ChunkKind::Math,
            },
            text: chunk.get().to_owned(),
        })
        .collect()
}

impl ReferenceRecord {
    pub(crate) fn from_reference(reference: &Reference, path: &str) -> Result<Self, String> {
        let entry_type = match &reference.type_ {
            EntryType::Unknown(name) => name.clone(),
            entry_type => serde_json::to_value(entry_type)
                .map_err(|e| e.to_string())?
                .as_str()
                .ok_or("Unexpected entry type serialization")?
                .to_owned(),
        };
        Ok(Self {
            id: format!("{}\u{1f}{}", path, reference.cite_key),
            cite_key: reference.cite_key.clone(),
            source: reference.source.clone(),
            entry_type,
            author: reference.author.clone().unwrap_or_default(),
            title: chunks(&reference.title),
            journal: reference.journal.clone(),
            year: reference.year,
            full_journal: reference.full_journal.clone(),
            volume: reference.volume,
            number: reference.number.clone(),
            pages: reference.pages.as_ref().map(|range| PageRange {
                start: range.start,
                end: range.end,
            }),
            note: chunks(&reference.note),
            doi: reference.doi.clone(),
            mrclass: reference.mrclass.clone(),
            publisher: reference.publisher.clone().unwrap_or_default(),
            isbn: reference.isbn.clone(),
            series: reference.series.clone(),
            url: reference.url.clone(),
            file: reference.file.clone(),
            abstract_text: chunks(&reference.abstract_),
            edition: reference.edition,
            issue: chunks(&reference.issue),
            book_pages: reference.book_pages.clone(),
            school: reference.school.clone(),
            address: reference.address.clone(),
            book_title: chunks(&reference.book_title),
            editor: reference
                .editor
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|(name, role)| EditorRecord {
                    name: name.clone(),
                    role: role.clone(),
                })
                .collect(),
            month: reference.month.clone(),
            organization: reference.organization.clone().unwrap_or_default(),
            institution: reference.institution.clone(),
            eprint: reference.eprint.clone(),
            archive_prefix: reference.archive_prefix.clone(),
            arxiv_primary_class: reference.arxiv_primary_class.clone(),
            how_published: reference.how_published.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_record_preserves_enum_math_source_and_absent_fields() {
        let bibliography = biblatex::Bibliography::parse(
            "@article{example,title={A $x^2$ result},author={Doe, Jane},year={2026}}",
        )
        .unwrap();
        let references = crate::core::utils::read_bibliography(bibliography);
        let record = ReferenceRecord::from_reference(&references[0], "/work.bib").unwrap();
        assert_eq!(record.entry_type, "Article");
        assert_eq!(record.id, "/work.bib\u{1f}example");
        assert_eq!(record.year, Some(2026));
        assert_eq!(record.source, references[0].source);
        assert!(
            record
                .title
                .iter()
                .any(|chunk| matches!(chunk.kind, ChunkKind::Math) && chunk.text == "x^2")
        );
        assert!(record.publisher.is_empty());
        assert!(record.editor.is_empty());
        assert!(record.pages.is_none());
    }
}
