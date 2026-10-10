# Rust bindings

macOS uses Mozilla UniFFI 0.32 and its Swift generator. Windows uses
Interoptopus 0.16 and its C# generator. Each platform selects its own binding
toolchain; both call the same Rust service, without duplicating business logic.

Sources of truth:

- `crates/bibcitex-service/src/api.rs`: shared operations and `CoreError`.
- `crates/bibcitex-service/src/records.rs`: shared records and `ChunkKind`.
- `crates/bibcitex-ffi/src/`: thin Swift exports and UniFFI remote type declarations.
- `crates/bibcitex-ffi/uniffi.toml`: Swift namespace configuration.
- `crates/bibcitex-csharp/src/`: thin Interoptopus exports and typed Wire requests/responses.
- `crates/xpaste`: clipboard, previous foreground tracking and verified paste.

## Build integration

Xcode's `RustBindings` target and Visual Studio's `Build/Rust.targets` own Rust
compilation and binding generation. Open the platform project and build directly;
neither IDE needs a prior script or Just invocation. Generated bindings are verified
by the platform integration tests.

The Xcode target invokes `bindings/generate-swift.sh` with its built Rust library
and an output directory in DerivedData. Both the runtime crate and generator
use `uniffi = "0.32"`; Cargo.lock currently resolves both to 0.32.2. Generation
disables formatting and retains UniFFI checksums
to detect mismatches. Generated files are consumed unchanged by the Swift host.

Swift imports `BibCiTeXCore`; its generated Clang module is `BibCiTeXCoreFFI`.
Swift functions/fields are lowerCamelCase: `helper_current` becomes
`helperCurrent()`, and `entry_type` becomes `entryType`.

MSBuild invokes the checked-in C# generator using the same Rust inventory as the Windows DLL. Output
is under `bindings/generated/csharp` in namespace `BibCiTeX.Core`, loading
`bibcitex_csharp.dll`. Interoptopus generates the C# interop and typed Wire transport;
application code does not supply a C/JSON protocol. The `0.16` dependency constraints
resolve to 0.16.5 in Cargo.lock. Both platform builds use `--locked` for Rust.

## Host contract

Calls are synchronous and may parse files: execute them on host background
executors, then publish UI changes on the UI thread. Ignore superseded search
results using a generation counter. Each binding library owns serialization,
memory management and panic conversion; application code consumes typed values.

Call `initialize()` before first activation of the main window. It starts a
once-only observer for the most recent external foreground window/application.
`libraries()` also initializes for compatibility. Before activating the helper,
await `capturePasteTarget()`: this snapshots the foreground external target or the
last external target if BibCiTeX already has focus. The observer does not change
this snapshot during a helper session. `paste(text)` copies the exact text, then
activates and verifies the target before Cmd+V / Ctrl+V. Windows verifies HWND plus
owning PID. macOS requires Accessibility permission. A failed focus/permission/key
operation throws and leaves the copied text available. If the helper is hidden
while pasting, restore its current state on failure so the existing error and
clipboard fallback remain reachable. Do not add LaTeX syntax around cite keys.

`libraries`, `addLibrary`, `removeLibrary` manage the existing registry.
`helperCurrent`, `helperSelect` store the helper's selection independently from
main-window selection. Removal deletes only a registry entry, never its file.
Settings stay at `dirs::config_dir()/BibCiTeX/setting.json`; invalid JSON is
reported without replacement. Unknown fields survive writes. Changes are serialized
within and across processes with the stable `setting.lock` sidecar, written to a
synced temporary file beside the destination, and atomically replaced. The sidecar
is retained so all app processes lock the same file even when settings are replaced. Return paths are canonical; use them unchanged for helperSelect.

`search(path, query, field, typeFilter)` returns all matches, never a UI-imposed
limit. Empty/whitespace queries preserve references with missing fields. Fields:
`all`, `author`, `title`, `journal`, `year`. Type filters: `all`, `Article`, `Book`,
`Thesis` (including MastersThesis and PhdThesis), `Booklet`, `InBook`,
`InCollection`, `InProceedings`, `Misc`, `TechReport`.

Ordinary search treats whitespace-separated terms as AND conditions. In `all`,
each term may match a different existing field; a selected field bounds every
term. Short and full journal names are both searched. Matching tolerates case,
Unicode representation, accents and common punctuation while preserving symbols
such as `C++` and `C#`. Years require complete matches. Titles and notes are
searched across their original chunk boundaries without changing those chunks;
formula operators are preserved during punctuation folding. Repeated keywords
do not change relevance ordering.
Nonempty results use deterministic relevance ordering: exact cite keys and
strong text matches precede partial matches. Small result sets may be supplemented
with one-edit spelling matches for Latin words of 5–64 letters in titles, authors
and journals; all AND terms still apply. Query text has no advanced-search syntax.
Parsed records and preprocessed search texts share one file-version snapshot;
the cache keeps up to eight libraries and 64 MiB of estimated search allocations,
evicting the least recently used snapshots. Returned record IDs and metadata are
unchanged.

Records preserve every original metadata field. `ReferenceRecord.id` is path +
U+001F + cite key. `entryType` contains enum names such as Article/InBook, not
BibTeX's lowercase Display representation. `abstractText` replaces the Rust core's
`abstract_` name. Author/publisher/organization/editor/chunk collections are empty
when absent; other optional fields remain optional. Chunks preserve Normal,
Verbatim and Math separately. Numeric volume/edition are 64-bit; pages remain
unsigned 32-bit ranges with core end semantics.

## Primary sources

- [Mozilla UniFFI proc macros](https://mozilla.github.io/uniffi-rs/latest/proc_macro/index.html)
- [Mozilla UniFFI Swift configuration](https://mozilla.github.io/uniffi-rs/latest/swift/configuration.html)
