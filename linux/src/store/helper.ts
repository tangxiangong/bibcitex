import { batch, computed, effect, signal, untracked } from "nsh";
import {
  Catalog,
  Query,
  SearchSession,
  Tags,
  type CapturedTarget,
  type LibraryDto,
  type QueryAnalysisDto,
  type QueryDocumentDto,
  type ReferenceDto,
  type SearchContextDto,
} from "bibcitex/native";
import { tr } from "../i18n";
import { helper as helperMetrics, helperHeight } from "../theme";
import { desktop, libraryList, tagsChanged } from "./desktop";
import { errorText, referenceKey } from "./format";

/** A typed condition shown as a capsule before the free text. */
export interface Capsule {
  id: number;
  text: string;
  /** The bound tree `Query.condition` built: tag leaves keep their stable
   * IDs, so a rename or merge refreshes the capsule instead of rebinding
   * it by name. */
  document: QueryDocumentDto;
}

/** `@` a type, `#` a tag, `$` a field (then its value). */
export type CandidateKind = "@" | "#" | "$";

export interface Candidate {
  id: string;
  name: string;
}

interface CandidateState {
  kind: CandidateKind;
  filter: string;
  index: number;
  /** For `$`: the chosen field, whose value is typed next. */
  field: string | null;
}

export interface HelperError {
  message: string;
  /** A failed paste keeps its citation for manual copying. */
  ticket: string | null;
}

const candidateRowHeight = 28;
const candidateListMax = 280;

/**
 * The compact helper's state: the external target captured before it shows,
 * its own `SearchSession`, capsules, library selection and the paste
 * transaction. Create it once in the app root.
 */
