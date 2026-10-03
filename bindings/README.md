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

## User-operated generation

These scripts have not been executed by the coding agent. They require an already
built platform binding library. Running them compiles the generators and
creates bindings; the user controls when to do that.

macOS: `bash bindings/generate-swift.sh /absolute/path/libbibcitex_ffi.dylib`.
Output is under `bindings/generated/swift`. Both the runtime crate and generator
use `uniffi = "0.32"`; Cargo.lock currently resolves both to 0.32.2. Generation
disables formatting and retains UniFFI checksums
to detect mismatches. Generated files are consumed unchanged by the Swift host.

Swift imports `BibCiTeXCore`; its generated Clang module is `BibCiTeXCoreFFI`.
Swift functions/fields are lowerCamelCase: `helper_current` becomes
`helperCurrent()`, and `entry_type` becomes `entryType`.

Windows: `pwsh -File bindings/generate-csharp.ps1 -Library C:\path\bibcitex_csharp.dll`.
Its checked-in generator uses the same Rust inventory as the Windows DLL. Output
is under `bindings/generated/csharp` in namespace `BibCiTeX.Core`, loading
`bibcitex_csharp.dll`. Interoptopus generates the C# interop and typed Wire transport;
application code does not supply a C/JSON protocol. The `0.16` dependency constraints
resolve to 0.16.5 in Cargo.lock. Both generation scripts use `--locked`.

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
