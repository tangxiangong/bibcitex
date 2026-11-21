import { batch, computed, effect, signal, untracked } from "nsh";
import {
  Catalog,
  SearchSession,
  type LibraryDto,
  type QueryAnalysisDto,
  type QueryDocumentDto,
  type ReferenceDto,
  type ReferenceLink,
  type SearchContextDto,
  type SelectionDto,
} from "bibcitex/native";
import { duration } from "../theme";
import type { CopySurface } from "../ui";
import { libraryByName, libraryByUid, libraryList, librariesReady, prefs, savePreference, showError, tagsChanged } from "./desktop";
import { referenceKey } from "./format";
import { emptyTagFilter, tagFilterDocument, type TagFilter } from "./tagFilter";

/** One query draft: ordinary and advanced search keep their own. */
interface Draft {
  query: string;
  /** The bound tree the advanced editor produced for `query`, if any. */
  document: QueryDocumentDto | null;
  tagRevision: string | null;
}

const emptyDraft: Draft = { query: "", document: null, tagRevision: null };

export type WorkbenchSurface = "main" | "tray";

/**
 * A workbench's search state: its own `SearchSession`, query drafts,
 * filters, selection, batch selection and copy confirmations. The main
 * window and the tray each own one, so neither ever shows the other's
 * results or confirmations.
 *
 * Create it inside a component (the app root): its effects follow the
 * library registry and the tag catalog for as long as that owner lives.
 */
