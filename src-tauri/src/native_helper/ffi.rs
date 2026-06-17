use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::slice;

use biblatex::{Chunk, EntryType};

use crate::core::bib::Reference;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiString {
    pub ptr: *mut c_char,
    pub len: usize,
}

impl FfiString {
    pub fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiStringArray {
    pub has_value: bool,
    pub ptr: *mut FfiString,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiEditor {
    pub name: FfiString,
    pub role: FfiString,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiEditorArray {
    pub has_value: bool,
    pub ptr: *mut FfiEditor,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum FfiChunkKind {
    Normal = 0,
    Verbatim = 1,
    Math = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiChunk {
    pub kind: FfiChunkKind,
    pub text: FfiString,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiChunkArray {
    pub has_value: bool,
    pub ptr: *mut FfiChunk,
    pub len: usize,
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq)]
pub enum FfiEntryTypeKind {
    Article = 0,
    Book = 1,
    Booklet = 2,
    InBook = 3,
    InCollection = 4,
    InProceedings = 5,
    Manual = 6,
    MastersThesis = 7,
    PhdThesis = 8,
    Misc = 9,
    Proceedings = 10,
    TechReport = 11,
    Unpublished = 12,
    MvBook = 13,
    BookInBook = 14,
    SuppBook = 15,
    Periodical = 16,
    SuppPeriodical = 17,
    Collection = 18,
    MvCollection = 19,
    SuppCollection = 20,
    Reference = 21,
    MvReference = 22,
    InReference = 23,
    MvProceedings = 24,
    Report = 25,
    Patent = 26,
    Thesis = 27,
    Online = 28,
    Software = 29,
    Dataset = 30,
    Set = 31,
    XData = 32,
    Unknown = 255,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiEntryType {
    pub kind: FfiEntryTypeKind,
    pub unknown: FfiString,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiRange {
    pub has_value: bool,
    pub start: u32,
    pub end: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiReference {
    pub cite_key: FfiString,
    pub source: FfiString,
    pub type_: FfiEntryType,
    pub author: FfiStringArray,
    pub title: FfiChunkArray,
    pub journal: FfiString,
    pub year: i32,
    pub has_year: bool,
    pub full_journal: FfiString,
    pub volume: i64,
    pub has_volume: bool,
    pub number: FfiString,
    pub pages: FfiRange,
    pub note: FfiChunkArray,
    pub doi: FfiString,
    pub mrclass: FfiString,
    pub publisher: FfiStringArray,
    pub series: FfiString,
    pub isbn: FfiString,
    pub url: FfiString,
    pub file: FfiString,
    pub abstract_: FfiChunkArray,
    pub edition: i64,
    pub has_edition: bool,
    pub issue: FfiChunkArray,
    pub book_pages: FfiString,
    pub school: FfiString,
    pub address: FfiString,
    pub book_title: FfiChunkArray,
    pub editor: FfiEditorArray,
    pub month: FfiString,
    pub organization: FfiStringArray,
    pub institution: FfiString,
    pub eprint: FfiString,
    pub archive_prefix: FfiString,
    pub arxiv_primary_class: FfiString,
    pub how_published: FfiString,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiReferenceArray {
    pub has_value: bool,
    pub ptr: *mut FfiReference,
    pub len: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiBibliography {
    pub name: FfiString,
    pub path: FfiString,
    pub updated_at: FfiString,
    pub description: FfiString,
    pub has_description: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FfiBibliographyArray {
    pub has_value: bool,
    pub ptr: *mut FfiBibliography,
    pub len: usize,
}

pub fn to_ffi_string(value: Option<String>) -> FfiString {
    match value {
        Some(v) => {
            let bytes = match CString::new(v) {
                Ok(raw) => raw,
                Err(_) => CString::new("").expect("CString empty should never fail"),
            };
            let len = bytes.as_bytes().len();
            let ptr = bytes.into_raw();
            FfiString { ptr, len }
        }
        None => FfiString::null(),
    }
}

pub fn to_ffi_owned_string(value: String) -> FfiString {
    to_ffi_string(Some(value))
}

fn to_ffi_string_array(values: &[String]) -> FfiStringArray {
    if values.is_empty() {
        return FfiStringArray {
            has_value: false,
            ptr: ptr::null_mut(),
            len: 0,
        };
    }

    let mapped = values
        .iter()
        .map(|v| to_ffi_owned_string(v.clone()))
        .collect::<Vec<_>>();
    let mut boxed = mapped.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    FfiStringArray {
        has_value: true,
        ptr,
        len,
    }
}

fn to_ffi_editor_array(values: &[(String, String)]) -> FfiEditorArray {
    if values.is_empty() {
        return FfiEditorArray {
            has_value: false,
            ptr: ptr::null_mut(),
            len: 0,
        };
    }

    let mapped = values
        .iter()
        .map(|(name, role)| FfiEditor {
            name: to_ffi_owned_string(name.clone()),
            role: to_ffi_owned_string(role.clone()),
        })
        .collect::<Vec<_>>();
    let mut boxed = mapped.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    FfiEditorArray {
        has_value: true,
        ptr,
        len,
    }
}

fn to_ffi_chunk_array(chunks: &[Chunk]) -> FfiChunkArray {
    if chunks.is_empty() {
        return FfiChunkArray {
            has_value: false,
            ptr: ptr::null_mut(),
            len: 0,
        };
    }

    let mapped = chunks
        .iter()
        .map(|chunk| {
            let kind = match chunk {
                Chunk::Normal(_) => FfiChunkKind::Normal,
                Chunk::Verbatim(_) => FfiChunkKind::Verbatim,
                Chunk::Math(_) => FfiChunkKind::Math,
            };
            FfiChunk {
                kind,
                text: to_ffi_owned_string(chunk.get().to_string()),
            }
        })
        .collect::<Vec<_>>();
    let mut boxed = mapped.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    FfiChunkArray {
        has_value: true,
        ptr,
        len,
    }
}

fn to_entry_type(entry_type: &EntryType) -> FfiEntryType {
    let value = entry_type.to_string();
    let kind = match value.as_str() {
        "Article" => FfiEntryTypeKind::Article,
        "Book" => FfiEntryTypeKind::Book,
        "Booklet" => FfiEntryTypeKind::Booklet,
        "InBook" => FfiEntryTypeKind::InBook,
        "InCollection" => FfiEntryTypeKind::InCollection,
        "InProceedings" => FfiEntryTypeKind::InProceedings,
        "Manual" => FfiEntryTypeKind::Manual,
        "MastersThesis" => FfiEntryTypeKind::MastersThesis,
        "PhdThesis" => FfiEntryTypeKind::PhdThesis,
        "Misc" => FfiEntryTypeKind::Misc,
        "Proceedings" => FfiEntryTypeKind::Proceedings,
        "TechReport" => FfiEntryTypeKind::TechReport,
        "Thesis" => FfiEntryTypeKind::Thesis,
        "Unpublished" => FfiEntryTypeKind::Unpublished,
        "MvBook" => FfiEntryTypeKind::MvBook,
        "BookInBook" => FfiEntryTypeKind::BookInBook,
        "SuppBook" => FfiEntryTypeKind::SuppBook,
        "Periodical" => FfiEntryTypeKind::Periodical,
        "SuppPeriodical" => FfiEntryTypeKind::SuppPeriodical,
        "Collection" => FfiEntryTypeKind::Collection,
        "MvCollection" => FfiEntryTypeKind::MvCollection,
        "SuppCollection" => FfiEntryTypeKind::SuppCollection,
        "Reference" => FfiEntryTypeKind::Reference,
        "MvReference" => FfiEntryTypeKind::MvReference,
        "InReference" => FfiEntryTypeKind::InReference,
        "MvProceedings" => FfiEntryTypeKind::MvProceedings,
        "Report" => FfiEntryTypeKind::Report,
        "Patent" => FfiEntryTypeKind::Patent,
        "Online" => FfiEntryTypeKind::Online,
        "Software" => FfiEntryTypeKind::Software,
        "Dataset" => FfiEntryTypeKind::Dataset,
        "Set" => FfiEntryTypeKind::Set,
        "XData" => FfiEntryTypeKind::XData,
        _ => FfiEntryTypeKind::Unknown,
    };
    FfiEntryType {
        kind,
        unknown: if kind == FfiEntryTypeKind::Unknown {
            to_ffi_owned_string(value)
        } else {
            FfiString::null()
        },
    }
}

pub fn to_ffi_reference(reference: &Reference) -> FfiReference {
    FfiReference {
        cite_key: to_ffi_owned_string(reference.cite_key.clone()),
        source: to_ffi_owned_string(reference.source.clone()),
        type_: to_entry_type(&reference.type_),
        author: to_ffi_string_array(reference.author.as_deref().unwrap_or(&[])),
        title: to_ffi_chunk_array(reference.title.as_deref().unwrap_or(&[])),
        journal: to_ffi_string(reference.journal.clone()),
        year: reference.year.unwrap_or(0),
        has_year: reference.year.is_some(),
        full_journal: to_ffi_string(reference.full_journal.clone()),
        volume: reference.volume.unwrap_or(0),
        has_volume: reference.volume.is_some(),
        number: to_ffi_string(reference.number.clone()),
        pages: match &reference.pages {
            Some(pages) => FfiRange {
                has_value: true,
                start: pages.start,
                end: pages.end,
            },
            None => FfiRange {
                has_value: false,
                start: 0,
                end: 0,
            },
        },
        note: to_ffi_chunk_array(reference.note.as_deref().unwrap_or(&[])),
        doi: to_ffi_string(reference.doi.clone()),
        mrclass: to_ffi_string(reference.mrclass.clone()),
        publisher: to_ffi_string_array(reference.publisher.as_deref().unwrap_or(&[])),
        series: to_ffi_string(reference.series.clone()),
        isbn: to_ffi_string(reference.isbn.clone()),
        url: to_ffi_string(reference.url.clone()),
        file: to_ffi_string(reference.file.clone()),
        abstract_: to_ffi_chunk_array(reference.abstract_.as_deref().unwrap_or(&[])),
        edition: reference.edition.unwrap_or(0),
        has_edition: reference.edition.is_some(),
        issue: to_ffi_chunk_array(reference.issue.as_deref().unwrap_or(&[])),
        book_pages: to_ffi_string(reference.book_pages.clone()),
        school: to_ffi_string(reference.school.clone()),
        address: to_ffi_string(reference.address.clone()),
        book_title: to_ffi_chunk_array(reference.book_title.as_deref().unwrap_or(&[])),
        editor: to_ffi_editor_array(reference.editor.as_deref().unwrap_or(&[])),
        month: to_ffi_string(reference.month.clone()),
        organization: to_ffi_string_array(reference.organization.as_deref().unwrap_or(&[])),
        institution: to_ffi_string(reference.institution.clone()),
        eprint: to_ffi_string(reference.eprint.clone()),
        archive_prefix: to_ffi_string(reference.archive_prefix.clone()),
        arxiv_primary_class: to_ffi_string(reference.arxiv_primary_class.clone()),
        how_published: to_ffi_string(reference.how_published.clone()),
    }
}

pub fn references_to_ffi_array(references: Vec<Reference>) -> FfiReferenceArray {
    if references.is_empty() {
        return FfiReferenceArray {
            has_value: false,
            ptr: ptr::null_mut(),
            len: 0,
        };
    }

    let mapped = references
        .into_iter()
        .map(|reference| to_ffi_reference(&reference))
        .collect::<Vec<_>>();
    let mut boxed = mapped.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    FfiReferenceArray {
        has_value: true,
        ptr,
        len,
    }
}

pub fn to_ffi_bibliography(
    name: String,
    path: String,
    updated_at: String,
    description: Option<String>,
) -> FfiBibliography {
    let has_description = description.is_some();
    FfiBibliography {
        name: to_ffi_owned_string(name),
        path: to_ffi_owned_string(path),
        updated_at: to_ffi_owned_string(updated_at),
        description: to_ffi_string(description),
        has_description,
    }
}

pub fn bibliography_to_ffi_array(
    bibliographies: Vec<(String, String, String, Option<String>)>,
) -> FfiBibliographyArray {
    if bibliographies.is_empty() {
        return FfiBibliographyArray {
            has_value: false,
            ptr: ptr::null_mut(),
            len: 0,
        };
    }

    let mapped = bibliographies
        .into_iter()
        .map(|(name, path, updated_at, description)| {
            to_ffi_bibliography(name, path, updated_at, description)
        })
        .collect::<Vec<_>>();
    let mut boxed = mapped.into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    FfiBibliographyArray {
        has_value: true,
        ptr,
        len,
    }
}

fn free_string(value: FfiString) {
    if !value.ptr.is_null() {
        // SAFETY: `ptr` is created by CString::into_raw in this module and freed only once here.
        drop(unsafe { CString::from_raw(value.ptr) });
    }
}

fn free_string_array(value: FfiStringArray) {
    if !value.has_value || value.ptr.is_null() {
        return;
    }

    let len = value.len;
    let slice = unsafe { slice::from_raw_parts_mut(value.ptr, len) };
    for item in &mut *slice {
        free_string(*item);
    }
    drop(unsafe { Box::from_raw(slice) });
}

fn free_editor_array(value: FfiEditorArray) {
    if !value.has_value || value.ptr.is_null() {
        return;
    }

    let len = value.len;
    let slice = unsafe { slice::from_raw_parts_mut(value.ptr, len) };
    for item in &mut *slice {
        free_string(item.name);
        free_string(item.role);
    }
    drop(unsafe { Box::from_raw(slice) });
}

fn free_chunk_array(value: FfiChunkArray) {
    if !value.has_value || value.ptr.is_null() {
        return;
    }

    let len = value.len;
    let slice = unsafe { slice::from_raw_parts_mut(value.ptr, len) };
    for item in &mut *slice {
        free_string(item.text);
    }
    drop(unsafe { Box::from_raw(slice) });
}

fn free_bibliography(value: FfiBibliography) {
    free_string(value.name);
    free_string(value.path);
    free_string(value.updated_at);
    free_string(value.description);
}

fn free_bibliography_array(value: FfiBibliographyArray) {
    if !value.has_value || value.ptr.is_null() {
        return;
    }

    let len = value.len;
    let slice = unsafe { slice::from_raw_parts_mut(value.ptr, len) };
    for item in &mut *slice {
        free_bibliography(*item);
    }
    drop(unsafe { Box::from_raw(slice) });
}

fn free_reference(value: FfiReference) {
    free_string(value.cite_key);
    free_string(value.source);
    free_string(value.journal);
    free_string(value.full_journal);
    free_string(value.number);
    free_string(value.doi);
    free_string(value.mrclass);
    free_string(value.series);
    free_string(value.isbn);
    free_string(value.url);
    free_string(value.file);
    free_string(value.book_pages);
    free_string(value.school);
    free_string(value.address);
    free_string(value.month);
    free_string(value.institution);
    free_string(value.eprint);
    free_string(value.archive_prefix);
    free_string(value.arxiv_primary_class);
    free_string(value.how_published);
    free_string(value.type_.unknown);

    free_string_array(value.author);
    free_chunk_array(value.title);
    free_chunk_array(value.note);
    free_string_array(value.publisher);
    free_chunk_array(value.abstract_);
    free_chunk_array(value.issue);
    free_chunk_array(value.book_title);
    free_editor_array(value.editor);
    free_string_array(value.organization);
}

fn free_reference_array(value: FfiReferenceArray) {
    if !value.has_value || value.ptr.is_null() {
        return;
    }

    let len = value.len;
    let slice = unsafe { slice::from_raw_parts_mut(value.ptr, len) };
    for item in &mut *slice {
        free_reference(*item);
    }
    drop(unsafe { Box::from_raw(slice) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_string(value: *const FfiString) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_string(*value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_string_array(value: *const FfiStringArray) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_string_array(*value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_editor_array(value: *const FfiEditorArray) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_editor_array(*value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_chunk_array(value: *const FfiChunkArray) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_chunk_array(*value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_bibliography(value: *const FfiBibliography) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_bibliography(*value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_bibliography_array(value: *const FfiBibliographyArray) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_bibliography_array(*value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_reference(value: *const FfiReference) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_reference(*value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_free_reference_array(value: *const FfiReferenceArray) {
    if let Some(value) = unsafe { value.as_ref() } {
        free_reference_array(*value);
    }
}

pub fn cstring_to_str(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    // SAFETY: caller must provide valid null-terminated C string pointer.
    let c_str = unsafe { CStr::from_ptr(ptr) };
    Some(c_str.to_string_lossy().into_owned())
}

pub fn bool_to_c_int(value: bool) -> c_int {
    if value { 1 } else { 0 }
}