export function createHelper(hidePanel: () => void, showPanel: () => void) {
  const session = new SearchSession("helper");
  const target = signal<CapturedTarget | null>(null);
  const library = signal<LibraryDto | null>(null);
  const input = signal("");
  const capsules = signal<readonly Capsule[]>([]);
  const selectingLibrary = signal(false);
  const libraryQuery = signal("");
  const libraryIndex = signal(0);
  const results = signal<readonly ReferenceDto[]>([]);
  const context = signal<SearchContextDto | null>(null);
  const analysis = signal<QueryAnalysisDto | null>(null);
  const loading = signal(false);
  const index = signal<number | null>(null);
  const picked = signal<ReadonlySet<string>>(new Set());
  const error = signal<HelperError | null>(null);
  const candidate = signal<CandidateState | null>(null);
  const candidateSource = signal<readonly Candidate[]>([]);
  const composing = signal(false);
  const focusRequest = signal(0);

  let generation = 0;
  let capsuleId = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let pasting = false;
  let committing = false;
  /** The ticket `pasteFailed` reported during the paste in flight. */
  let failureTicket: string | null = null;
  /** Bumped by every begin and end: a capture that resolves after the panel
   * was dismissed is released, not used. */
  let session_ = 0;
  /** The tag metadata revision the capsules' bound IDs were resolved at. */
  let tagRevision: string | null = null;

  const queryText = (): string => [...capsules().map((c) => c.text), input().trim()].filter((p) => p !== "").join(" ");

  const filteredLibraries = computed(() => {
    const q = libraryQuery().trim().toLowerCase();
    const all = libraryList();
    return q === "" ? all : all.filter((l) => l.name.toLowerCase().includes(q) || l.path.toLowerCase().includes(q));
  });

  const candidates = computed((): readonly Candidate[] => {
    const state = candidate();
    if (!state || state.field !== null) return [];
    const q = state.filter.trim().toLowerCase();
    const all = candidateSource();
    return q === "" ? all : all.filter((c) => c.name.toLowerCase().includes(q) || c.id.toLowerCase().includes(q));
  });

  /** The idle helper is the search row alone; a list appears only for a
   * query, a search in flight or library selection. */
  const showsList = (): boolean => loading() || selectingLibrary() || queryText() !== "";
  const showsBottomBar = (): boolean => !selectingLibrary() && candidate() === null && results().length > 0;

  const height = computed((): number => {
    const extra = (error() ? helperMetrics.errorBar : 0) + (diagnosticsShown() ? 20 : 0);
    const state = candidate();
    if (state) {
      if (state.field !== null) return helperMetrics.header + extra;
      const rows = Math.max(candidates().length, 1);
      return helperMetrics.header + 1 + Math.min(rows * candidateRowHeight + 12, candidateListMax) + extra;
    }
    if (!showsList()) return helperMetrics.header + extra;
    if (selectingLibrary()) {
      const rows = filteredLibraries().length;
      return rows === 0 ? helperMetrics.header + 1 + helperMetrics.emptyRow + extra : helperHeight(rows, "library", extra);
    }
    if (loading() && results().length === 0) return helperMetrics.header + 1 + helperMetrics.emptyRow + extra;
    const rows = results().length;
    return rows === 0 ? helperMetrics.header + 1 + helperMetrics.emptyRow + extra : helperHeight(rows, "reference", extra);
  });

  function diagnosticsShown(): boolean {
    return (analysis()?.diagnostics.length ?? 0) > 0;
  }

  function invalidate(): void {
    generation++;
    if (timer) clearTimeout(timer);
    timer = null;
    session.invalidate();
    context.set(null);
  }

  function schedule(): void {
    invalidate();
    const version = generation;
    const lib = untracked(library);
    const text = untracked(queryText);
    if (untracked(composing)) return;
    if (!lib || !lib.available || text === "") {
      batch(() => {
        results.set([]);
        analysis.set(null);
        index.set(null);
        picked.set(new Set());
        loading.set(false);
      });
      return;
    }
    loading.set(true);
    timer = setTimeout(() => void run(version, lib, text), 80);
  }

  /** The capsules' bound trees and the free text joined with AND, rendered
   * to its canonical projection; `null` when the free text does not parse
   * (the text search then reports the diagnostics). */
  async function boundQuery(free: string): Promise<QueryAnalysisDto | null> {
    let doc: QueryDocumentDto | null = null;
    const append = async (operand: QueryDocumentDto): Promise<boolean> => {
      if (!doc) {
        doc = operand;
        return true;
      }
      const joined = await Query.edit({ document: doc, action: "appendAnd", target: null, operand });
      if (joined.status !== "valid" || !joined.document) return false;
      doc = joined.document;
      return true;
    };
    for (const capsule of untracked(capsules)) if (!(await append(capsule.document))) return null;
    if (free !== "") {
      const parsed = await Query.analyze(free);
      if (parsed.status !== "valid" || !parsed.document || !(await append(parsed.document))) return null;
    }
    if (!doc) return null;
    const rendered = await Query.render(doc);
    return rendered.status === "valid" && rendered.document ? rendered : null;
  }

  async function run(version: number, lib: LibraryDto, text: string, retried = false): Promise<void> {
    try {
      const bound = untracked(capsules).length > 0 ? await boundQuery(untracked(input).trim()) : null;
      if (version !== generation) return;
      const reply = await session.search({
        path: lib.path,
        mode: "advanced",
        inputKind: bound ? "bound" : "text",
        text: bound ? bound.text : text,
        document: bound ? bound.document : null,
        expectedTagRevision: bound ? (bound.tagRevision ?? tagRevision) : null,
        defaultField: "all",
        typeFilter: "all",
        queryVersion: String(version),
      });
      if (version !== generation) return;
      // Tags changed since the capsules were bound: refresh them, search again.
      if (bound && reply.analysis.status === "staleQuery" && !retried) {
        await refreshCapsules();
        if (version === generation) await run(version, lib, untracked(queryText), true);
        return;
      }
      batch(() => {
        results.set(reply.references);
        context.set(reply.context ?? null);
        analysis.set(reply.analysis);
        index.set(reply.references.length > 0 ? 0 : null);
        picked.set(new Set());
        loading.set(false);
      });
    } catch (e) {
      if (version !== generation) return;
      batch(() => {
        results.set([]);
        loading.set(false);
        error.set({ message: tr("搜索失败：{0}", errorText(e)), ticket: null });
      });
    }
  }

  async function loadLibrary(): Promise<void> {
    try {
      const current = await Catalog.helperCurrent();
      library.set(current ?? untracked(libraryList).find((l) => l.available) ?? null);
    } catch (e) {
      error.set({ message: tr("加载文献库失败：{0}", errorText(e)), ticket: null });
    }
  }

  // Another window switched, renamed, moved or removed a library: read the
  // helper's selection again, drop results and prepared pastes of the old
  // source, and search again while a query is active.
  let seenLibraries = untracked(libraryList);
  effect(() => {
    const libs = libraryList();
    if (libs === seenLibraries) return;
    seenLibraries = libs;
    untracked(() => {
      invalidate();
      void loadLibrary().then(() => {
        if (untracked(queryText) !== "") schedule();
      });
    });
  });

  /** Resolves the capsules' bound tags again (renames, merges, deletions). */
  async function refreshCapsules(): Promise<void> {
    const current = untracked(capsules);
    if (current.length === 0) return;
    const refreshed = await Promise.all(
      current.map(async (capsule) => {
        try {
          const result = await Query.refresh(capsule.document);
          if (result.tagRevision) tagRevision = result.tagRevision;
          return result.document ? { ...capsule, text: result.text, document: result.document } : capsule;
        } catch {
          return capsule;
        }
      }),
    );
    // Keep capsules removed or added meanwhile as the user left them.
    capsules.update((all) => all.map((c) => refreshed.find((r) => r.id === c.id) ?? c));
  }

  let seenTags = untracked(tagsChanged);
  effect(() => {
    const revision = tagsChanged();
    if (revision === seenTags) return;
    seenTags = revision;
    untracked(() => {
      void refreshCapsules().then(() => {
        if (untracked(queryText) !== "") schedule();
      });
    });
  });

  /** Captures the external target, then resets to the idle search row.
   * Called before the panel takes the focus. */
  async function begin(): Promise<boolean> {
    const mine = ++session_;
    const previous = untracked(target);
    if (previous && !committing) desktop.cancelPaste(previous.ticket);
    target.set(null);
    invalidate();
    batch(() => {
      input.set("");
      capsules.set([]);
      selectingLibrary.set(false);
      libraryQuery.set("");
      candidate.set(null);
      results.set([]);
      analysis.set(null);
      index.set(null);
      picked.set(new Set());
      error.set(null);
      loading.set(false);
    });
    let captured: CapturedTarget | null = null;
    try {
      captured = await desktop.captureTarget();
    } catch {
      captured = null;
    }
    if (mine !== session_) {
      if (captured) desktop.cancelPaste(captured.ticket);
      return false;
    }
    target.set(captured);
    void loadLibrary();
    return true;
  }

  /** The panel hid without pasting: release the captured target. */
  function end(): void {
    if (committing) return;
    session_++;
    invalidate();
    const t = untracked(target);
    if (t) desktop.cancelPaste(t.ticket);
    target.set(null);
  }

  function requestFocus(): void {
    focusRequest.update((n) => n + 1);
  }

  // Text and triggers.

  function setInput(value: string): void {
    if (selectingLibrary()) {
      batch(() => {
        libraryQuery.set(value);
        libraryIndex.set(0);
      });
      return;
    }
    const previous = input();
    const typed = value.length === previous.length + 1 && value.startsWith(previous) ? value.slice(-1) : "";
    if ((typed === "@" || typed === "#" || typed === "$") && (previous === "" || /\s$/.test(previous))) {
      input.set(previous);
      beginCandidates(typed);
      return;
    }
    input.set(value);
    if (error()?.ticket == null) error.set(null);
    schedule();
  }

  function setComposing(active: boolean): void {
    composing.set(active);
    if (active) invalidate();
    else schedule();
  }

  function beginCandidates(kind: CandidateKind): void {
    candidate.set({ kind, filter: "", index: 0, field: null });
    candidateSource.set([]);
    const catalog = (() => {
      try {
        return Query.catalog();
      } catch {
        return null;
      }
    })();
    if (kind === "@") {
      candidateSource.set((catalog?.typeDescriptors ?? []).map((t) => ({ id: t.name, name: tr(t.labelKey) })));
    } else if (kind === "$") {
      candidateSource.set(
        (catalog?.fieldDescriptors ?? [])
          .filter((f) => f.matchMode !== "tag" && f.matchMode !== "type")
          .map((f) => ({ id: f.name, name: tr(f.labelKey) })),
      );
    } else {
      Tags.catalog()
        .then((snapshot) => {
          if (candidate()?.kind === "#") candidateSource.set(snapshot.tags.map((t) => ({ id: t.id, name: t.name })));
        })
        .catch((e) => error.set({ message: errorText(e), ticket: null }));
    }
  }

  function setCandidateFilter(value: string): void {
    candidate.update((state) => (state ? { ...state, filter: value, index: 0 } : state));
  }

  function moveCandidate(delta: -1 | 1): void {
    const count = candidates().length;
    if (count === 0) return;
    candidate.update((state) => (state ? { ...state, index: Math.min(Math.max(state.index + delta, 0), count - 1) } : state));
  }

  function cancelCandidates(): void {
    candidate.set(null);
    requestFocus();
  }

  async function addCondition(fieldName: string, value: string, tagId: string | null): Promise<void> {
    try {
      const result = await Query.condition({ fieldName, textValue: value, phrase: false, exclude: false, tagId });
      if (result.status !== "valid" || !result.document) {
        error.set({ message: result.diagnostics.map((d) => tr(d.code)).join(" · "), ticket: null });
        return;
      }
      batch(() => {
        if (result.tagRevision) tagRevision = result.tagRevision;
        capsules.update((all) => [...all, { id: ++capsuleId, text: result.text, document: result.document! }]);
        candidate.set(null);
      });
      requestFocus();
      schedule();
    } catch (e) {
      error.set({ message: errorText(e), ticket: null });
    }
  }

  function confirmCandidate(at?: number): void {
    const state = candidate();
    if (!state) return;
    if (state.field !== null) {
      const value = state.filter.trim();
      if (value === "") cancelCandidates();
      else void addCondition(state.field, value, null);
      return;
    }
    const choice = candidates()[at ?? state.index];
    if (!choice) return;
    if (state.kind === "$") {
      candidate.set({ ...state, field: choice.id, filter: "", index: 0 });
      return;
    }
    if (state.kind === "@") void addCondition("type", choice.id, null);
    else void addCondition("tag", choice.name, choice.id);
  }

  function removeCapsule(id: number): void {
    capsules.update((all) => all.filter((c) => c.id !== id));
    requestFocus();
    schedule();
  }

  /** Backspace in an empty field takes the last capsule back as text. */
  function deletePreviousCapsule(): void {
    const all = capsules();
    const last = all[all.length - 1];
    if (!last || selectingLibrary() || candidate()) return;
    batch(() => {
      capsules.set(all.slice(0, -1));
      input.set(last.text);
    });
    schedule();
  }

  // Libraries.

  function startSelectingLibrary(): void {
    batch(() => {
      candidate.set(null);
      selectingLibrary.set(true);
      libraryQuery.set("");
      const uid = library()?.uid;
      const at = filteredLibraries().findIndex((l) => l.uid === uid);
      libraryIndex.set(at < 0 ? 0 : at);
    });
    requestFocus();
  }

  function stopSelectingLibrary(): void {
    selectingLibrary.set(false);
    requestFocus();
  }

  async function chooseLibrary(lib: LibraryDto): Promise<void> {
    if (!lib.available) return;
    try {
      const chosen = await Catalog.helperSelect(lib.name, lib.path);
      batch(() => {
        library.set(chosen);
        selectingLibrary.set(false);
        error.set(null);
      });
      requestFocus();
      schedule();
    } catch (e) {
      error.set({ message: errorText(e), ticket: null });
    }
  }

  // Keyboard.

  function move(delta: -1 | 1): void {
    if (candidate()) {
      moveCandidate(delta);
      return;
    }
    if (selectingLibrary()) {
      const count = filteredLibraries().length;
      if (count > 0) libraryIndex.set(Math.min(Math.max(libraryIndex() + delta, 0), count - 1));
      return;
    }
    const count = results().length;
    if (count === 0) return;
    const current = index();
    batch(() => {
      index.set(current === null ? (delta < 0 ? count - 1 : 0) : Math.min(Math.max(current + delta, 0), count - 1));
      picked.set(new Set());
    });
  }

  function submit(): void {
    if (candidate()) {
      confirmCandidate();
      return;
    }
    if (selectingLibrary()) {
      const lib = filteredLibraries()[libraryIndex()];
      if (lib) void chooseLibrary(lib);
      return;
    }
    activateSelection();
  }

  /** Escape closes the innermost layer: candidates, library selection, the panel. */
  function escape(): void {
    if (candidate()) cancelCandidates();
    else if (selectingLibrary()) stopSelectingLibrary();
    else hidePanel();
  }

  function tab(): void {
    if (candidate()) return;
    if (selectingLibrary()) stopSelectingLibrary();
    else startSelectingLibrary();
  }

  // Selection and paste.

  function pasteSelection(): readonly ReferenceDto[] {
    const rows = results();
    const set = picked();
    if (set.size > 0) return rows.filter((r) => set.has(referenceKey(r)));
    const at = index();
    return at !== null && rows[at] ? [rows[at]] : [];
  }

  /** Command-click toggles a row; Shift-click extends from the current one. */
  function pick(at: number, mode: "toggle" | "extend"): void {
    const rows = results();
    const row = rows[at];
    if (!row) return;
    const next = new Set(picked());
    if (next.size === 0 && index() !== null && rows[index()!]) next.add(referenceKey(rows[index()!]));
    if (mode === "extend" && index() !== null) {
      const [from, to] = [Math.min(index()!, at), Math.max(index()!, at)];
      for (let i = from; i <= to; i++) next.add(referenceKey(rows[i]));
    } else if (next.has(referenceKey(row))) next.delete(referenceKey(row));
    else next.add(referenceKey(row));
    batch(() => {
      picked.set(next);
      index.set(at);
    });
  }

  function activate(at: number): void {
    batch(() => {
      index.set(at);
      picked.set(new Set());
    });
    activateSelection();
  }

  /** Prepares the citation, hides the panel, then pastes into the captured
   * target. Re-entry is refused until the transaction ends.
   *
   * The service reports a refused target with `pasteFailed(ticket)` before
   * `preparePaste` rejects: that ticket (and the citation it keeps) wins over
   * the rejection, so the manual copy still copies the exact citation. */
  async function activateSelection(): Promise<void> {
    if (pasting) return;
    const refs = pasteSelection();
    const ctx = context();
    const t = target();
    if (refs.length === 0 || !ctx || !t || loading()) return;
    pasting = true;
    failureTicket = null;
    let hidden = false;
    try {
      const plan = await session.preparePaste(
        { context: ctx, targets: refs.map((r) => ({ libraryUid: r.libraryUid, recordUid: r.recordUid })) },
        t.ticket,
      );
      committing = true;
      hidden = true;
      hidePanel();
      const outcome = await desktop.commitPaste(plan.ticket);
      committing = false;
      target.set(null);
      if (outcome.status === "manualCopy") {
        // The citation is kept under the ticket for 引用.
        showFailure(plan.ticket, tr("粘贴失败：{0}", outcome.message ? tr(outcome.message) : tr("缺少辅助功能权限，无法控制其他应用")));
      } else if (outcome.status === "stale") {
        // The ticket expired with its selection: nothing valid to copy from it.
        error.set({ message: tr("粘贴失败：{0}", outcome.message ? tr(outcome.message) : tr("StaleQuery")), ticket: null });
        showPanel();
      }
    } catch (e) {
      committing = false;
      const message = tr("粘贴失败：{0}", errorText(e));
      const ticket = failureTicket ?? untracked(error)?.ticket ?? null;
      error.set({ message: ticket && untracked(error)?.ticket === ticket ? untracked(error)!.message : message, ticket });
      if (hidden) showPanel();
    } finally {
      pasting = false;
    }
  }

  /** A failed paste: show the panel again with the error and the manual copy. */
  function showFailure(ticket: string, message: string): void {
    if (pasting) failureTicket = ticket;
    error.set({ message, ticket });
    showPanel();
  }

  async function copyFallback(): Promise<void> {
    const failure = error();
    try {
      if (failure?.ticket) await desktop.copyFallback(failure.ticket);
      else {
        const refs = pasteSelection();
        if (refs.length > 0) await desktop.copyText(refs.map((r) => r.citeKey).join(","));
      }
      error.set(null);
    } catch (e) {
      error.set({ message: tr("复制失败：{0}", errorText(e)), ticket: failure?.ticket ?? null });
    }
  }

  return {
    // Reactive reads.
    target: () => target(),
    library: () => library(),
    input: () => (selectingLibrary() ? libraryQuery() : input()),
    capsules: () => capsules(),
    selectingLibrary: () => selectingLibrary(),
    libraries: () => filteredLibraries(),
    libraryIndex: () => libraryIndex(),
    results: () => results(),
    analysis: () => analysis(),
    loading: () => loading(),
    index: () => index(),
    isPicked: (r: ReferenceDto, at: number) => (picked().size > 0 ? picked().has(referenceKey(r)) : index() === at),
    pickedCount: () => pasteSelection().length,
    error: () => error(),
    candidate: () => candidate(),
    candidates: () => candidates(),
    focusRequest: () => focusRequest(),
    showsList,
    showsBottomBar,
    height: () => height(),
    // Changes.
    begin,
    end,
    setInput,
    setComposing,
    setCandidateFilter,
    confirmCandidate,
    cancelCandidates,
    removeCapsule,
    deletePreviousCapsule,
    startSelectingLibrary,
    chooseLibrary,
    move,
    submit,
    escape,
    tab,
    pick,
    activate,
    activateSelection: () => void activateSelection(),
    showFailure,
    copyFallback: () => void copyFallback(),
    dismissError: () => error.set(null),
  };
}

export type Helper = ReturnType<typeof createHelper>;
