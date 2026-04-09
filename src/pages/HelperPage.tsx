import { createEffect, createSignal, For, onCleanup, onMount, Show, Switch, Match } from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  copyToClipboard,
  getHelperBib,
  loadBibliography,
  loadSettings,
  pasteToApp,
  resizeHelperWindow,
  searchReferences,
  setHelperBib,
} from "@/tauri.ts";
import type { EntryType, Reference } from "@/types.ts";
import ChunksComp from "@/components/ChunksComp.tsx";
import { TRANSPARENT_LOGO } from "@/constants/icons.ts";

const MIN_HEIGHT = 70;
const MAX_HEIGHT = 2000;

interface BibInfo {
  name: string;
  path: string;
  updatedAt: string;
  description?: string;
  exists: boolean;
}

function HelperPage() {
  const [query, setQuery] = createSignal("");
  const [results, setResults] = createSignal<Reference[]>([]);
  const [selectedIndex, setSelectedIndex] = createSignal<number | null>(0);
  const [isSelectingBib, setIsSelectingBib] = createSignal(true);
  const [bibSelectedIndex, setBibSelectedIndex] = createSignal<number | null>(0);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const [bibs, setBibs] = createSignal<BibInfo[]>([]);
  const [currentBib, setCurrentBib] = createSignal<
    {
      name: string;
      refs: Reference[];
    } | null
  >(null);

  let inputRef: HTMLInputElement | undefined;
  let containerRef: HTMLDivElement | undefined;
  let searchTimeoutRef: ReturnType<typeof setTimeout> | null = null;
  let lastHeight = MIN_HEIGHT;

  const updateWindowHeight = async () => {
    await new Promise((resolve) => setTimeout(resolve, 10));

    if (!containerRef) return;

    let totalHeight = MIN_HEIGHT;
    const scrollable = containerRef.querySelector('.overflow-y-auto');
    if (scrollable) {
      totalHeight = 64 + scrollable.scrollHeight + 20;
    } else {
      totalHeight = containerRef.scrollHeight;
    }

    const screenMax = typeof window !== "undefined" ? window.screen.availHeight * 0.9 : MAX_HEIGHT;
    const effectiveMax = Math.min(MAX_HEIGHT, screenMax);

    const finalHeight = Math.min(
      Math.max(totalHeight, MIN_HEIGHT),
      effectiveMax,
    );

    if (Math.abs(finalHeight - lastHeight) > 2) {
      lastHeight = finalHeight;
      try {
        await resizeHelperWindow(finalHeight);
      } catch (e) {
        console.error("Failed to resize window:", e);
      }
    }
  };

  const doSearch = (q: string) => {
    if (searchTimeoutRef) clearTimeout(searchTimeoutRef);

    const bib = currentBib();
    if (!bib || q.trim() === "") {
      setResults([]);
      setSelectedIndex(null);
      setTimeout(updateWindowHeight, 50);
      return;
    }

    searchTimeoutRef = setTimeout(async () => {
      try {
        const r = await searchReferences(bib.refs, q);
        setResults(r);
        setSelectedIndex(r.length > 0 ? 0 : null);
        setTimeout(updateWindowHeight, 50);
      } catch (e) {
        console.error("Search failed:", e);
        setResults([]);
      }
    }, 100);
  };

  const handleInput = (e: Event & { currentTarget: HTMLInputElement }) => {
    const value = e.currentTarget.value;
    setQuery(value);
    if (!isSelectingBib()) {
      doSearch(value);
    }
  };

  const selectBib = async (name: string, path: string) => {
    try {
      const refs = await loadBibliography(path);
      setCurrentBib({ name, refs });
      setIsSelectingBib(false);
      setQuery("");
      setResults([]);
      setSelectedIndex(null);
      setErrorMessage(null);

      await setHelperBib(name, path);
      setTimeout(updateWindowHeight, 100);
      setTimeout(() => inputRef?.focus(), 150);
    } catch (e) {
      setErrorMessage(`加载失败: ${e}`);
    }
  };

  const scrollToItem = (index: number) => {
    const item = document.querySelector(
      `[data-item-index="${index}"]`,
    ) as HTMLElement;
    if (!item) return;

    const container = item.closest(".overflow-y-auto") as HTMLElement;
    if (!container) return;

    const containerRect = container.getBoundingClientRect();
    const itemRect = item.getBoundingClientRect();

    const itemTop = itemRect.top - containerRect.top + container.scrollTop;
    const itemHeight = itemRect.height;
    const containerHeight = containerRect.height;

    let targetScrollTop;
    if (index < 3) {
      targetScrollTop = Math.max(0, itemTop - 8);
    } else {
      targetScrollTop = itemTop - containerHeight / 2 + itemHeight / 2;
    }

    container.scrollTo({
      top: targetScrollTop,
      behavior: "smooth",
    });
  };

  const handleSelect = async (ref: Reference) => {
    try {
      await copyToClipboard(ref.cite_key);
      await pasteToApp(ref.cite_key);
      const window = getCurrentWindow();
      await window.hide();
    } catch (e) {
      console.error("Paste failed:", e);
    }
  };

  const handleKeyDown = async (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      try {
        const win = getCurrentWindow();
        await win.hide();
      } catch (err) {
        console.error("Failed to hide window:", err);
      }
      return;
    }

    if (isSelectingBib()) {
      const len = bibs().length;
      if (len === 0) return;

      if (e.key === "ArrowDown") {
        e.preventDefault();
        const idx = bibSelectedIndex();
        if (idx === null) {
          setBibSelectedIndex(0);
        } else {
          setBibSelectedIndex(Math.min(idx + 1, len - 1));
        }
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        const idx = bibSelectedIndex();
        if (idx === null) {
          setBibSelectedIndex(len - 1);
        } else {
          setBibSelectedIndex(Math.max(idx - 1, 0));
        }
      } else if (e.key === "Enter") {
        e.preventDefault();
        const idx = bibSelectedIndex();
        if (idx !== null && bibs()[idx]) {
          await selectBib(bibs()[idx].name, bibs()[idx].path);
        }
      }
    } else {
      const len = results().length;
      if (len === 0) return;

      if (e.key === "ArrowDown") {
        e.preventDefault();
        const idx = selectedIndex();
        const newIndex = idx === null ? 0 : Math.min(idx + 1, len - 1);
        setSelectedIndex(newIndex);
        scrollToItem(newIndex);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        const idx = selectedIndex();
        const newIndex = idx === null ? len - 1 : Math.max(idx - 1, 0);
        setSelectedIndex(newIndex);
        scrollToItem(newIndex);
      } else if (e.key === "Enter") {
        e.preventDefault();
        const idx = selectedIndex();
        if (idx !== null && results()[idx]) {
          await handleSelect(results()[idx]);
        }
      }
    }
  };

  const handleBibSelectClick = () => {
    setIsSelectingBib(true);
    setQuery("");
    setResults([]);
    setSelectedIndex(null);
    setErrorMessage(null);
    setTimeout(() => {
      inputRef?.focus();
      updateWindowHeight();
    }, 100);
  };

  // Load bibs on mount
  onMount(async () => {
    inputRef?.focus();

    try {
      const settings = await loadSettings();
      const bibEntries = Object.entries(settings.bibliographies);

      const loadedBibs = bibEntries
        .map(([name, info]: [string, unknown]) => {
          const bibInfo = info as {
            path: string;
            updated_at: string;
            description?: string;
          };
          return {
            name,
            path: bibInfo.path,
            updatedAt: bibInfo.updated_at,
            description: bibInfo.description,
            exists: true,
          };
        })
        .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));

      setBibs(loadedBibs);

      await new Promise((resolve) => setTimeout(resolve, 0));
      updateWindowHeight();

      const storedBib = await getHelperBib();
      if (
        storedBib &&
        loadedBibs.some((b) =>
          b.name === storedBib[0] && b.path === storedBib[1]
        )
      ) {
        await selectBib(storedBib[0], storedBib[1]);
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    }

    const focusInput = () => inputRef?.focus();
    focusInput();
    setTimeout(focusInput, 50);
    setTimeout(focusInput, 150);
    setTimeout(focusInput, 300);
  });

  // Transparent background
  onMount(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
  });

  onCleanup(() => {
    document.documentElement.style.background = "";
    document.body.style.background = "";
    if (searchTimeoutRef) clearTimeout(searchTimeoutRef);
  });

  // Resize on content changes
  createEffect(() => {
    // Track reactive dependencies
    results().length;
    bibs().length;
    isSelectingBib();
    setTimeout(updateWindowHeight, 100);
  });

  createEffect(() => {
    errorMessage();
    setTimeout(updateWindowHeight, 50);
  });

  const getEntryTypeLabel = (type_: EntryType): string => {
    if (typeof type_ === "string") {
      if (type_ === "MastersThesis") return "Master Thesis";
      if (type_ === "PhdThesis") return "PhD Thesis";
      return type_;
    }
    if (typeof type_ === "object" && "Unknown" in type_) return type_.Unknown;
    return "Unknown";
  };

  const getBorderColor = (type_: EntryType): string => {
    const typeStr = typeof type_ === "string" ? type_ : "Unknown";
    switch (typeStr) {
      case "Article":
        return "border-l-info";
      case "Book":
        return "border-l-success";
      case "MastersThesis":
      case "PhdThesis":
      case "Thesis":
        return "border-l-secondary";
      case "InProceedings":
        return "border-l-primary";
      case "TechReport":
        return "border-l-warning";
      case "Misc":
        return "border-l-neutral";
      case "Booklet":
        return "border-l-info";
      case "InBook":
        return "border-l-accent";
      case "InCollection":
        return "border-l-secondary";
      default:
        return "border-l-base-content/20";
    }
  };

  const getBadgeColor = (type_: EntryType): string => {
    const typeStr = typeof type_ === "string" ? type_ : "Unknown";
    switch (typeStr) {
      case "Article":
        return "badge-info badge-soft";
      case "Book":
        return "badge-success badge-soft";
      case "MastersThesis":
      case "PhdThesis":
      case "Thesis":
        return "badge-secondary badge-soft";
      case "InProceedings":
        return "badge-primary badge-soft";
      case "TechReport":
        return "badge-warning badge-soft";
      case "Misc":
        return "badge-neutral badge-soft";
      case "Booklet":
        return "badge-info badge-soft";
      case "InBook":
        return "badge-accent badge-soft";
      case "InCollection":
        return "badge-secondary badge-soft";
      default:
        return "badge-ghost";
    }
  };

  return (
    <div
      ref={containerRef}
      class="helper-container flex flex-col w-full h-full bg-base-100/80 backdrop-blur-xl border border-base-content/20 shadow-2xl rounded-xl overflow-hidden"
    >
      <div
        class="relative w-full max-w-full h-16 bg-transparent z-20 border-b border-base-content/10 shrink-0 overflow-hidden box-border"
        data-tauri-drag-region
      >
        <div class="absolute left-4 top-1/2 -translate-y-1/2 text-base-content/40">
          <svg
            class="w-5 h-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
        </div>

        <input
          ref={inputRef}
          value={query()}
          onInput={handleInput}
          onKeyDown={handleKeyDown}
          class="w-full max-w-full h-full pl-12 pr-36 text-lg bg-transparent border-none outline-none ring-0 focus:border-none focus:outline-none focus:ring-0 placeholder:text-base-content/30 text-base-content disabled:cursor-default disabled:opacity-100 overflow-hidden"
          type="text"
          placeholder={isSelectingBib()
            ? "选择文献库..."
            : "搜索文献、作者、标题..."}
          readOnly={isSelectingBib()}
        />

        <div class="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-2">
          <Show
            when={currentBib()}
            fallback={
              <button
                type="button"
                class="badge badge-ghost cursor-pointer hover:bg-base-200"
                onClick={handleBibSelectClick}
              >
                选择文献库
              </button>
            }
          >
            <button
              type="button"
              class="badge badge-primary badge-soft gap-1 cursor-pointer hover:scale-105 transition-transform font-medium"
              onClick={handleBibSelectClick}
            >
              <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
              {currentBib()!.name}
            </button>
          </Show>
          <img
            class="opacity-80 hover:opacity-100 transition-opacity duration-300 w-6 h-6"
            src={TRANSPARENT_LOGO}
            alt="logo"
          />
        </div>
      </div>

      <div class="w-full max-w-full overflow-hidden box-border">
        <Switch>
          {/* Selecting bib - no bibs */}
          <Match when={isSelectingBib() && bibs().length === 0}>
            <div class="shrink-0 px-5 py-10 text-center text-base-content/60 text-sm">
              未找到文献库，请先在主页添加文献库
            </div>
          </Match>

          {/* Selecting bib - has bibs */}
          <Match when={isSelectingBib()}>
            <div class="flex flex-col h-full w-full max-w-full overflow-hidden box-border">
              <div
                class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-2 scroll-smooth w-full max-w-full box-border custom-scrollbar"
              >
                <For each={bibs()}>
                  {(bib, i) => (
                    <button
                      type="button"
                      data-item-index={i()}
                      class={`w-full max-w-full text-left rounded-lg transition-all duration-200 cursor-pointer mx-2 overflow-hidden ${
                        i() === bibSelectedIndex()
                          ? "bg-primary/10 text-primary shadow-sm"
                          : "hover:bg-base-200/50 hover:shadow-sm border border-transparent"
                      }`}
                      onClick={() => selectBib(bib.name, bib.path)}
                      onMouseEnter={() => setBibSelectedIndex(i())}
                    >
                      <div class="p-3">
                        <div class="flex items-center justify-between gap-2">
                          <div class="flex items-center gap-2 flex-1 min-w-0">
                            <Show
                              when={bib.exists}
                              fallback={
                                <div class="badge badge-error badge-xs gap-1 border-none shrink-0">
                                  <div class="w-1.5 h-1.5 rounded-full bg-white">
                                  </div>
                                  Error
                                </div>
                              }
                            >
                              <div class="badge badge-success badge-xs gap-1 border-none shrink-0">
                                <div class="w-1.5 h-1.5 rounded-full bg-white animate-pulse">
                                </div>
                                Ready
                              </div>
                            </Show>
                            <h3 class="font-bold text-base truncate">
                              {bib.name}
                            </h3>
                          </div>
                          <span class="text-xs opacity-60 font-mono shrink-0">
                            {bib.updatedAt}
                          </span>
                        </div>
                        <Show when={bib.description}>
                          <p class="text-sm opacity-80 mt-1">
                            {bib.description}
                          </p>
                        </Show>
                        <p class="text-xs opacity-50 truncate mt-1 font-mono">
                          {bib.path}
                        </p>
                      </div>
                    </button>
                  )}
                </For>
              </div>
              <Show when={errorMessage()}>
                <div class="alert alert-error shadow-lg m-2">
                  <span>{errorMessage()}</span>
                </div>
              </Show>
            </div>
          </Match>

          {/* Search mode - empty query */}
          <Match when={!isSelectingBib() && query().trim() === ""}>
            <div class="flex flex-col items-center justify-center h-32 text-base-content/40 gap-4">
              <svg
                class="w-12 h-12 opacity-50"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="1.5"
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              <span class="text-sm font-medium">开始输入以搜索文献...</span>
            </div>
          </Match>

          {/* Search mode - no results */}
          <Match when={!isSelectingBib() && results().length === 0}>
            <div class="flex flex-col items-center justify-center h-32 text-base-content/40 gap-4">
              <svg
                class="w-12 h-12 opacity-50"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="1.5"
                  d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <span class="text-sm font-medium">未找到匹配的文献</span>
            </div>
          </Match>

          {/* Search mode - has results */}
          <Match when={!isSelectingBib() && results().length > 0}>
            <div
              class="overflow-y-auto overflow-x-hidden p-2 space-y-2 w-full max-w-full box-border custom-scrollbar"
            >
              <For each={results()}>
                {(ref, i) => (
                  <button
                    type="button"
                    data-item-index={i()}
                    class={`w-full max-w-full text-left group relative rounded-lg px-1 transition-all duration-200 cursor-pointer border-l-[3px] mx-2 overflow-hidden box-border ${
                      getBorderColor(ref.type_)
                    } ${
                      i() === selectedIndex()
                        ? "bg-primary/15 shadow-md ring-1 ring-primary/30 scale-[1.01]"
                        : "hover:bg-base-200/50 border-opacity-50 hover:border-opacity-100"
                    }`}
                    onClick={() => handleSelect(ref)}
                    onMouseEnter={() => setSelectedIndex(i())}
                  >
                    <div class="py-3 px-3 w-full max-w-full overflow-hidden box-border">
                      <div class="w-full">
                        <div class="flex justify-between items-center gap-2">
                          <div class="flex items-center gap-2 flex-1 min-w-0">
                            <div
                              class={`badge ${
                                getBadgeColor(ref.type_)
                              } badge-sm font-bold shrink-0`}
                            >
                              {getEntryTypeLabel(ref.type_)}
                            </div>
                            <Show
                              when={ref.title}
                              fallback={
                                <span class="text-gray-900 dark:text-gray-100 font-serif italic truncate">
                                  No title available
                                </span>
                              }
                            >
                              <span class="text-gray-900 dark:text-gray-100 font-serif font-medium truncate">
                                <ChunksComp
                                  chunks={ref.title!}
                                  citeKey={ref.cite_key}
                                />
                              </span>
                            </Show>
                          </div>
                          <div class="flex items-center shrink-0">
                            <div class="text-xs font-mono opacity-50">
                              {ref.cite_key}
                            </div>
                          </div>
                        </div>
                        <div class="mt-1 flex flex-wrap gap-1">
                          <For each={ref.author || []}>
                            {(author, idx) => (
                              <span class="text-xs opacity-70">
                                {author}
                                {idx() < (ref.author?.length || 0) - 1 ? "," : ""}
                              </span>
                            )}
                          </For>
                        </div>
                        <div class="mt-1 flex items-center gap-2 text-xs opacity-60">
                          <Show when={ref.journal}>
                            <span class="italic">{ref.journal}</span>
                          </Show>
                          <Show when={ref.year}>
                            <span>{ref.year}</span>
                          </Show>
                        </div>
                      </div>
                    </div>
                  </button>
                )}
              </For>
            </div>
          </Match>
        </Switch>
      </div>
    </div>
  );
}

export default HelperPage;
