// UniFFI's remote derives register the shared types without redefining their layout.
pub use bibcitex_service::{
    ChunkKind, ChunkRecord, CoreError, EditorRecord, LibraryRecord, PageRange, ReferenceRecord,
};
#[uniffi::remote(Error)]
enum CoreError {
    Operation { message: String },
}
#[uniffi::remote(Record)]
pub struct LibraryRecord {
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: Option<String>,
    pub pinned: bool,
    pub available: bool,
}
#[uniffi::remote(Enum)]
pub enum ChunkKind {
    Normal,
    Verbatim,
    Math,
}
#[uniffi::remote(Record)]
pub struct ChunkRecord {
    pub kind: ChunkKind,
    pub text: String,
}
#[uniffi::remote(Record)]
pub struct EditorRecord {
    pub name: String,
    pub role: String,
}
#[uniffi::remote(Record)]
pub struct PageRange {
    pub start: u32,
    pub end: u32,
}
#[uniffi::remote(Record)]
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
