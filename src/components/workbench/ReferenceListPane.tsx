import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from "solid-js";
import { useApp } from "../../context/AppContext.tsx";
import { searchByField, searchReferences } from "../../tauri.ts";
import type { FilterField, FilterType, Reference } from "../../types.ts";
import IconButton from "../ui/IconButton.tsx";
import SvgIcon from "../ui/SvgIcon.tsx";
import { getReferenceTypeKey } from "../reference/semantic.ts";
import ReferenceRow from "./ReferenceRow.tsx";

const FILTER_TYPES: FilterType[] = [
  "All",
  "Article",
  "Book",
  "Thesis",
  "TechReport",
  "Misc",
  "Booklet",
  "InBook",
  "InCollection",
  "InProceedings",
];

const FILTER_FIELDS: FilterField[] = ["All", "Author", "Title", "Journal", "Year"];

function matchesFilterType(reference: Reference, filterType: FilterType) {
  if (filterType === "All") return true;
  if (filterType === "Thesis") {
    return getReferenceTypeKey(reference.type_) === "thesis";
  }
  return reference.type_ === filterType;
}

export default function ReferenceListPane() {
  const {
    currentBibName,
    currentReferences,
    selectedReference,
    setSelectedReference,
    leftPaneOpen,
    toggleLeftPane,
    rightPaneOpen,
    toggleRightPane,
  } = useApp();
  const [query, setQuery] = createSignal("");
  const [filterType, setFilterType] = createSignal<FilterType>("All");
  const [filterField, setFilterField] = createSignal<FilterField>("All");
  const [visibleReferences, setVisibleReferences] = createSignal<Reference[]>([]);
  const [searching, setSearching] = createSignal(false);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);

  const references = createMemo(() => currentReferences() ?? []);
  const typeFilteredReferences = createMemo(() =>
    references().filter((reference) => matchesFilterType(reference, filterType())),
  );

  createEffect(() => {
    const filtered = typeFilteredReferences();
    const trimmedQuery = query().trim();
    const field = filterField();
    let cancelled = false;

    const run = async () => {
      setSearching(trimmedQuery !== "");
      setErrorMessage(null);
      try {
        if (trimmedQuery === "") {
          setVisibleReferences(filtered);
          return;
        }
        const results =
          field === "All"
            ? await searchReferences(filtered, trimmedQuery)
            : await searchByField(filtered, trimmedQuery, field);
        if (!cancelled) {
          setVisibleReferences(results);
        }
      } catch (e) {
        if (!cancelled) {
          setVisibleReferences(filtered);
          setErrorMessage(`Search failed: ${e}`);
        }
      } finally {
        if (!cancelled) {
          setSearching(false);
        }
      }
    };

    void run();

    onCleanup(() => {
      cancelled = true;
    });
  });

  return (
    <main class="flex min-w-0 flex-1 flex-col bg-base-100">
      <div class="flex h-14 shrink-0 items-center gap-2 border-b border-base-300 px-3">
        <IconButton
          icon={leftPaneOpen() ? "panelLeftClose" : "panelLeftOpen"}
          label={leftPaneOpen() ? "Hide libraries" : "Show libraries"}
          size="sm"
          onClick={toggleLeftPane}
        />
        <div class="min-w-0 flex-1">
          <h1 class="truncate text-sm font-semibold">{currentBibName() ?? "Workbench"}</h1>
          <p class="text-xs text-base-content/55">
            {references().length} references
          </p>
        </div>
        <IconButton
          icon={rightPaneOpen() ? "panelRightClose" : "panelRightOpen"}
          label={rightPaneOpen() ? "Hide details" : "Show details"}
          size="sm"
          onClick={toggleRightPane}
        />
      </div>

      <div class="grid shrink-0 grid-cols-[minmax(8rem,11rem)_minmax(8rem,11rem)_minmax(12rem,1fr)] gap-2 border-b border-base-300 bg-base-200/40 p-3">
        <label class="select select-sm select-bordered flex items-center gap-2">
          <SvgIcon name="tag" class="h-3.5 w-3.5 shrink-0 opacity-60" aria-hidden />
          <select
            class="min-w-0 flex-1 bg-transparent text-sm outline-none"
            value={filterType()}
            onChange={(event) => setFilterType(event.currentTarget.value as FilterType)}
          >
            <For each={FILTER_TYPES}>
              {(type) => <option value={type}>{type}</option>}
            </For>
          </select>
        </label>

        <label class="select select-sm select-bordered flex items-center gap-2">
          <SvgIcon name="fileText" class="h-3.5 w-3.5 shrink-0 opacity-60" aria-hidden />
          <select
            class="min-w-0 flex-1 bg-transparent text-sm outline-none"
            value={filterField()}
            onChange={(event) => setFilterField(event.currentTarget.value as FilterField)}
          >
            <For each={FILTER_FIELDS}>
              {(field) => <option value={field}>{field}</option>}
            </For>
          </select>
        </label>

        <label class="input input-sm input-bordered flex min-w-0 items-center gap-2">
          <SvgIcon name="search" class="h-3.5 w-3.5 shrink-0 opacity-60" aria-hidden />
          <input
            type="search"
            class="min-w-0 flex-1"
            placeholder="Search references"
            value={query()}
            onInput={(event) => setQuery(event.currentTarget.value)}
          />
          <Show when={searching()}>
            <span class="loading loading-spinner loading-xs" />
          </Show>
        </label>
      </div>

      <Show when={errorMessage()}>
        <div class="border-b border-error/25 bg-error/10 px-4 py-2 text-xs text-error">
          {errorMessage()}
        </div>
      </Show>

      <div class="min-h-0 flex-1 overflow-y-auto">
        <Show
          when={visibleReferences().length > 0}
          fallback={
            <div class="flex h-full items-center justify-center px-6 text-sm text-base-content/55">
              No references to display.
            </div>
          }
        >
          <For each={visibleReferences()}>
            {(reference) => (
              <ReferenceRow
                reference={reference}
                selected={selectedReference()?.cite_key === reference.cite_key}
                onSelect={() => setSelectedReference(reference)}
              />
            )}
          </For>
        </Show>
      </div>
    </main>
  );
}
