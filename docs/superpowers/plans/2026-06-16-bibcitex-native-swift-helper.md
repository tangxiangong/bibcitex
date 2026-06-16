# BibCiTeX Native Swift Helper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the macOS helper window with a native SwiftUI/AppKit implementation while preserving the existing Windows/Linux `/helper` webview fallback and all current helper behavior.

**Architecture:** Rust remains the owner of settings, BibTeX parsing, search, clipboard, and cross-app paste. macOS links a native Xcode-managed Swift library from `macos/NativeHelper/`; SwiftUI renders the helper UI, AppKit owns only the `NSPanel`, and Rust/Swift communicate through explicit C ABI structs, handles, arrays, and free functions without JSON.

**Tech Stack:** Tauri v2, Rust 2024, C ABI FFI, Xcode macOS library target, SwiftUI, AppKit `NSPanel`, LaTeXSwiftUI, Bun/SolidJS fallback for non-macOS helper.

---

## File Structure

Create or modify these files:

- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/build.rs`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/commands.rs`
- Create: `src-tauri/src/native_helper/mod.rs`
- Create: `src-tauri/src/native_helper/ffi.rs`
- Create: `src-tauri/src/native_helper/state.rs`
- Create: `src-tauri/src/native_helper/bridge.rs`
- Create: `src-tauri/src/native_helper/tests.rs`
- Create: `macos/NativeHelper/NativeHelper.xcodeproj/project.pbxproj`
- Create: `macos/NativeHelper/NativeHelper/BibCiTeXNativeHelper.swift`
- Create: `macos/NativeHelper/NativeHelper/NativeHelperPanelController.swift`
- Create: `macos/NativeHelper/NativeHelper/HelperView.swift`
- Create: `macos/NativeHelper/NativeHelper/HelperViewModel.swift`
- Create: `macos/NativeHelper/NativeHelper/NativeHelperModels.swift`
- Create: `macos/NativeHelper/NativeHelper/NativeHelperFFI.swift`
- Create: `macos/NativeHelper/NativeHelper/MathChunkText.swift`
- Create: `macos/NativeHelper/NativeHelper/NativeHelperTheme.swift`
- Create: `macos/NativeHelper/NativeHelper/NativeHelper.xcconfig`
- Modify: `src/context/AppContext.tsx`
- Modify: `src/tauri.ts`
- Leave in place: `src/pages/HelperPage.tsx` and `/helper` route for Windows/Linux fallback.

## Commit Rule

For every task commit in this plan:

1. Inspect `git status`, `git diff`, and `git diff --staged` before committing.
2. Stage only the files listed in the task.
3. Create the temporary commit message file named in the task command.
4. Use a Conventional Commits subject.
5. Include required `Summary:`, `Rationale:`, and `Tests:` sections.
6. End with `Co-authored-by: Codex <noreply@openai.com>`.
7. Run `git commit -F <message-file>`.

Do not include unrelated user changes in any task commit.

## User Checkpoints

Before implementing each task group, confirm with the user when the task changes one of these boundaries:

- C ABI shape.
- Xcode project structure.
- LaTeXSwiftUI integration details.
- Theme synchronization behavior.
- macOS window behavior.

## Task 1: Add Rust FFI Types With Failing Tests

**Files:**
- Create: `src-tauri/src/native_helper/mod.rs`
- Create: `src-tauri/src/native_helper/ffi.rs`
- Create: `src-tauri/src/native_helper/tests.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Confirm C ABI checkpoint**

Ask the user to confirm this concrete ABI shape before writing code:

```text
FfiString { ptr: *mut c_char, len: usize }
FfiStringArray { ptr: *mut FfiString, len: usize }
FfiChunk { kind: FfiChunkKind, text: FfiString }
FfiChunkArray { ptr: *mut FfiChunk, len: usize }
FfiEntryType { kind: FfiEntryTypeKind, unknown: FfiString }
FfiOptionalI64 { has_value: bool, value: i64 }
FfiOptionalI32 { has_value: bool, value: i32 }
FfiOptionalU32Range { has_value: bool, start: u32, end: u32 }
FfiReference { all fields from Rust Reference, represented with explicit FFI option/array wrappers }
FfiReferenceList { ptr: *mut FfiReference, len: usize }
```

Expected: user confirms or revises.

- [ ] **Step 2: Create module declarations**

Create `src-tauri/src/native_helper/mod.rs`:

```rust
pub mod ffi;

#[cfg(test)]
mod tests;
```

Modify `src-tauri/src/lib.rs` near existing module declarations:

```rust
mod commands;
mod error;
pub mod native_helper;
mod core;
mod xpaste;
```

- [ ] **Step 3: Write failing FFI allocation tests**

Create `src-tauri/src/native_helper/tests.rs`:

```rust
use super::ffi::*;
use biblatex::{Chunk, EntryType};