export function createWorkbench(surface: WorkbenchSurface) {
  const session = new SearchSession(surface);
  const libraryUid = signal<string | null>(null);
  const advanced = signal(false);
  const drafts = signal<{ ordinary: Draft; advanced: Draft }>({ ordinary: emptyDraft, advanced: emptyDraft });
  const field = signal("all");
  const type = signal("all");
  const tagFilter = signal<TagFilter>(emptyTagFilter);
  const references = signal<readonly ReferenceDto[]>([]);
  const context = signal<SearchContextDto | null>(null);
  const analysis = signal<QueryAnalysisDto | null>(null);
  const loading = signal(false);
  const selected = signal<string | null>(null);
  const batchActive = signal(false);
  const batchIds = signal<ReadonlySet<string>>(new Set());
  const copied = signal<Partial<Record<CopySurface, string>>>({});
  const composing = signal(false);

  const draft = (): Draft => (advanced() ? drafts().advanced : drafts().ordinary);
  const library = computed((): LibraryDto | undefined => libraryByUid(libraryUid()));
  const reference = computed((): ReferenceDto | undefined => {
    const key = selected();
    return key ? references().find((r) => referenceKey(r) === key) : undefined;
  });
  const selectedReferences = computed(() => {
    const ids = batchIds();
    return references().filter((r) => ids.has(referenceKey(r)));
  });

  let generation = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;
  const copyTimers: Partial<Record<CopySurface, ReturnType<typeof setTimeout>>> = {};

  function clearResults(): void {
    batch(() => {
      references.set([]);
      context.set(null);
      analysis.set(null);
      batchIds.set(new Set());
    });
  }

  /** Starts a search for the current inputs after a short pause; anything
   * older is dropped, and actions wait for the new result's context. */
  function schedule(options: { immediate?: boolean } = {}): void {
    const version = ++generation;
    if (timer) clearTimeout(timer);
    timer = null;
    context.set(null);
    if (composing()) return;
    const lib = library();
    if (!lib || !lib.available) {
      session.invalidate();
      batch(() => {
        clearResults();
        selected.set(null);
        loading.set(false);
      });
      return;
    }
    loading.set(true);
    timer = setTimeout(() => void run(version, lib), options.immediate ? 0 : 100);
  }

  async function run(version: number, lib: LibraryDto): Promise<void> {
    const d = untracked(draft);
    const isAdvanced = untracked(advanced);
    const bound = isAdvanced && d.document !== null;
    const previous = untracked(selected);
    try {
      const reply = await session.search(
        {
          path: lib.path,
          mode: isAdvanced ? "advanced" : "ordinary",
          inputKind: bound ? "bound" : "text",
          text: d.query,
          document: bound ? d.document : null,
          expectedTagRevision: bound ? d.tagRevision : null,
          defaultField: untracked(field),
          typeFilter: untracked(type),
          queryVersion: String(version),
        },
        tagFilterDocument(untracked(tagFilter)),
      );
      if (version !== generation) return;
      batch(() => {
        // A refreshed projection (renamed tags) replaces the visible text.
        if (bound && reply.analysis.document && reply.analysis.text !== d.query) {
          setDraft({ query: reply.analysis.text, document: reply.analysis.document, tagRevision: reply.analysis.tagRevision ?? d.tagRevision });
        }
        references.set(reply.references);
        context.set(reply.context ?? null);
        analysis.set(reply.analysis);
        batchIds.set(new Set());
        const keys = reply.references.map(referenceKey);
        selected.set(previous && keys.includes(previous) ? previous : (keys[0] ?? null));
        loading.set(false);
      });
    } catch (error) {
      if (version !== generation) return;
      batch(() => {
        clearResults();
        selected.set(null);
        loading.set(false);
      });
      showError(error);
    }
  }

  function setDraft(next: Draft): void {
    drafts.update((all) => (untracked(advanced) ? { ...all, advanced: next } : { ...all, ordinary: next }));
  }

  // The registry decides which library is shown: the saved one, else the
  // first available. A missing saved library stays selected as unavailable.
  let restored = false;
  /** A library just added, selected once the registry lists it. */
  let pendingUid: string | null = null;
  const prefsLoaded = computed(() => prefs() !== null);
  effect(() => {
    const libs = libraryList();
    if (!librariesReady() || !prefsLoaded()) return;
    untracked(() => {
      const saved = prefs();
      if (!saved) return;
      if (!restored) {
        restored = true;
        advanced.set(surface === "main" ? saved.mainAdvancedSearchEnabled : saved.trayAdvancedSearchEnabled);
      }
      if (pendingUid && libs.some((l) => l.uid === pendingUid)) {
        const uid = pendingUid;
        pendingUid = null;
        libraryUid.set(null);
        selectLibrary(uid);
        return;
      }
      const current = libraryUid();
      if (current && libs.some((l) => l.uid === current)) {
        schedule();
        return;
      }
      const savedName = surface === "main" ? saved.library : saved.trayLibrary;
      const next = libraryByName(savedName) ?? libs.find((l) => l.available);
      libraryUid.set(next?.uid ?? null);
      session.invalidate();
      schedule({ immediate: true });
    });
  });

  // Tag edits change rows (their chips) and may rename bound conditions.
  let seenTags = untracked(tagsChanged);
  effect(() => {
    const revision = tagsChanged();
    if (revision === seenTags) return;
    seenTags = revision;
    untracked(() => schedule());
  });

  function selectLibrary(uid: string): void {
    if (libraryUid() === uid) return;
    const lib = libraryByUid(uid);
    if (!lib) {
      pendingUid = uid;
      return;
    }
    session.invalidate();
    batch(() => {
      libraryUid.set(uid);
      selected.set(null);
      batchActive.set(false);
      clearResults();
    });
    savePreference(surface === "main" ? { library: lib.name } : { trayLibrary: lib.name });
    schedule({ immediate: true });
  }

  function setQuery(text: string): void {
    if (draft().query === text) return;
    setDraft({ query: text, document: null, tagRevision: null });
    if (text === "") session.invalidate();
    schedule();
  }

  function applyAdvancedQuery(text: string, document: QueryDocumentDto | null, tagRevision: string | null): void {
    setDraft({ query: text, document, tagRevision });
    schedule({ immediate: true });
  }

  function setAdvanced(enabled: boolean): void {
    if (advanced() === enabled) return;
    advanced.set(enabled);
    savePreference(surface === "main" ? { mainAdvancedSearchEnabled: enabled } : { trayAdvancedSearchEnabled: enabled });
    session.invalidate();
    schedule({ immediate: true });
  }

  function setComposing(active: boolean): void {
    composing.set(active);
    if (active) {
      generation++;
      if (timer) clearTimeout(timer);
      session.invalidate();
      context.set(null);
    } else schedule();
  }

  function selection(refs: readonly ReferenceDto[]): SelectionDto | null {
    const ctx = context();
    if (!ctx || loading() || refs.length === 0) return null;
    return { context: ctx, targets: refs.map((r) => ({ libraryUid: r.libraryUid, recordUid: r.recordUid })) };
  }

  function confirmCopy(where: CopySurface, token: string): void {
    const existing = copyTimers[where];
    if (existing) clearTimeout(existing);
    copied.update((all) => ({ ...all, [where]: token }));
    copyTimers[where] = setTimeout(() => {
      copied.update((all) => {
        const next = { ...all };
        delete next[where];
        return next;
      });
    }, duration.copiedFeedback);
  }

  /** The confirmation token a copy of `refs` leaves on `where`. */
  const citeToken = (refs: readonly ReferenceDto[]): string => `cite:${refs.map(referenceKey).join(",")}`;
  const bibtexToken = (r: ReferenceDto): string => `bibtex:${referenceKey(r)}`;

  async function copy(refs: readonly ReferenceDto[], kind: "citation" | "bibtex", where: CopySurface): Promise<void> {
    const sel = selection(refs);
    if (!sel) return;
    const version = generation;
    try {
      await session.copy(sel, kind);
      if (version !== generation) return;
      confirmCopy(where, kind === "citation" ? citeToken(refs) : bibtexToken(refs[0]));
    } catch (error) {
      if (version !== generation) return;
      schedule();
      showError(error);
    }
  }

  async function open(r: ReferenceDto, kind: ReferenceLink): Promise<void> {
    const sel = selection([r]);
    if (!sel) return;
    try {
      await session.openReference(sel, kind);
    } catch (error) {
      showError(error);
    }
  }

  function move(delta: -1 | 1): void {
    const rows = references();
    if (loading() || rows.length === 0) return;
    const keys = rows.map(referenceKey);
    const index = keys.indexOf(selected() ?? "");
    const next = index < 0 ? (delta < 0 ? keys.length - 1 : 0) : Math.min(Math.max(index + delta, 0), keys.length - 1);
    select(keys[next]);
  }

  function select(key: string | null): void {
    batch(() => {
      selected.set(key);
      if (!batchActive()) batchIds.set(new Set(key ? [key] : []));
    });
  }

  function toggleBatchItem(key: string): void {
    batch(() => {
      if (!batchActive()) batchActive.set(true);
      const next = new Set(batchIds());
      if (next.has(key)) next.delete(key);
      else next.add(key);
      batchIds.set(next);
      selected.set(key);
    });
  }

  function setBatchActive(active: boolean): void {
    batch(() => {
      batchActive.set(active);
      batchIds.set(new Set());
    });
  }

  function selectAll(): void {
    batchIds.set(new Set(references().map(referenceKey)));
  }

  function clearBatch(): void {
    batchIds.set(new Set());
  }

  async function setPinned(lib: LibraryDto, pinned: boolean): Promise<void> {
    try {
      await Catalog.setPinned(lib.name, pinned);
    } catch (error) {
      showError(error);
    }
  }

  async function removeLibrary(lib: LibraryDto): Promise<void> {
    try {
      await Catalog.remove(lib.name);
    } catch (error) {
      showError(error);
    }
  }

  function refresh(): void {
    session.invalidate();
    schedule({ immediate: true });
  }

  return {
    surface,
    // Reactive reads.
    libraryUid: () => libraryUid(),
    library: () => library(),
    advanced: () => advanced(),
    query: () => draft().query,
    field: () => field(),
    type: () => type(),
    tagFilter: () => tagFilter(),
    references: () => references(),
    analysis: () => analysis(),
    loading: () => loading(),
    selected: () => selected(),
    reference: () => reference(),
    batchActive: () => batchActive(),
    batchIds: () => batchIds(),
    selectedReferences: () => selectedReferences(),
    /** Whether `where`'s last copy was `token` (see `citeToken`). */
    copiedOn: (where: CopySurface) => copied()[where] ?? null,
    ready: () => context() !== null,
    citeToken,
    bibtexToken,
    // Changes.
    selectLibrary,
    setQuery,
    applyAdvancedQuery,
    advancedDraft: () => ({ text: drafts().advanced.query, document: drafts().advanced.document }),
    setAdvanced,
    setField: (value: string) => {
      if (field() === value) return;
      field.set(value);
      schedule();
    },
    setType: (value: string) => {
      if (type() === value) return;
      type.set(value);
      schedule();
    },
    setTagFilter: (value: TagFilter) => {
      tagFilter.set(value);
      schedule();
    },
    setComposing,
    move,
    select,
    toggleBatchItem,
    setBatchActive,
    selectAll,
    clearBatch,
    cite: (refs: readonly ReferenceDto[], where: CopySurface = "list") => copy(refs, "citation", where),
    copyBibtex: (r: ReferenceDto, where: CopySurface = "detail") => copy([r], "bibtex", where),
    open,
    setPinned,
    removeLibrary,
    refresh,
  };
}

export type Workbench = ReturnType<typeof createWorkbench>;