unsafe fn ffi_string_to_string(value: &FfiString) -> String {
    if value.ptr.is_null() || value.len == 0 {
        return String::new();
    }
    let bytes = std::slice::from_raw_parts(value.ptr.cast::<u8>(), value.len);
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[test]
fn ffi_string_preserves_utf8_and_releases() {
    let value = FfiString::from_string("BibCiTeX 助手".to_string());

    assert_eq!(unsafe { ffi_string_to_string(&value) }, "BibCiTeX 助手");

    unsafe { bibcitex_helper_free_string(value) };
}

#[test]
fn chunk_array_preserves_kind_and_text() {
    let chunks = vec![
        Chunk::Normal("A ".to_string()),
        Chunk::Math("x^2".to_string()),
        Chunk::Verbatim(" code ".to_string()),
    ];

    let array = FfiChunkArray::from_chunks(Some(chunks));

    assert!(array.has_value);
    assert_eq!(array.len, 3);
    let values = unsafe { std::slice::from_raw_parts(array.ptr, array.len) };
    assert_eq!(values[0].kind, FfiChunkKind::Normal);
    assert_eq!(unsafe { ffi_string_to_string(&values[0].text) }, "A ");
    assert_eq!(values[1].kind, FfiChunkKind::Math);
    assert_eq!(unsafe { ffi_string_to_string(&values[1].text) }, "x^2");
    assert_eq!(values[2].kind, FfiChunkKind::Verbatim);
    assert_eq!(unsafe { ffi_string_to_string(&values[2].text) }, " code ");

    unsafe { bibcitex_helper_free_chunk_array(array) };
}

#[test]
fn entry_type_unknown_preserves_payload() {
    let entry_type = FfiEntryType::from_entry_type(&EntryType::Unknown("Dataset".to_string()));

    assert_eq!(entry_type.kind, FfiEntryTypeKind::Unknown);
    assert_eq!(unsafe { ffi_string_to_string(&entry_type.unknown) }, "Dataset");

    unsafe { bibcitex_helper_free_entry_type(entry_type) };
}
```

- [ ] **Step 4: Run tests to verify RED**

Run:

```bash
cd src-tauri && cargo test native_helper
```

Expected:

```text
FAIL with unresolved types/functions such as FfiString, FfiChunkArray, and bibcitex_helper_free_string
```

- [ ] **Step 5: Commit failing tests**

Create `/tmp/bibcitex-native-helper-task-1-commit.txt`:

```text
test(helper): define native helper ffi expectations

Summary:
- add native helper module files
- add failing tests for UTF-8 strings, chunk arrays, and entry type payloads

Rationale:
- document the low-level C ABI behavior before implementing the Rust bridge

Tests:
- cd src-tauri && cargo test native_helper (expected failure before implementation)

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add src-tauri/src/lib.rs src-tauri/src/native_helper/mod.rs src-tauri/src/native_helper/tests.rs
git commit -F /tmp/bibcitex-native-helper-task-1-commit.txt
```

## Task 2: Implement Rust FFI Conversions and Free Functions

**Files:**
- Modify: `src-tauri/src/native_helper/ffi.rs`
- Modify: `src-tauri/src/native_helper/tests.rs`

- [ ] **Step 1: Implement FFI scalar and collection types**

Create `src-tauri/src/native_helper/ffi.rs`:

```rust
use crate::core::bib::Reference;
use biblatex::{Chunk, EntryType};
use std::ffi::{c_char, CString};
use std::ptr;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiString {
    pub ptr: *mut c_char,
    pub len: usize,
}

impl FfiString {
    pub fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn from_string(value: String) -> Self {
        if value.is_empty() {
            return Self::empty();
        }
        let len = value.len();
        let c_string = CString::new(value).unwrap_or_default();
        Self {
            ptr: c_string.into_raw(),
            len,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiStringArray {
    pub has_value: bool,
    pub ptr: *mut FfiString,
    pub len: usize,
}

impl FfiStringArray {
    pub fn none() -> Self {
        Self {
            has_value: false,
            ptr: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn from_option(values: Option<Vec<String>>) -> Self {
        let Some(values) = values else {
            return Self::none();
        };
        let mut ffi_values = values
            .into_iter()
            .map(FfiString::from_string)
            .collect::<Vec<_>>();
        let len = ffi_values.len();
        let ptr = ffi_values.as_mut_ptr();
        std::mem::forget(ffi_values);
        Self {
            has_value: true,
            ptr,
            len,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfiChunkKind {
    Normal = 0,
    Verbatim = 1,
    Math = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiChunk {
    pub kind: FfiChunkKind,
    pub text: FfiString,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiChunkArray {
    pub has_value: bool,
    pub ptr: *mut FfiChunk,
    pub len: usize,
}

impl FfiChunkArray {
    pub fn none() -> Self {
        Self {
            has_value: false,
            ptr: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn from_chunks(chunks: Option<Vec<Chunk>>) -> Self {
        let Some(chunks) = chunks else {
            return Self::none();
        };
        let mut values = chunks
            .into_iter()
            .map(|chunk| match chunk {
                Chunk::Normal(text) => FfiChunk {
                    kind: FfiChunkKind::Normal,
                    text: FfiString::from_string(text),
                },
                Chunk::Verbatim(text) => FfiChunk {
                    kind: FfiChunkKind::Verbatim,
                    text: FfiString::from_string(text),
                },
                Chunk::Math(text) => FfiChunk {
                    kind: FfiChunkKind::Math,
                    text: FfiString::from_string(text),
                },
            })
            .collect::<Vec<_>>();
        let len = values.len();
        let ptr = values.as_mut_ptr();
        std::mem::forget(values);
        Self {
            has_value: true,
            ptr,
            len,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy)]
pub struct FfiEntryType {
    pub kind: FfiEntryTypeKind,
    pub unknown: FfiString,
}

impl FfiEntryType {
    pub fn from_entry_type(entry_type: &EntryType) -> Self {
        let kind = match entry_type {
            EntryType::Article => FfiEntryTypeKind::Article,
            EntryType::Book => FfiEntryTypeKind::Book,
            EntryType::Booklet => FfiEntryTypeKind::Booklet,
            EntryType::InBook => FfiEntryTypeKind::InBook,
            EntryType::InCollection => FfiEntryTypeKind::InCollection,
            EntryType::InProceedings => FfiEntryTypeKind::InProceedings,
            EntryType::Manual => FfiEntryTypeKind::Manual,
            EntryType::MastersThesis => FfiEntryTypeKind::MastersThesis,
            EntryType::PhdThesis => FfiEntryTypeKind::PhdThesis,
            EntryType::Misc => FfiEntryTypeKind::Misc,
            EntryType::Proceedings => FfiEntryTypeKind::Proceedings,
            EntryType::TechReport => FfiEntryTypeKind::TechReport,
            EntryType::Unpublished => FfiEntryTypeKind::Unpublished,
            EntryType::MvBook => FfiEntryTypeKind::MvBook,
            EntryType::BookInBook => FfiEntryTypeKind::BookInBook,
            EntryType::SuppBook => FfiEntryTypeKind::SuppBook,
            EntryType::Periodical => FfiEntryTypeKind::Periodical,
            EntryType::SuppPeriodical => FfiEntryTypeKind::SuppPeriodical,
            EntryType::Collection => FfiEntryTypeKind::Collection,
            EntryType::MvCollection => FfiEntryTypeKind::MvCollection,
            EntryType::SuppCollection => FfiEntryTypeKind::SuppCollection,
            EntryType::Reference => FfiEntryTypeKind::Reference,
            EntryType::MvReference => FfiEntryTypeKind::MvReference,
            EntryType::InReference => FfiEntryTypeKind::InReference,
            EntryType::MvProceedings => FfiEntryTypeKind::MvProceedings,
            EntryType::Report => FfiEntryTypeKind::Report,
            EntryType::Patent => FfiEntryTypeKind::Patent,
            EntryType::Thesis => FfiEntryTypeKind::Thesis,
            EntryType::Online => FfiEntryTypeKind::Online,
            EntryType::Software => FfiEntryTypeKind::Software,
            EntryType::Dataset => FfiEntryTypeKind::Dataset,
            EntryType::Set => FfiEntryTypeKind::Set,
            EntryType::XData => FfiEntryTypeKind::XData,
            EntryType::Unknown(_) => FfiEntryTypeKind::Unknown,
        };
        let unknown = match entry_type {
            EntryType::Unknown(value) => FfiString::from_string(value.clone()),
            _ => FfiString::empty(),
        };
        Self { kind, unknown }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiOptionalI32 {
    pub has_value: bool,
    pub value: i32,
}

impl FfiOptionalI32 {
    pub fn from_option(value: Option<i32>) -> Self {
        match value {
            Some(value) => Self {
                has_value: true,
                value,
            },
            None => Self {
                has_value: false,
                value: 0,
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiOptionalI64 {
    pub has_value: bool,
    pub value: i64,
}

impl FfiOptionalI64 {
    pub fn from_option(value: Option<i64>) -> Self {
        match value {
            Some(value) => Self {
                has_value: true,
                value,
            },
            None => Self {
                has_value: false,
                value: 0,
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiOptionalU32Range {
    pub has_value: bool,
    pub start: u32,
    pub end: u32,
}

impl FfiOptionalU32Range {
    pub fn from_option(value: Option<std::ops::Range<u32>>) -> Self {
        match value {
            Some(value) => Self {
                has_value: true,
                start: value.start,
                end: value.end,
            },
            None => Self {
                has_value: false,
                start: 0,
                end: 0,
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiOptionalString {
    pub has_value: bool,
    pub value: FfiString,
}

impl FfiOptionalString {
    pub fn from_option(value: Option<String>) -> Self {
        match value {
            Some(value) => Self {
                has_value: true,
                value: FfiString::from_string(value),
            },
            None => Self {
                has_value: false,
                value: FfiString::empty(),
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiEditor {
    pub name: FfiString,
    pub editor_type: FfiString,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiEditorArray {
    pub has_value: bool,
    pub ptr: *mut FfiEditor,
    pub len: usize,
}

impl FfiEditorArray {
    pub fn from_option(values: Option<Vec<(String, String)>>) -> Self {
        let Some(values) = values else {
            return Self {
                has_value: false,
                ptr: ptr::null_mut(),
                len: 0,
            };
        };
        let mut ffi_values = values
            .into_iter()
            .map(|(name, editor_type)| FfiEditor {
                name: FfiString::from_string(name),
                editor_type: FfiString::from_string(editor_type),
            })
            .collect::<Vec<_>>();
        let len = ffi_values.len();
        let ptr = ffi_values.as_mut_ptr();
        std::mem::forget(ffi_values);
        Self {
            has_value: true,
            ptr,
            len,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiReference {
    pub cite_key: FfiString,
    pub source: FfiString,
    pub type_: FfiEntryType,
    pub author: FfiStringArray,
    pub title: FfiChunkArray,
    pub journal: FfiOptionalString,
    pub year: FfiOptionalI32,
    pub full_journal: FfiOptionalString,
    pub volume: FfiOptionalI64,
    pub number: FfiOptionalString,
    pub pages: FfiOptionalU32Range,
    pub note: FfiChunkArray,
    pub doi: FfiOptionalString,
    pub mrclass: FfiOptionalString,
    pub publisher: FfiStringArray,
    pub isbn: FfiOptionalString,
    pub series: FfiOptionalString,
    pub url: FfiOptionalString,
    pub file: FfiOptionalString,
    pub abstract_: FfiChunkArray,
    pub edition: FfiOptionalI64,
    pub issue: FfiChunkArray,
    pub book_pages: FfiOptionalString,
    pub school: FfiOptionalString,
    pub address: FfiOptionalString,
    pub book_title: FfiChunkArray,
    pub editor: FfiEditorArray,
    pub month: FfiOptionalString,
    pub organization: FfiStringArray,
    pub institution: FfiOptionalString,
    pub eprint: FfiOptionalString,
    pub archive_prefix: FfiOptionalString,
    pub arxiv_primary_class: FfiOptionalString,
    pub how_published: FfiOptionalString,
}

impl FfiReference {
    pub fn from_reference(reference: Reference) -> Self {
        Self {
            cite_key: FfiString::from_string(reference.cite_key),
            source: FfiString::from_string(reference.source),
            type_: FfiEntryType::from_entry_type(&reference.type_),
            author: FfiStringArray::from_option(reference.author),
            title: FfiChunkArray::from_chunks(reference.title),
            journal: FfiOptionalString::from_option(reference.journal),
            year: FfiOptionalI32::from_option(reference.year),
            full_journal: FfiOptionalString::from_option(reference.full_journal),
            volume: FfiOptionalI64::from_option(reference.volume),
            number: FfiOptionalString::from_option(reference.number),
            pages: FfiOptionalU32Range::from_option(reference.pages),
            note: FfiChunkArray::from_chunks(reference.note),
            doi: FfiOptionalString::from_option(reference.doi),
            mrclass: FfiOptionalString::from_option(reference.mrclass),
            publisher: FfiStringArray::from_option(reference.publisher),
            isbn: FfiOptionalString::from_option(reference.isbn),
            series: FfiOptionalString::from_option(reference.series),
            url: FfiOptionalString::from_option(reference.url),
            file: FfiOptionalString::from_option(reference.file),
            abstract_: FfiChunkArray::from_chunks(reference.abstract_),
            edition: FfiOptionalI64::from_option(reference.edition),
            issue: FfiChunkArray::from_chunks(reference.issue),
            book_pages: FfiOptionalString::from_option(reference.book_pages),
            school: FfiOptionalString::from_option(reference.school),
            address: FfiOptionalString::from_option(reference.address),
            book_title: FfiChunkArray::from_chunks(reference.book_title),
            editor: FfiEditorArray::from_option(reference.editor),
            month: FfiOptionalString::from_option(reference.month),
            organization: FfiStringArray::from_option(reference.organization),
            institution: FfiOptionalString::from_option(reference.institution),
            eprint: FfiOptionalString::from_option(reference.eprint),
            archive_prefix: FfiOptionalString::from_option(reference.archive_prefix),
            arxiv_primary_class: FfiOptionalString::from_option(reference.arxiv_primary_class),
            how_published: FfiOptionalString::from_option(reference.how_published),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiReferenceList {
    pub ptr: *mut FfiReference,
    pub len: usize,
}

impl FfiReferenceList {
    pub fn from_references(references: Vec<Reference>) -> Self {
        let mut values = references
            .into_iter()
            .map(FfiReference::from_reference)
            .collect::<Vec<_>>();
        let len = values.len();
        let ptr = values.as_mut_ptr();
        std::mem::forget(values);
        Self { ptr, len }
    }
}

pub unsafe fn free_string(value: FfiString) {
    if !value.ptr.is_null() {
        let _ = CString::from_raw(value.ptr);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_string(value: FfiString) {
    unsafe { free_string(value) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_string_array(array: FfiStringArray) {
    if array.ptr.is_null() {
        return;
    }
    let values = unsafe { Vec::from_raw_parts(array.ptr, array.len, array.len) };
    for value in values {
        unsafe { free_string(value) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_chunk_array(array: FfiChunkArray) {
    if array.ptr.is_null() {
        return;
    }
    let values = unsafe { Vec::from_raw_parts(array.ptr, array.len, array.len) };
    for value in values {
        unsafe { free_string(value.text) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_entry_type(entry_type: FfiEntryType) {
    unsafe { free_string(entry_type.unknown) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_optional_string(value: FfiOptionalString) {
    if value.has_value {
        unsafe { free_string(value.value) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_editor_array(array: FfiEditorArray) {
    if array.ptr.is_null() {
        return;
    }
    let values = unsafe { Vec::from_raw_parts(array.ptr, array.len, array.len) };
    for value in values {
        unsafe { free_string(value.name) };
        unsafe { free_string(value.editor_type) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_reference(reference: FfiReference) {
    unsafe { free_string(reference.cite_key) };
    unsafe { free_string(reference.source) };
    unsafe { bibcitex_helper_free_entry_type(reference.type_) };
    unsafe { bibcitex_helper_free_string_array(reference.author) };
    unsafe { bibcitex_helper_free_chunk_array(reference.title) };
    unsafe { bibcitex_helper_free_optional_string(reference.journal) };
    unsafe { bibcitex_helper_free_optional_string(reference.full_journal) };
    unsafe { bibcitex_helper_free_optional_string(reference.number) };
    unsafe { bibcitex_helper_free_chunk_array(reference.note) };
    unsafe { bibcitex_helper_free_optional_string(reference.doi) };
    unsafe { bibcitex_helper_free_optional_string(reference.mrclass) };
    unsafe { bibcitex_helper_free_string_array(reference.publisher) };
    unsafe { bibcitex_helper_free_optional_string(reference.isbn) };
    unsafe { bibcitex_helper_free_optional_string(reference.series) };
    unsafe { bibcitex_helper_free_optional_string(reference.url) };
    unsafe { bibcitex_helper_free_optional_string(reference.file) };
    unsafe { bibcitex_helper_free_chunk_array(reference.abstract_) };
    unsafe { bibcitex_helper_free_chunk_array(reference.issue) };
    unsafe { bibcitex_helper_free_optional_string(reference.book_pages) };
    unsafe { bibcitex_helper_free_optional_string(reference.school) };
    unsafe { bibcitex_helper_free_optional_string(reference.address) };
    unsafe { bibcitex_helper_free_chunk_array(reference.book_title) };
    unsafe { bibcitex_helper_free_editor_array(reference.editor) };
    unsafe { bibcitex_helper_free_optional_string(reference.month) };
    unsafe { bibcitex_helper_free_string_array(reference.organization) };
    unsafe { bibcitex_helper_free_optional_string(reference.institution) };
    unsafe { bibcitex_helper_free_optional_string(reference.eprint) };
    unsafe { bibcitex_helper_free_optional_string(reference.archive_prefix) };
    unsafe { bibcitex_helper_free_optional_string(reference.arxiv_primary_class) };
    unsafe { bibcitex_helper_free_optional_string(reference.how_published) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_reference_list(list: FfiReferenceList) {
    if list.ptr.is_null() {
        return;
    }
    let values = unsafe { Vec::from_raw_parts(list.ptr, list.len, list.len) };
    for value in values {
        unsafe { bibcitex_helper_free_reference(value) };
    }
}
```

- [ ] **Step 2: Run Task 1 tests to verify GREEN**

Run:

```bash
cd src-tauri && cargo test native_helper
```

Expected:

```text
test result: ok
```

- [ ] **Step 3: Add full Reference conversion test**

Append to `src-tauri/src/native_helper/tests.rs`:

```rust
use crate::core::bib::Reference;

fn sample_reference() -> Reference {
    Reference {
        cite_key: "smith2026".to_string(),
        source: "@article{smith2026}".to_string(),
        type_: EntryType::Article,
        author: Some(vec!["Ada Lovelace".to_string(), "Grace Hopper".to_string()]),
        title: Some(vec![Chunk::Normal("Native ".to_string()), Chunk::Math("x^2".to_string())]),
        journal: Some("Journal of FFI".to_string()),
        year: Some(2026),
        full_journal: Some("Journal of Foreign Function Interfaces".to_string()),
        volume: Some(12),
        number: Some("3".to_string()),
        pages: Some(10..24),
        note: None,
        doi: Some("10.1000/example".to_string()),
        mrclass: None,
        publisher: Some(vec!["Example Press".to_string()]),
        isbn: None,
        series: None,
        url: Some("https://example.com".to_string()),
        file: None,
        abstract_: None,
        edition: None,
        issue: None,
        book_pages: None,
        school: None,
        address: None,
        book_title: None,
        editor: Some(vec![("Editor One".to_string(), "editor".to_string())]),
        month: Some("June".to_string()),
        organization: None,
        institution: None,
        eprint: None,
        archive_prefix: None,
        arxiv_primary_class: None,
        how_published: None,
    }
}

#[test]
fn reference_conversion_preserves_complete_helper_fields() {
    let list = FfiReferenceList::from_references(vec![sample_reference()]);

    assert_eq!(list.len, 1);
    let first = unsafe { &*list.ptr };
    assert_eq!(unsafe { ffi_string_to_string(&first.cite_key) }, "smith2026");
    assert_eq!(first.type_.kind, FfiEntryTypeKind::Article);
    assert!(first.title.has_value);
    assert_eq!(first.year.value, 2026);
    assert!(first.pages.has_value);
    assert_eq!(first.pages.start, 10);
    assert_eq!(first.pages.end, 24);
    assert!(first.editor.has_value);

    unsafe { bibcitex_helper_free_reference_list(list) };
}
```

- [ ] **Step 4: Run native helper tests**

Run:

```bash
cd src-tauri && cargo test native_helper
```

Expected:

```text
test result: ok
```

- [ ] **Step 5: Commit**

Create `/tmp/bibcitex-native-helper-task-2-commit.txt`:

```text
feat(helper): add native helper ffi reference types

Summary:
- implement C ABI string, option, chunk, enum, and reference structures
- add Rust-owned free functions for Swift-consumed FFI values
- verify full Reference conversion and release behavior

Rationale:
- provide a JSON-free data boundary for the native Swift helper
- keep ownership explicit so Swift can consume Rust data safely

Tests:
- cd src-tauri && cargo test native_helper

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add src-tauri/src/native_helper/ffi.rs src-tauri/src/native_helper/tests.rs
git commit -F /tmp/bibcitex-native-helper-task-2-commit.txt
```

## Task 3: Add Rust Helper State Handles and Search FFI

**Files:**
- Create: `src-tauri/src/native_helper/state.rs`
- Modify: `src-tauri/src/native_helper/mod.rs`
- Modify: `src-tauri/src/native_helper/tests.rs`

- [ ] **Step 1: Write failing state tests**

Append to `src-tauri/src/native_helper/tests.rs`:

```rust
use super::state::*;

#[test]
fn library_handle_searches_with_existing_rust_logic() {
    let handle = HelperLibrary::from_references_for_test(vec![sample_reference()]);

    let results = handle.search("Native");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].cite_key, "smith2026");
}

#[test]
fn helper_state_remembers_current_bibliography_in_memory() {
    let state = HelperState::default();

    state.set_current_bibliography("Main".to_string(), "/tmp/main.bib".to_string());

    assert_eq!(
        state.current_bibliography(),
        Some(("Main".to_string(), "/tmp/main.bib".to_string()))
    );
}
```

- [ ] **Step 2: Run tests to verify RED**

Run:

```bash
cd src-tauri && cargo test native_helper
```

Expected:

```text
FAIL with unresolved HelperLibrary and HelperState
```

- [ ] **Step 3: Implement state and handle layer**

Create `src-tauri/src/native_helper/state.rs`:

```rust
use crate::core::bib::Reference;
use crate::core::search::search_references;
use std::sync::Mutex;

pub struct HelperLibrary {
    references: Vec<Reference>,
}

impl HelperLibrary {
    pub fn new(references: Vec<Reference>) -> Self {
        Self { references }
    }

    #[cfg(test)]
    pub fn from_references_for_test(references: Vec<Reference>) -> Self {
        Self::new(references)
    }

    pub fn search(&self, query: &str) -> Vec<Reference> {
        search_references(&self.references, query)
    }
}

#[derive(Default)]
pub struct HelperState {
    current_bibliography: Mutex<Option<(String, String)>>,
}

impl HelperState {
    pub fn set_current_bibliography(&self, name: String, path: String) {
        *self.current_bibliography.lock().unwrap() = Some((name, path));
    }

    pub fn current_bibliography(&self) -> Option<(String, String)> {
        self.current_bibliography.lock().unwrap().clone()
    }
}
```

Modify `src-tauri/src/native_helper/mod.rs`:

```rust
pub mod ffi;
pub mod state;

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Run tests to verify GREEN**

Run:

```bash
cd src-tauri && cargo test native_helper
```

Expected:

```text
test result: ok
```

- [ ] **Step 5: Add exported handle FFI functions**

Append to `src-tauri/src/native_helper/state.rs`:

```rust
use super::ffi::FfiReferenceList;
use crate::core::bib::parse;

#[repr(C)]
pub struct HelperLibraryHandle {
    library: HelperLibrary,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_load_library(
    path_ptr: *const std::ffi::c_char,
    path_len: usize,
) -> *mut HelperLibraryHandle {
    if path_ptr.is_null() {
        return std::ptr::null_mut();
    }
    let bytes = unsafe { std::slice::from_raw_parts(path_ptr.cast::<u8>(), path_len) };
    let Ok(path) = std::str::from_utf8(bytes) else {
        return std::ptr::null_mut();
    };
    let Ok(bibliography) = parse(path) else {
        return std::ptr::null_mut();
    };
    let references = bibliography.iter().map(Reference::from).collect::<Vec<_>>();
    Box::into_raw(Box::new(HelperLibraryHandle {
        library: HelperLibrary::new(references),
    }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_library(handle: *mut HelperLibraryHandle) {
    if !handle.is_null() {
        let _ = unsafe { Box::from_raw(handle) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_search_library(
    handle: *const HelperLibraryHandle,
    query_ptr: *const std::ffi::c_char,
    query_len: usize,
) -> FfiReferenceList {
    if handle.is_null() || query_ptr.is_null() {
        return FfiReferenceList {
            ptr: std::ptr::null_mut(),
            len: 0,
        };
    }
    let bytes = unsafe { std::slice::from_raw_parts(query_ptr.cast::<u8>(), query_len) };
    let Ok(query) = std::str::from_utf8(bytes) else {
        return FfiReferenceList {
            ptr: std::ptr::null_mut(),
            len: 0,
        };
    };
    let handle = unsafe { &*handle };
    FfiReferenceList::from_references(handle.library.search(query))
}
```

- [ ] **Step 6: Run tests and clippy for state layer**

Run:

```bash
cd src-tauri && cargo test native_helper
cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings
```

Expected:

```text
cargo test exits 0
cargo clippy exits 0
```

- [ ] **Step 7: Commit**

Create `/tmp/bibcitex-native-helper-task-3-commit.txt`:

```text
feat(helper): add rust-owned native helper library handles

Summary:
- add Rust helper state for current bibliography memory
- add library handle loading, searching, and release functions for Swift
- reuse existing Rust search logic behind the FFI handle

Rationale:
- keep large bibliography data owned by Rust while giving Swift short-lived search results
- preserve the existing search semantics for the native helper

Tests:
- cd src-tauri && cargo test native_helper
- cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add src-tauri/src/native_helper/mod.rs src-tauri/src/native_helper/state.rs src-tauri/src/native_helper/tests.rs
git commit -F /tmp/bibcitex-native-helper-task-3-commit.txt
```

## Task 4: Scaffold Xcode Native Helper Library

**Files:**
- Create: `macos/NativeHelper/NativeHelper.xcodeproj/project.pbxproj`
- Create: `macos/NativeHelper/NativeHelper/NativeHelper.xcconfig`
- Create: `macos/NativeHelper/NativeHelper/BibCiTeXNativeHelper.swift`
- Create: `macos/NativeHelper/NativeHelper/NativeHelperFFI.swift`
- Create: `macos/NativeHelper/NativeHelper/NativeHelperModels.swift`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Confirm Xcode checkpoint**

Ask the user to confirm:

```text
Create an Xcode macOS static-library project under macos/NativeHelper.
Set macOS deployment target to 12.0 because LaTeXSwiftUI requires macOS 12+.
Cargo remains the final application build entrypoint.
```

Expected: user confirms or revises.

- [ ] **Step 2: Update macOS minimum system version**

Modify `src-tauri/tauri.conf.json`:

```json
"minimumSystemVersion": "12.0"
```

- [ ] **Step 3: Create native helper xcconfig**

Create `macos/NativeHelper/NativeHelper/NativeHelper.xcconfig`:

```text
MACOSX_DEPLOYMENT_TARGET = 12.0
SWIFT_VERSION = 6.0
PRODUCT_BUNDLE_IDENTIFIER = com.bibcitex.NativeHelper
DEFINES_MODULE = YES
SKIP_INSTALL = YES
```

- [ ] **Step 4: Create Swift FFI type mirrors**

Create `macos/NativeHelper/NativeHelper/NativeHelperFFI.swift`:

```swift
import Foundation

public struct FfiString {
    public var ptr: UnsafeMutablePointer<CChar>?
    public var len: Int
}

public struct FfiStringArray {
    public var has_value: Bool
    public var ptr: UnsafeMutablePointer<FfiString>?
    public var len: Int
}

public enum FfiChunkKind: Int32 {
    case normal = 0
    case verbatim = 1
    case math = 2
}

public struct FfiChunk {
    public var kind: FfiChunkKind
    public var text: FfiString
}

public struct FfiChunkArray {
    public var has_value: Bool
    public var ptr: UnsafeMutablePointer<FfiChunk>?
    public var len: Int
}

public enum FfiEntryTypeKind: Int32 {
    case article = 0
    case book = 1
    case booklet = 2
    case inBook = 3
    case inCollection = 4
    case inProceedings = 5
    case manual = 6
    case mastersThesis = 7
    case phdThesis = 8
    case misc = 9
    case proceedings = 10
    case techReport = 11
    case unpublished = 12
    case mvBook = 13
    case bookInBook = 14
    case suppBook = 15
    case periodical = 16
    case suppPeriodical = 17
    case collection = 18
    case mvCollection = 19
    case suppCollection = 20
    case reference = 21
    case mvReference = 22
    case inReference = 23
    case mvProceedings = 24
    case report = 25
    case patent = 26
    case thesis = 27
    case online = 28
    case software = 29
    case dataset = 30
    case set = 31
    case xData = 32
    case unknown = 255
}
```

- [ ] **Step 5: Create Swift domain models**

Create `macos/NativeHelper/NativeHelper/NativeHelperModels.swift`:

```swift
import Foundation

public enum HelperChunkKind {
    case normal
    case verbatim
    case math
}

public struct HelperChunk: Identifiable {
    public let id = UUID()
    public let kind: HelperChunkKind
    public let text: String
}

public struct HelperReference: Identifiable {
    public let id = UUID()
    public let citeKey: String
    public let typeLabel: String
    public let title: [HelperChunk]
    public let authors: [String]
    public let year: Int?
    public let venue: String?
}

public struct HelperBibliography: Identifiable {
    public var id: String { path }
    public let name: String
    public let path: String
    public let updatedAt: String
    public let description: String?
}
```

- [ ] **Step 6: Create exported Swift entrypoints**

Create `macos/NativeHelper/NativeHelper/BibCiTeXNativeHelper.swift`:

```swift
import AppKit
import SwiftUI

@_cdecl("bibcitex_native_helper_show")
public func bibcitexNativeHelperShow() {
    NativeHelperPanelController.shared.show()
}

@_cdecl("bibcitex_native_helper_hide")
public func bibcitexNativeHelperHide() {
    NativeHelperPanelController.shared.hide()
}

@_cdecl("bibcitex_native_helper_set_theme")
public func bibcitexNativeHelperSetTheme(_ theme: Int32) {
    NativeHelperPanelController.shared.setTheme(theme)
}
```

- [ ] **Step 7: Create minimal Xcode project**

Create `macos/NativeHelper/NativeHelper.xcodeproj/project.pbxproj` with Xcode. Use a macOS static library target named `NativeHelper` with these fixed settings:

```text
Project path: macos/NativeHelper/NativeHelper.xcodeproj
Target name: NativeHelper
Product: libNativeHelper.a
Language: Swift
Deployment target: macOS 12.0
Source group: macos/NativeHelper/NativeHelper
Swift files in target: BibCiTeXNativeHelper.swift, NativeHelperFFI.swift, NativeHelperModels.swift
Linked frameworks: AppKit.framework, SwiftUI.framework
Swift package dependency: https://github.com/colinc86/LaTeXSwiftUI from 2.0.0
```

After creating the project, commit the generated `project.pbxproj` exactly as Xcode writes it, then verify that `xcodebuild -list -project macos/NativeHelper/NativeHelper.xcodeproj` lists the `NativeHelper` scheme.

- [ ] **Step 8: Verify Xcode project discovery**

Run:

```bash
xcodebuild -list -project macos/NativeHelper/NativeHelper.xcodeproj
```

Expected:

```text
Schemes:
    NativeHelper
```

- [ ] **Step 9: Commit**

Create `/tmp/bibcitex-native-helper-task-4-commit.txt`:

```text
build(helper): scaffold native Swift helper project

Summary:
- add the macOS NativeHelper Xcode project and initial Swift FFI models
- set the macOS deployment target to 12.0 for LaTeXSwiftUI support
- add native helper exported show, hide, and theme entrypoints

Rationale:
- establish the Xcode-managed SwiftUI helper library before linking it from Cargo
- satisfy the native math rendering dependency requirement

Tests:
- xcodebuild -list -project macos/NativeHelper/NativeHelper.xcodeproj

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add macos/NativeHelper src-tauri/tauri.conf.json
git commit -F /tmp/bibcitex-native-helper-task-4-commit.txt
```

## Task 5: Build and Link Native Helper From Cargo

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/build.rs`
- Create: `src-tauri/src/native_helper/bridge.rs`
- Modify: `src-tauri/src/native_helper/mod.rs`

- [ ] **Step 1: Write failing bridge compile expectation**

Create `src-tauri/src/native_helper/bridge.rs`:

```rust
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn bibcitex_native_helper_show();
    fn bibcitex_native_helper_hide();
    fn bibcitex_native_helper_set_theme(theme: i32);
}

#[cfg(target_os = "macos")]
pub fn show() {
    unsafe { bibcitex_native_helper_show() };
}

#[cfg(target_os = "macos")]
pub fn hide() {
    unsafe { bibcitex_native_helper_hide() };
}

#[cfg(target_os = "macos")]
pub fn set_theme(theme: i32) {
    unsafe { bibcitex_native_helper_set_theme(theme) };
}
```

Modify `src-tauri/src/native_helper/mod.rs`:

```rust
#[cfg(target_os = "macos")]
pub mod bridge;
pub mod ffi;
pub mod state;

#[cfg(test)]
mod tests;
```

- [ ] **Step 2: Run build to verify link failure before build.rs integration**

Run:

```bash
cd src-tauri && cargo check
```

Expected:

```text
FAIL at link/build stage or unresolved native helper symbols before build.rs links NativeHelper
```

- [ ] **Step 3: Keep build dependencies unchanged**

Keep `src-tauri/Cargo.toml` build dependencies unchanged. The build script uses only `std::process::Command` to invoke Xcode:

```toml
[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 4: Link Xcode build output in build.rs**

Modify `src-tauri/build.rs`:

```rust
use std::{env, path::PathBuf, process::Command};

fn main() {
    #[cfg(target_os = "macos")]
    build_native_helper();

    tauri_build::build();
}

#[cfg(target_os = "macos")]
fn build_native_helper() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.parent().unwrap();
    let project = repo_root.join("macos/NativeHelper/NativeHelper.xcodeproj");
    let derived_data = PathBuf::from(env::var("OUT_DIR").unwrap()).join("NativeHelperDerivedData");
    let configuration = if env::var("DEBUG").unwrap_or_default() == "true" {
        "Debug"
    } else {
        "Release"
    };

    let status = Command::new("xcodebuild")
        .arg("build")
        .arg("-project")
        .arg(&project)
        .arg("-scheme")
        .arg("NativeHelper")
        .arg("-configuration")
        .arg(configuration)
        .arg("-derivedDataPath")
        .arg(&derived_data)
        .arg("MACOSX_DEPLOYMENT_TARGET=12.0")
        .status()
        .expect("failed to run xcodebuild for NativeHelper");

    assert!(status.success(), "failed to build NativeHelper with xcodebuild");

    let products = derived_data
        .join("Build")
        .join("Products")
        .join(configuration);

    println!("cargo:rerun-if-changed={}", project.display());
    println!("cargo:rerun-if-changed={}", repo_root.join("macos/NativeHelper/NativeHelper").display());
    println!("cargo:rustc-link-search=native={}", products.display());
    println!("cargo:rustc-link-lib=static=NativeHelper");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=SwiftUI");
}
```

The Xcode target must produce `libNativeHelper.a`. Do not switch to a framework without stopping and asking the user.

- [ ] **Step 5: Verify cargo check**

Run:

```bash
cd src-tauri && cargo check
```

Expected:

```text
cargo check exits 0
```

- [ ] **Step 6: Commit**

Create `/tmp/bibcitex-native-helper-task-5-commit.txt`:

```text
build(helper): link native Swift helper from cargo

Summary:
- invoke the NativeHelper Xcode build from the Tauri build script on macOS
- link the native helper library into the Rust application
- add Rust bridge functions for Swift show, hide, and theme entrypoints

Rationale:
- keep Cargo and Tauri as the final build entrypoint while using Xcode for Swift sources
- expose a narrow native helper bridge to the existing Rust command layer

Tests:
- cd src-tauri && cargo check

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/build.rs src-tauri/src/native_helper/bridge.rs src-tauri/src/native_helper/mod.rs
git commit -F /tmp/bibcitex-native-helper-task-5-commit.txt
```

## Task 6: Route macOS Helper Commands to Swift and Preserve Fallback

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`

- [ ] **Step 1: Confirm platform routing checkpoint**

Ask the user to confirm:

```text
macOS open/toggle/hide helper commands call native_helper::bridge.
Windows/Linux keep existing WebviewWindowBuilder /helper path.
Remove all tauri-nspanel uses and dependency.
```

Expected: user confirms or revises.

- [ ] **Step 2: Remove tauri-nspanel dependency**

Modify `src-tauri/Cargo.toml`:

```toml
[target."cfg(target_os = \"macos\")".dependencies]
objc2-app-kit = "0.3.2"
```

Do not edit `Cargo.lock` manually. It will be refreshed by the first successful Cargo command after the dependency and Rust references are removed.

After Step 4, run:

```bash
cd src-tauri && cargo check
```

Expected:

```text
Cargo.lock no longer contains tauri-nspanel
cargo check exits 0
```

- [ ] **Step 3: Remove nspanel initialization**

In `src-tauri/src/lib.rs`, remove:

```rust
// Initialize nspanel plugin on macOS
#[cfg(target_os = "macos")]
let builder = builder.plugin(tauri_nspanel::init());
```

Keep `builder.setup(...)` chained directly from the existing builder.

- [ ] **Step 4: Replace macOS helper command implementation**

In `src-tauri/src/commands.rs`, remove:

```rust
#[cfg(target_os = "macos")]
use tauri_nspanel::tauri_panel;

#[cfg(target_os = "macos")]
tauri_panel! { ... }

#[cfg(target_os = "macos")]
static HELPER_PANEL: Mutex<Option<std::sync::Arc<dyn tauri_nspanel::Panel<tauri::Wry>>>> =
    Mutex::new(None);
```

Replace macOS helper functions with:

```rust
#[cfg(target_os = "macos")]
pub fn toggle_helper_panel(_app: &AppHandle) {
    crate::native_helper::bridge::show();
}

#[cfg(target_os = "macos")]
pub fn hide_helper_panel() {
    crate::native_helper::bridge::hide();
}

#[cfg(target_os = "macos")]
pub fn create_helper_window(_app: AppHandle) -> Result<()> {
    crate::native_helper::bridge::show();
    Ok(())
}
```

Keep the existing non-macOS `create_helper_window` implementation.

Update `helper_webview_window` to only return a helper window on non-macOS:

```rust
fn helper_webview_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        None
    }

    #[cfg(not(target_os = "macos"))]
    {
        app.get_webview_window("helper")
    }
}
```

Simplify `resize_helper_window` on macOS to no-op returning `Ok(())`, because native Swift owns dynamic sizing.

- [ ] **Step 5: Verify no tauri-nspanel references remain**

Run:

```bash
rg -n "tauri_nspanel|tauri-nspanel|nspanel|HELPER_PANEL|PanelBuilder" src-tauri/Cargo.toml src-tauri/src src-tauri/Cargo.lock
```

Expected:

```text
no matches
```

- [ ] **Step 6: Verify Rust checks**

Run:

```bash
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cd src-tauri && cargo test
```

Expected:

```text
all commands exit 0
```

- [ ] **Step 7: Commit**

Create `/tmp/bibcitex-native-helper-task-6-commit.txt`:

```text
feat(helper): route macOS helper to native Swift

Summary:
- remove tauri-nspanel dependency and initialization
- route macOS helper open and hide commands to the native Swift bridge
- keep the existing Tauri webview helper path for Windows and Linux

Rationale:
- make macOS helper creation explicitly native while preserving non-macOS fallback behavior
- remove the unused panel plugin after replacing its ownership surface

Tests:
- rg -n "tauri_nspanel|tauri-nspanel|nspanel|HELPER_PANEL|PanelBuilder" src-tauri/Cargo.toml src-tauri/src src-tauri/Cargo.lock
- cd src-tauri && cargo fmt
- cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings
- cd src-tauri && cargo test

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src-tauri/src/commands.rs
git commit -F /tmp/bibcitex-native-helper-task-6-commit.txt
```

## Task 7: Implement Swift Panel Shell and View Model

**Files:**
- Modify: `macos/NativeHelper/NativeHelper/NativeHelperPanelController.swift`
- Modify: `macos/NativeHelper/NativeHelper/HelperViewModel.swift`
- Modify: `macos/NativeHelper/NativeHelper/NativeHelperModels.swift`
- Modify: `macos/NativeHelper/NativeHelper/NativeHelperFFI.swift`

- [ ] **Step 1: Confirm panel behavior checkpoint**

Ask user to confirm:

```text
Panel: floating, hides on deactivate, fixed width, dynamic height, centered horizontally, upper-third vertical placement, transparent background, can join all spaces.
```

Expected: user confirms or revises.

- [ ] **Step 2: Add Swift imports and FFI function declarations**

Extend `NativeHelperFFI.swift` with C function declarations matching Rust exports:

```swift
@_silgen_name("bibcitex_helper_load_library")
func bibcitex_helper_load_library(_ path: UnsafePointer<CChar>?, _ len: Int) -> UnsafeMutableRawPointer?

@_silgen_name("bibcitex_helper_free_library")
func bibcitex_helper_free_library(_ handle: UnsafeMutableRawPointer?)

@_silgen_name("bibcitex_helper_search_library")
func bibcitex_helper_search_library(_ handle: UnsafeRawPointer?, _ query: UnsafePointer<CChar>?, _ len: Int) -> FfiReferenceList

@_silgen_name("bibcitex_helper_free_reference_list")
func bibcitex_helper_free_reference_list(_ list: FfiReferenceList)
```

Also add Swift mirrors for `FfiReferenceList` and every field in `FfiReference`. `HelperReference` may display a subset, but the Swift FFI mirror must cover the full Rust ABI.

- [ ] **Step 3: Implement panel controller**

Create `macos/NativeHelper/NativeHelper/NativeHelperPanelController.swift`:

```swift
import AppKit
import SwiftUI

@MainActor
public final class NativeHelperPanelController {
    public static let shared = NativeHelperPanelController()

    private var panel: NSPanel?
    private let viewModel = HelperViewModel()

    private init() {}

    public func show() {
        let panel = existingOrCreatePanel()
        viewModel.prepareForOpen()
        panel.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    public func hide() {
        panel?.orderOut(nil)
    }

    public func setTheme(_ theme: Int32) {
        viewModel.applyTheme(NativeHelperTheme(rawValue: theme) ?? .latte)
    }

    public func updateHeight(_ height: CGFloat) {
        guard let panel else { return }
        var frame = panel.frame
        let oldMaxY = frame.maxY
        frame.size.height = height
        frame.origin.y = oldMaxY - height
        panel.setFrame(frame, display: true, animate: true)
    }

    private func existingOrCreatePanel() -> NSPanel {
        if let panel { return panel }

        let content = HelperView(viewModel: viewModel)
        let hostingView = NSHostingView(rootView: content)
        let size = NSSize(width: 760, height: 70)
        let frame = centeredFrame(size: size)
        let panel = NSPanel(
            contentRect: frame,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        panel.contentView = hostingView
        panel.isFloatingPanel = true
        panel.level = .mainMenu
        panel.hidesOnDeactivate = true
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient, .ignoresCycle]
        panel.becomesKeyOnlyIfNeeded = false
        self.panel = panel
        return panel
    }

    private func centeredFrame(size: NSSize) -> NSRect {
        let visible = NSScreen.main?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        let x = visible.midX - size.width / 2
        let y = visible.maxY - visible.height / 3 - size.height / 2
        return NSRect(x: x, y: y, width: size.width, height: size.height)
    }
}
```

- [ ] **Step 4: Implement ViewModel skeleton**

Create `macos/NativeHelper/NativeHelper/HelperViewModel.swift`:

```swift
import Foundation
import SwiftUI

@MainActor
public final class HelperViewModel: ObservableObject {
    @Published public var query = ""
    @Published public var isSelectingBibliography = true
    @Published public var bibliographies: [HelperBibliography] = []
    @Published public var references: [HelperReference] = []
    @Published public var selectedIndex: Int? = nil
    @Published public var errorMessage: String? = nil
    @Published public var failedPasteKey: String? = nil
    @Published public var theme: NativeHelperTheme = .latte

    private var libraryHandle: UnsafeMutableRawPointer?
    private var searchGeneration = 0
    private var searchTask: Task<Void, Never>?

    deinit {
        bibcitex_helper_free_library(libraryHandle)
    }

    public func prepareForOpen() {
        errorMessage = nil
        failedPasteKey = nil
        if isSelectingBibliography {
            query = ""
            selectedIndex = bibliographies.isEmpty ? nil : 0
        } else {
            query = ""
            references = []
            selectedIndex = nil
        }
    }

    public func applyTheme(_ theme: NativeHelperTheme) {
        self.theme = theme
    }

    public func updateQuery(_ value: String) {
        query = value
        errorMessage = nil
        failedPasteKey = nil
        if isSelectingBibliography {
            selectedIndex = filteredBibliographies.isEmpty ? nil : 0
        } else {
            scheduleSearch()
        }
    }

    public var filteredBibliographies: [HelperBibliography] {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard !trimmed.isEmpty else { return bibliographies }
        return bibliographies.filter {
            $0.name.lowercased().contains(trimmed)
                || $0.path.lowercased().contains(trimmed)
                || ($0.description?.lowercased().contains(trimmed) ?? false)
        }
    }

    private func scheduleSearch() {
        searchGeneration += 1
        let generation = searchGeneration
        let query = self.query
        searchTask?.cancel()
        searchTask = Task {
            try? await Task.sleep(nanoseconds: 120_000_000)
            guard !Task.isCancelled else { return }
            await runSearch(query: query, generation: generation)
        }
    }

    private func runSearch(query: String, generation: Int) async {
        guard let libraryHandle else { return }
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            references = []
            selectedIndex = nil
            return
        }
        let list = trimmed.withCString { ptr in
            bibcitex_helper_search_library(libraryHandle, ptr, strlen(ptr))
        }
        let mapped = HelperReference.fromFfiList(list)
        bibcitex_helper_free_reference_list(list)
        guard generation == searchGeneration else { return }
        references = mapped
        selectedIndex = mapped.isEmpty ? nil : 0
    }
}
```

- [ ] **Step 5: Verify Swift build**

Run:

```bash
xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug
```

Expected:

```text
** BUILD SUCCEEDED **
```

- [ ] **Step 6: Commit**

Create `/tmp/bibcitex-native-helper-task-7-commit.txt`:

```text
feat(helper): add native Swift panel state

Summary:
- implement the AppKit NSPanel shell for the SwiftUI helper
- add the SwiftUI view model search debounce and generation handling
- declare Swift-side FFI calls for Rust-owned library handles and search results

Rationale:
- move macOS helper lifecycle and UI state into native Swift while keeping Rust data ownership
- keep search responsive by avoiding main-thread blocking and stale result application

Tests:
- xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add macos/NativeHelper/NativeHelper/NativeHelperPanelController.swift macos/NativeHelper/NativeHelper/HelperViewModel.swift macos/NativeHelper/NativeHelper/NativeHelperModels.swift macos/NativeHelper/NativeHelper/NativeHelperFFI.swift
git commit -F /tmp/bibcitex-native-helper-task-7-commit.txt
```

## Task 8: Implement SwiftUI Helper View and Math Rendering

**Files:**
- Modify: `macos/NativeHelper/NativeHelper/HelperView.swift`
- Modify: `macos/NativeHelper/NativeHelper/MathChunkText.swift`
- Modify: `macos/NativeHelper/NativeHelper/NativeHelperTheme.swift`
- Modify: `macos/NativeHelper/NativeHelper/HelperViewModel.swift`

- [ ] **Step 1: Confirm visual checkpoint**

Ask user to confirm:

```text
Use Spotlight-like fixed-width panel, translucent rounded surface, search row, bibliography chip, lazy lists, Chinese error messages, and LaTeXSwiftUI for all math chunks.
```

Expected: user confirms or revises.

- [ ] **Step 2: Implement theme model**

Create `macos/NativeHelper/NativeHelper/NativeHelperTheme.swift`:

```swift
import SwiftUI

public enum NativeHelperTheme: Int32 {
    case latte = 0
    case mocha = 1

    var background: Color {
        switch self {
        case .latte: return Color(red: 0.94, green: 0.95, blue: 0.97).opacity(0.95)
        case .mocha: return Color(red: 0.12, green: 0.12, blue: 0.18).opacity(0.95)
        }
    }

    var foreground: Color {
        switch self {
        case .latte: return Color(red: 0.17, green: 0.18, blue: 0.25)
        case .mocha: return Color(red: 0.80, green: 0.82, blue: 0.90)
        }
    }

    var border: Color {
        switch self {
        case .latte: return Color.black.opacity(0.12)
        case .mocha: return Color.white.opacity(0.14)
        }
    }

    var selected: Color {
        switch self {
        case .latte: return Color.blue.opacity(0.12)
        case .mocha: return Color.blue.opacity(0.22)
        }
    }
}
```

- [ ] **Step 3: Implement math chunk renderer**

Create `macos/NativeHelper/NativeHelper/MathChunkText.swift`:

```swift
import LaTeXSwiftUI
import SwiftUI

public struct MathChunkText: View {
    let chunks: [HelperChunk]
    let theme: NativeHelperTheme

    public var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 0) {
            ForEach(chunks) { chunk in
                switch chunk.kind {
                case .normal:
                    Text(chunk.text)
                case .verbatim:
                    Text(chunk.text).font(.system(.body, design: .monospaced))
                case .math:
                    LaTeX("$\(chunk.text)$")
                }
            }
        }
        .foregroundStyle(theme.foreground)
    }
}
```

- [ ] **Step 4: Implement helper SwiftUI surface**

Create `macos/NativeHelper/NativeHelper/HelperView.swift`:

```swift
import SwiftUI

public struct HelperView: View {
    @ObservedObject var viewModel: HelperViewModel
    @FocusState private var searchFocused: Bool

    public var body: some View {
        VStack(spacing: 0) {
            header
            content
        }
        .frame(width: 760)
        .background(viewModel.theme.background)
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .stroke(viewModel.theme.border, lineWidth: 1)
        )
        .shadow(radius: 22, y: 12)
        .onAppear {
            searchFocused = true
        }
    }

    private var header: some View {
        HStack(spacing: 12) {
            Image(systemName: "magnifyingglass")
                .foregroundStyle(viewModel.theme.foreground.opacity(0.55))
            TextField(
                viewModel.isSelectingBibliography ? "搜索或选择文献库" : "搜索文献、作者、标题",
                text: Binding(
                    get: { viewModel.query },
                    set: { viewModel.updateQuery($0) }
                )
            )
            .textFieldStyle(.plain)
            .focused($searchFocused)
            .onSubmit {
                viewModel.activateSelection()
            }
            bibliographyChip
        }
        .padding(.horizontal, 16)
        .frame(height: 64)
    }

    private var bibliographyChip: some View {
        Button {
            viewModel.startSelectingBibliography()
        } label: {
            Text(viewModel.currentBibliographyName ?? "选择文献库")
                .lineLimit(1)
                .font(.caption)
                .padding(.horizontal, 10)
                .padding(.vertical, 5)
                .background(viewModel.theme.selected)
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private var content: some View {
        if viewModel.isSelectingBibliography {
            bibliographyList
        } else if viewModel.query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            EmptyView()
        } else {
            referenceList
        }
    }

    private var bibliographyList: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(Array(viewModel.filteredBibliographies.enumerated()), id: \.element.id) { index, item in
                    BibliographyRow(item: item, selected: index == viewModel.selectedIndex, theme: viewModel.theme)
                        .onTapGesture { viewModel.selectBibliography(item) }
                }
            }
        }
        .frame(maxHeight: 480)
    }

    private var referenceList: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(Array(viewModel.references.enumerated()), id: \.element.id) { index, item in
                    ReferenceRow(item: item, selected: index == viewModel.selectedIndex, theme: viewModel.theme)
                        .onTapGesture { viewModel.selectReference(item) }
                }
            }
        }
        .frame(maxHeight: 480)
    }
}
```

Append row views in the same file:

```swift
private struct BibliographyRow: View {
    let item: HelperBibliography
    let selected: Bool
    let theme: NativeHelperTheme

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(item.name).font(.headline)
                Spacer()
                Text(item.updatedAt).font(.caption).monospacedDigit()
            }
            if let description = item.description {
                Text(description).font(.caption).lineLimit(1)
            }
            Text(item.path).font(.caption2).lineLimit(1).foregroundStyle(theme.foreground.opacity(0.55))
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(selected ? theme.selected : Color.clear)
    }
}

private struct ReferenceRow: View {
    let item: HelperReference
    let selected: Bool
    let theme: NativeHelperTheme

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            MathChunkText(chunks: item.title, theme: theme)
                .lineLimit(2)
            HStack(spacing: 6) {
                Text(item.citeKey).font(.caption).monospaced()
                if let year = item.year {
                    Text(String(year)).font(.caption)
                }
                if let venue = item.venue {
                    Text(venue).font(.caption).lineLimit(1)
                }
            }
            .foregroundStyle(theme.foreground.opacity(0.65))
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(selected ? theme.selected : Color.clear)
    }
}
```

- [ ] **Step 5: Add keyboard and selection methods**

Extend `HelperViewModel.swift` with:

```swift
public var currentBibliographyName: String?

public func startSelectingBibliography() {
    isSelectingBibliography = true
    query = ""
    references = []
    selectedIndex = filteredBibliographies.isEmpty ? nil : 0
}

public func activateSelection() {
    if isSelectingBibliography {
        guard let selectedIndex, filteredBibliographies.indices.contains(selectedIndex) else { return }
        selectBibliography(filteredBibliographies[selectedIndex])
    } else {
        guard let selectedIndex, references.indices.contains(selectedIndex) else { return }
        selectReference(references[selectedIndex])
    }
}

public func selectBibliography(_ bibliography: HelperBibliography) {
    currentBibliographyName = bibliography.name
    isSelectingBibliography = false
    query = ""
    references = []
    selectedIndex = nil
}

public func selectReference(_ reference: HelperReference) {
    failedPasteKey = nil
    errorMessage = nil
}
```

- [ ] **Step 6: Verify Swift build**

Run:

```bash
xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug
```

Expected:

```text
** BUILD SUCCEEDED **
```

- [ ] **Step 7: Commit**

Create `/tmp/bibcitex-native-helper-task-8-commit.txt`:

```text
feat(helper): build SwiftUI helper surface

Summary:
- add the Spotlight-style SwiftUI helper surface with lazy bibliography and reference lists
- add native theme colors and LaTeXSwiftUI chunk rendering
- add selection state hooks for bibliography and reference actions

Rationale:
- migrate the macOS helper UI to SwiftUI while preserving the existing visual hierarchy
- render math chunks natively across helper text

Tests:
- xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add macos/NativeHelper/NativeHelper/HelperView.swift macos/NativeHelper/NativeHelper/MathChunkText.swift macos/NativeHelper/NativeHelper/NativeHelperTheme.swift macos/NativeHelper/NativeHelper/HelperViewModel.swift
git commit -F /tmp/bibcitex-native-helper-task-8-commit.txt
```

## Task 9: Add Bibliography List and Current Library FFI

**Files:**
- Modify: `src-tauri/src/native_helper/ffi.rs`
- Modify: `src-tauri/src/native_helper/state.rs`
- Modify: `src-tauri/src/native_helper/tests.rs`
- Modify: `macos/NativeHelper/NativeHelper/NativeHelperFFI.swift`
- Modify: `macos/NativeHelper/NativeHelper/HelperViewModel.swift`

- [ ] **Step 1: Write failing Rust bibliography list test**

Append to `src-tauri/src/native_helper/tests.rs`:

```rust
use crate::core::setting::BibliographyInfo;
use chrono::Local;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn bibliography_list_preserves_sort_ready_fields() {
    let now = Local::now();
    let mut values = BTreeMap::new();
    values.insert(
        "Main".to_string(),
        BibliographyInfo {
            path: PathBuf::from("/tmp/main.bib"),
            created_at: now,
            updated_at: now,
            description: Some("Primary".to_string()),
        },
    );

    let list = FfiBibliographyList::from_bibliographies(values);

    assert_eq!(list.len, 1);
    let first = unsafe { &*list.ptr };
    assert_eq!(unsafe { ffi_string_to_string(&first.name) }, "Main");
    assert_eq!(unsafe { ffi_string_to_string(&first.path) }, "/tmp/main.bib");
    assert!(first.description.has_value);

    unsafe { bibcitex_helper_free_bibliography_list(list) };
}
```

- [ ] **Step 2: Run test to verify RED**

Run:

```bash
cd src-tauri && cargo test bibliography_list_preserves_sort_ready_fields
```

Expected:

```text
FAIL with unresolved FfiBibliographyList
```

- [ ] **Step 3: Implement Rust bibliography FFI**

Add to `src-tauri/src/native_helper/ffi.rs`:

```rust
use crate::core::setting::BibliographyInfo;
use std::collections::BTreeMap;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiBibliography {
    pub name: FfiString,
    pub path: FfiString,
    pub updated_at: FfiString,
    pub description: FfiOptionalString,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiBibliographyList {
    pub ptr: *mut FfiBibliography,
    pub len: usize,
}

impl FfiBibliographyList {
    pub fn from_bibliographies(values: BTreeMap<String, BibliographyInfo>) -> Self {
        let mut values = values
            .into_iter()
            .map(|(name, info)| FfiBibliography {
                name: FfiString::from_string(name),
                path: FfiString::from_string(info.path.to_string_lossy().to_string()),
                updated_at: FfiString::from_string(info.updated_at.to_rfc3339()),
                description: FfiOptionalString::from_option(info.description),
            })
            .collect::<Vec<_>>();
        values.sort_by(|a, b| {
            // Swift performs display sorting too; Rust emits stable strings for direct consumers.
            unsafe_string_for_sort(&b.updated_at).cmp(&unsafe_string_for_sort(&a.updated_at))
        });
        let len = values.len();
        let ptr = values.as_mut_ptr();
        std::mem::forget(values);
        Self { ptr, len }
    }
}

fn unsafe_string_for_sort(value: &FfiString) -> String {
    if value.ptr.is_null() || value.len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr.cast::<u8>(), value.len) };
    String::from_utf8_lossy(bytes).to_string()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_free_bibliography_list(list: FfiBibliographyList) {
    if list.ptr.is_null() {
        return;
    }
    let values = unsafe { Vec::from_raw_parts(list.ptr, list.len, list.len) };
    for value in values {
        unsafe { free_string(value.name) };
        unsafe { free_string(value.path) };
        unsafe { free_string(value.updated_at) };
        unsafe { bibcitex_helper_free_optional_string(value.description) };
    }
}
```

Add exported getter to `src-tauri/src/native_helper/state.rs`:

```rust
use super::ffi::FfiBibliographyList;
use crate::core::setting::Setting;

#[unsafe(no_mangle)]
pub extern "C" fn bibcitex_helper_load_bibliographies() -> FfiBibliographyList {
    FfiBibliographyList::from_bibliographies(Setting::load().bibliographies)
}
```

- [ ] **Step 4: Run Rust tests**

Run:

```bash
cd src-tauri && cargo test native_helper
```

Expected:

```text
test result: ok
```

- [ ] **Step 5: Wire Swift bibliography loading**

Extend `NativeHelperFFI.swift` with:

```swift
public struct FfiBibliography {
    public var name: FfiString
    public var path: FfiString
    public var updated_at: FfiString
    public var description: FfiOptionalString
}

public struct FfiBibliographyList {
    public var ptr: UnsafeMutablePointer<FfiBibliography>?
    public var len: Int
}

@_silgen_name("bibcitex_helper_load_bibliographies")
func bibcitex_helper_load_bibliographies() -> FfiBibliographyList

@_silgen_name("bibcitex_helper_free_bibliography_list")
func bibcitex_helper_free_bibliography_list(_ list: FfiBibliographyList)
```

Extend `HelperViewModel.prepareForOpen()` to load bibliographies on first open:

```swift
if bibliographies.isEmpty {
    let list = bibcitex_helper_load_bibliographies()
    bibliographies = HelperBibliography.fromFfiList(list).sorted { $0.updatedAt > $1.updatedAt }
    bibcitex_helper_free_bibliography_list(list)
}
```

Implement `HelperBibliography.fromFfiList` in `NativeHelperModels.swift`.

- [ ] **Step 6: Verify Rust and Swift builds**

Run:

```bash
cd src-tauri && cargo test native_helper
xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug
```

Expected:

```text
cargo test exits 0
xcodebuild exits 0
```

- [ ] **Step 7: Commit**

Create `/tmp/bibcitex-native-helper-task-9-commit.txt`:

```text
feat(helper): load bibliographies through native ffi

Summary:
- add Rust FFI structures for bibliography list transfer
- expose settings-backed bibliography loading to Swift
- load and sort bibliography choices in the Swift helper view model

Rationale:
- preserve the existing helper first-open library selection flow in the native helper
- keep settings ownership in Rust while presenting native SwiftUI choices

Tests:
- cd src-tauri && cargo test native_helper
- xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add src-tauri/src/native_helper/ffi.rs src-tauri/src/native_helper/state.rs src-tauri/src/native_helper/tests.rs macos/NativeHelper/NativeHelper/NativeHelperFFI.swift macos/NativeHelper/NativeHelper/HelperViewModel.swift macos/NativeHelper/NativeHelper/NativeHelperModels.swift
git commit -F /tmp/bibcitex-native-helper-task-9-commit.txt
```

## Task 10: Wire Paste, Error Fallback, Theme Sync, and Frontend Theme Notification

**Files:**
- Modify: `src-tauri/src/native_helper/state.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `macos/NativeHelper/NativeHelper/HelperViewModel.swift`
- Modify: `macos/NativeHelper/NativeHelper/HelperView.swift`
- Modify: `src/tauri.ts`
- Modify: `src/context/AppContext.tsx`

- [ ] **Step 1: Add Rust paste FFI**

Add to `src-tauri/src/native_helper/state.rs`:

```rust
use crate::commands::paste_to_app;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bibcitex_helper_paste_cite_key(
    cite_key_ptr: *const std::ffi::c_char,
    cite_key_len: usize,
) -> bool {
    if cite_key_ptr.is_null() {
        return false;
    }
    let bytes = unsafe { std::slice::from_raw_parts(cite_key_ptr.cast::<u8>(), cite_key_len) };
    let Ok(cite_key) = std::str::from_utf8(bytes) else {
        return false;
    };
    paste_to_app(cite_key.to_string()).is_ok()
}
```

- [ ] **Step 2: Add native theme command in Rust**

Add command in `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub fn set_native_helper_theme(theme: String) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let theme_code = if theme == "mocha" { 1 } else { 0 };
        crate::native_helper::bridge::set_theme(theme_code);
    }
    Ok(())
}
```

Add it to `generate_handler!` in `src-tauri/src/lib.rs`.

- [ ] **Step 3: Add frontend wrapper**

Modify `src/tauri.ts`:

```ts
export async function setNativeHelperTheme(theme: "latte" | "mocha"): Promise<void> {
  return await invoke("set_native_helper_theme", { theme });
}
```

- [ ] **Step 4: Notify theme changes**

Modify `src/context/AppContext.tsx` inside the theme effect:

```ts
import { setNativeHelperTheme } from "@/tauri.ts";
```

and:

```ts
createEffect(() => {
  const nextTheme = theme();
  document.documentElement.setAttribute("data-theme", nextTheme);
  setNativeHelperTheme(nextTheme).catch((error) => {
    console.error("Failed to sync native helper theme:", error);
  });
});
```

Keep this wrapper safe on non-macOS because the Rust command is a no-op there.

- [ ] **Step 5: Wire Swift paste call**

Extend `NativeHelperFFI.swift`:

```swift
@_silgen_name("bibcitex_helper_paste_cite_key")
func bibcitex_helper_paste_cite_key(_ citeKey: UnsafePointer<CChar>?, _ len: Int) -> Bool
```

Implement in `HelperViewModel.selectReference`:

```swift
public func selectReference(_ reference: HelperReference) {
    let success = reference.citeKey.withCString { ptr in
        bibcitex_helper_paste_cite_key(ptr, strlen(ptr))
    }
    if success {
        NativeHelperPanelController.shared.hide()
    } else {
        failedPasteKey = reference.citeKey
        errorMessage = "粘贴失败，已保留 cite key，可手动复制: \(reference.citeKey)"
    }
}
```

- [ ] **Step 6: Show error and copy fallback in SwiftUI**

Add to `HelperView.swift` content below lists:

```swift
if let errorMessage = viewModel.errorMessage {
    HStack {
        Text(errorMessage)
            .font(.caption)
            .foregroundStyle(.red)
            .lineLimit(1)
        Spacer()
        if let failedPasteKey = viewModel.failedPasteKey {
            Button("复制") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(failedPasteKey, forType: .string)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.small)
        }
    }
    .padding(.horizontal, 12)
    .padding(.vertical, 8)
}
```

- [ ] **Step 7: Verify all checks**

Run:

```bash
bun run check
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cd src-tauri && cargo test
xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug
```

Expected:

```text
all commands exit 0
```

- [ ] **Step 8: Commit**

Create `/tmp/bibcitex-native-helper-task-10-commit.txt`:

```text
feat(helper): wire native paste and theme sync

Summary:
- expose Rust paste and native helper theme commands to Swift and the frontend
- synchronize main app theme changes into the native helper
- show native helper paste failures with a copy fallback

Rationale:
- preserve the existing automatic paste flow and fallback behavior in the SwiftUI helper
- keep macOS native helper appearance aligned with the main app theme

Tests:
- bun run check
- cd src-tauri && cargo fmt
- cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings
- cd src-tauri && cargo test
- xcodebuild build -project macos/NativeHelper/NativeHelper.xcodeproj -scheme NativeHelper -configuration Debug

Co-authored-by: Codex <noreply@openai.com>
```

Run:

```bash
git add src-tauri/src/native_helper/state.rs src-tauri/src/commands.rs src-tauri/src/lib.rs macos/NativeHelper/NativeHelper/HelperViewModel.swift macos/NativeHelper/NativeHelper/HelperView.swift macos/NativeHelper/NativeHelper/NativeHelperFFI.swift src/tauri.ts src/context/AppContext.tsx
git commit -F /tmp/bibcitex-native-helper-task-10-commit.txt
```

## Task 11: Full Verification and Manual macOS Acceptance

**Files:**
- No planned file changes.

- [ ] **Step 1: Run repository checks**

Run:

```bash
bun run check
bun run build
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cd src-tauri && cargo test
```

Expected:

```text
all commands exit 0
```

- [ ] **Step 2: Run Tauri dev build check**

Run:

```bash
bun run tauri:build
```

Expected:

```text
production Tauri build exits 0
```

Allow the production build to run for up to 10 minutes. If it is still running after that, report the elapsed time and current build phase to the user before deciding whether to continue.

- [ ] **Step 3: Manual macOS helper checklist**

Run the app and verify:

```text
Global shortcut opens SwiftUI helper.
Tray helper item opens SwiftUI helper.
Main window helper button opens SwiftUI helper.
No macOS /helper webview is created.
First open shows bibliography selection.
No bibliography shows Chinese empty state.
Bibliography chip returns to selection mode.
Search input filters bibliographies.
Selecting a bibliography loads it.
Search input returns Rust search results.
Keyboard up/down changes selection.
Enter pastes cite key through Rust xpaste.
Escape hides helper.
Mouse hover and click select rows.
Dynamic height grows and caps with scrolling.
Theme switch updates helper while open.
Inline and display math render through LaTeXSwiftUI.
Paste failure shows Chinese error and copy fallback.
```

- [ ] **Step 4: Stop on verification failures**

If any command or manual acceptance check fails, capture the exact failure, identify the owning task, and fix it in that task's files before rerunning the failed check plus any broader affected checks. Do not create a generic verification-fix commit.
