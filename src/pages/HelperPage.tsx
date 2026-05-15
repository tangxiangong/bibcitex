import {
  createEffect,
  createMemo,
  createSignal,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  copyToClipboard,
  getHelperBib,
  hideHelperWindow,
  loadBibliography,
  loadSettings,
  pasteToApp,
  resizeHelperWindow,
  searchReferences,
  setHelperBib,
} from "@/tauri.ts";
import type { Reference } from "@/types.ts";
import ChunksComp from "@/components/ChunksComp.tsx";
import { SvgIcon } from "@/components/ui/SvgIcon";
import {
  getReferenceTypeKey,
  getReferenceVenue,
  METADATA_STYLES,
  TYPE_STYLES,
} from "@/components/reference/semantic";

const MIN_HEIGHT = 70;
const HEADER_HEIGHT = 64;
const MAX_CONTENT_HEIGHT = 480;
const BIB_ROW_HEIGHT = 86;
const RESULT_ROW_HEIGHT = 96;
const VIRTUAL_OVERSCAN = 5;

interface BibInfo {
  name: string;
  path: string;
  updatedAt: string;
  description?: string;
  exists: boolean;
}

interface VirtualItem<T> {
  item: T;
  index: number;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function getViewportHeight(count: number, rowHeight: number): number {
  return Math.min(count * rowHeight, MAX_CONTENT_HEIGHT);
}

function getVirtualItems<T>(
  items: T[],
  scrollTop: number,
  viewportHeight: number,
  rowHeight: number,
): VirtualItem<T>[] {
  if (items.length === 0 || viewportHeight === 0) return [];

  const start = clamp(
    Math.floor(scrollTop / rowHeight) - VIRTUAL_OVERSCAN,
    0,
    items.length - 1,
  );
  const end = clamp(
    Math.ceil((scrollTop + viewportHeight) / rowHeight) + VIRTUAL_OVERSCAN,
    start + 1,
    items.length,
  );

  return items.slice(start, end).map((item, offset) => ({
    item,
    index: start + offset,
  }));
}

function normalizeText(value: string): string {
  return value.trim().toLocaleLowerCase();
}

function bibliographyMatchesQuery(bib: BibInfo, query: string): boolean {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) return true;

  return [bib.name, bib.path, bib.updatedAt, bib.description ?? ""].some((value) =>
    normalizeText(value).includes(normalizedQuery),
  );
}

function HelperPage() {
  const [query, setQuery] = createSignal("");
  const [results, setResults] = createSignal<Reference[]>([]);
  const [selectedIndex, setSelectedIndex] = createSignal<number | null>(0);
  const [isSelectingBib, setIsSelectingBib] = createSignal(true);
  const [bibSelectedIndex, setBibSelectedIndex] = createSignal<number | null>(0);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);
  const [failedPasteKey, setFailedPasteKey] = createSignal<string | null>(null);
  const [bibs, setBibs] = createSignal<BibInfo[]>([]);
  const [isLoadingSettings, setIsLoadingSettings] = createSignal(true);
  const [loadingBibPath, setLoadingBibPath] = createSignal<string | null>(null);
  const [bibScrollTop, setBibScrollTop] = createSignal(0);
  const [resultScrollTop, setResultScrollTop] = createSignal(0);
  const [currentBib, setCurrentBib] = createSignal<
    { name: string; path: string; refs: Reference[] } | null
  >(null);

  let inputRef: HTMLInputElement | undefined;
  let bibListRef: HTMLDivElement | undefined;
  let resultListRef: HTMLDivElement | undefined;
  let searchTimeoutRef: ReturnType<typeof setTimeout> | null = null;
  let resizeRafId: number | null = null;
  let lastHeight: number | null = null;

  const filteredBibs = createMemo(() =>
    bibs().filter((bib) => bibliographyMatchesQuery(bib, query())),
  );

  const bibViewportHeight = createMemo(() =>
    getViewportHeight(filteredBibs().length, BIB_ROW_HEIGHT),
  );
  const resultViewportHeight = createMemo(() =>
    getViewportHeight(results().length, RESULT_ROW_HEIGHT),
  );
  const visibleBibs = createMemo(() =>
    getVirtualItems(
      filteredBibs(),
      bibScrollTop(),
      bibViewportHeight(),
      BIB_ROW_HEIGHT,
    ),
  );
  const visibleResults = createMemo(() =>
    getVirtualItems(
      results(),
      resultScrollTop(),
      resultViewportHeight(),
      RESULT_ROW_HEIGHT,
    ),
  );

  const measureContentHeight = (): number => {
    const errorHeight = errorMessage() ? 48 : 0;

    if (isSelectingBib()) {
      if (isLoadingSettings() || filteredBibs().length === 0) {
        return HEADER_HEIGHT + 128 + errorHeight;
      }
      return HEADER_HEIGHT + bibViewportHeight() + errorHeight;
    }

    if (query().trim() === "") {
      return HEADER_HEIGHT + errorHeight;
    }

    if (results().length === 0) {
      return HEADER_HEIGHT + 128 + errorHeight;
    }

    return HEADER_HEIGHT + resultViewportHeight() + errorHeight;
  };

  const doResize = async () => {
    const finalHeight = Math.max(measureContentHeight(), MIN_HEIGHT);
    if (finalHeight === lastHeight) return;

    lastHeight = finalHeight;
    try {
      await resizeHelperWindow(finalHeight);
    } catch (e) {
      console.error("Failed to resize window:", e);
    }
  };

  const updateWindowSize = () => {
    if (resizeRafId !== null) cancelAnimationFrame(resizeRafId);
    resizeRafId = requestAnimationFrame(() => {
      resizeRafId = requestAnimationFrame(() => {
        resizeRafId = null;
        doResize();
      });
    });
  };

  const focusSearchInput = () => {
    inputRef?.focus();
    inputRef?.select();
  };

  const syncSpotlightSurface = () => {
    focusSearchInput();
    updateWindowSize();
  };

  const handleHelperOpened = () => {
    setErrorMessage(null);
    setFailedPasteKey(null);

    if (currentBib() && !isSelectingBib()) {
      setQuery("");
      setResults([]);
      setSelectedIndex(null);
      setResultScrollTop(0);
      if (resultListRef) resultListRef.scrollTop = 0;
    } else if (isSelectingBib()) {
      setQuery("");
      setBibSelectedIndex(filteredBibs().length > 0 ? 0 : null);
      setBibScrollTop(0);
      if (bibListRef) bibListRef.scrollTop = 0;
    }

    syncSpotlightSurface();
  };

  const doSearch = (value: string) => {
    if (searchTimeoutRef) clearTimeout(searchTimeoutRef);

    const bib = currentBib();
    if (!bib || value.trim() === "") {
      setResults([]);
      setSelectedIndex(null);
      setResultScrollTop(0);
      updateWindowSize();
      return;
    }

    searchTimeoutRef = setTimeout(async () => {
      try {
        const found = await searchReferences(bib.refs, value);
        setResults(found);
        setSelectedIndex(found.length > 0 ? 0 : null);
        setResultScrollTop(0);
        if (resultListRef) resultListRef.scrollTop = 0;
        updateWindowSize();
      } catch (e) {
        console.error("Search failed:", e);
        setResults([]);
        setSelectedIndex(null);
        setResultScrollTop(0);
        setErrorMessage(`搜索失败: ${e}`);
        updateWindowSize();
      }
    }, 100);
  };

  const handleInput = (event: Event & { currentTarget: HTMLInputElement }) => {
    const value = event.currentTarget.value;
    setQuery(value);
    setErrorMessage(null);
    setFailedPasteKey(null);
    if (isSelectingBib()) {
      setBibSelectedIndex(0);
      setBibScrollTop(0);
      if (bibListRef) bibListRef.scrollTop = 0;
      updateWindowSize();
    } else {
      doSearch(value);
    }
  };

  const selectBib = async (name: string, path: string) => {
    try {
      setLoadingBibPath(path);
      setErrorMessage(null);
      const refs = await loadBibliography(path);
      setCurrentBib({ name, path, refs });
      setIsSelectingBib(false);
      setQuery("");
      setResults([]);
      setSelectedIndex(null);
      setErrorMessage(null);
      setFailedPasteKey(null);
      setResultScrollTop(0);
      await setHelperBib(name, path);
      updateWindowSize();
      setTimeout(focusSearchInput, 50);
    } catch (e) {
      setErrorMessage(`加载失败: ${e}`);
      updateWindowSize();
    } finally {
      setLoadingBibPath(null);
    }
  };

  const scrollVirtualItemIntoView = (
    container: HTMLDivElement | undefined,
    index: number,
    rowHeight: number,
  ) => {
    if (!container) return;
    const itemTop = index * rowHeight;
    const itemBottom = itemTop + rowHeight;
    const viewTop = container.scrollTop;
    const viewBottom = viewTop + container.clientHeight;

    if (itemTop < viewTop) {
      container.scrollTo({ top: itemTop, behavior: "smooth" });
    } else if (itemBottom > viewBottom) {
      container.scrollTo({
        top: itemBottom - container.clientHeight,
        behavior: "smooth",
      });
    }
  };

  const handleSelect = async (ref: Reference) => {
    try {
      await copyToClipboard(ref.cite_key);
      await pasteToApp(ref.cite_key);
      await hideHelperWindow();
    } catch (e) {
      console.error("Paste failed:", e);
      setFailedPasteKey(ref.cite_key);
      setErrorMessage(`粘贴失败，已保留 cite key，可手动复制: ${ref.cite_key}`);
      updateWindowSize();
    }
  };

  const handleKeyDown = async (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      try {
        await hideHelperWindow();
      } catch (e) {
        console.error("Failed to hide window:", e);
      }
      return;
    }

    if (isSelectingBib()) {
      const list = filteredBibs();
      const len = list.length;
      if (len === 0) return;

      if (event.key === "ArrowDown") {
        event.preventDefault();
        const index = bibSelectedIndex();
        const nextIndex = index === null ? 0 : (index + 1) % len;
        setBibSelectedIndex(nextIndex);
        scrollVirtualItemIntoView(bibListRef, nextIndex, BIB_ROW_HEIGHT);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        const index = bibSelectedIndex();
        const nextIndex = index === null ? len - 1 : (index - 1 + len) % len;
        setBibSelectedIndex(nextIndex);
        scrollVirtualItemIntoView(bibListRef, nextIndex, BIB_ROW_HEIGHT);
      } else if (event.key === "Enter") {
        event.preventDefault();
        const index = bibSelectedIndex();
        if (index !== null && list[index]) {
          await selectBib(list[index].name, list[index].path);
        }
      }
      return;
    }

    const len = results().length;
    if (len === 0) return;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      const index = selectedIndex();
      const nextIndex = index === null ? 0 : (index + 1) % len;
      setSelectedIndex(nextIndex);
      scrollVirtualItemIntoView(resultListRef, nextIndex, RESULT_ROW_HEIGHT);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      const index = selectedIndex();
      const nextIndex = index === null ? len - 1 : (index - 1 + len) % len;
      setSelectedIndex(nextIndex);
      scrollVirtualItemIntoView(resultListRef, nextIndex, RESULT_ROW_HEIGHT);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const index = selectedIndex();
      if (index !== null && results()[index]) await handleSelect(results()[index]);
    }
  };

  const handleBibSelectClick = () => {
    setIsSelectingBib(true);
    setQuery("");
    setResults([]);
    setSelectedIndex(null);
    setErrorMessage(null);
    setFailedPasteKey(null);
    setBibSelectedIndex(0);
    setBibScrollTop(0);
    if (bibListRef) bibListRef.scrollTop = 0;
    setTimeout(focusSearchInput, 50);
    updateWindowSize();
  };

  onMount(async () => {
    focusSearchInput();
    try {
      setIsLoadingSettings(true);
      const settings = await loadSettings();
      const loadedBibs = Object.entries(settings.bibliographies)
        .map(([name, info]) => ({
          name,
          path: info.path,
          updatedAt: info.updated_at,
          description: info.description,
          exists: true,
        }))
        .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));

      setBibs(loadedBibs);
      setBibSelectedIndex(loadedBibs.length > 0 ? 0 : null);
      updateWindowSize();

      const storedBib = await getHelperBib();
      if (
        storedBib &&
        loadedBibs.some((bib) => bib.name === storedBib[0] && bib.path === storedBib[1])
      ) {
        await selectBib(storedBib[0], storedBib[1]);
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
      setErrorMessage(`加载文献库失败: ${e}`);
    } finally {
      setIsLoadingSettings(false);
      updateWindowSize();
    }

    syncSpotlightSurface();
    setTimeout(syncSpotlightSurface, 50);
    setTimeout(syncSpotlightSurface, 150);
    setTimeout(syncSpotlightSurface, 300);
  });

  onMount(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
  });

  onMount(() => {
    if (!window.__TAURI__) return;

    const unlistenFns: Array<() => void> = [];
    let appWindow: ReturnType<typeof getCurrentWindow>;

    try {
      appWindow = getCurrentWindow();
    } catch (e) {
      console.error("Failed to get helper window:", e);
      return;
    }

    appWindow
      .listen("helper-opened", handleHelperOpened)
      .then((unlisten) => unlistenFns.push(unlisten))
      .catch((e) => console.error("Failed to listen helper-opened:", e));

    appWindow
      .onFocusChanged(({ payload: focused }) => {
        if (focused) syncSpotlightSurface();
      })
      .then((unlisten) => unlistenFns.push(unlisten))
      .catch((e) => console.error("Failed to listen helper focus:", e));

    onCleanup(() => {
      for (const unlisten of unlistenFns) unlisten();
    });
  });

  onCleanup(() => {
    document.documentElement.style.background = "";
    document.body.style.background = "";
    if (searchTimeoutRef) clearTimeout(searchTimeoutRef);
    if (resizeRafId !== null) cancelAnimationFrame(resizeRafId);
  });

  createEffect(() => {
    results().length;
    filteredBibs().length;
    isSelectingBib();
    errorMessage();
    failedPasteKey();
    bibScrollTop();
    resultScrollTop();
    updateWindowSize();
  });

  createEffect(() => {
    if (!isSelectingBib()) return;
    const len = filteredBibs().length;
    const currentIndex = bibSelectedIndex();
    if (len === 0 && currentIndex !== null) {
      setBibSelectedIndex(null);
    } else if (len > 0 && (currentIndex === null || currentIndex >= len)) {
      setBibSelectedIndex(0);
    }
  });

  createEffect(() => {
    if (isSelectingBib()) return;
    const len = results().length;
    const currentIndex = selectedIndex();
    if (len === 0 && currentIndex !== null) {
      setSelectedIndex(null);
    } else if (len > 0 && (currentIndex === null || currentIndex >= len)) {
      setSelectedIndex(0);
    }
  });

  const renderError = () => (
    <Show when={errorMessage()}>
      <div class="m-2 rounded-box border border-error/30 bg-error/10 px-3 py-2 text-sm text-error">
        <div class="flex items-center gap-2">
          <SvgIcon name="alert" size={16} aria-hidden />
          <span class="min-w-0 flex-1 truncate">{errorMessage()}</span>
          <Show when={failedPasteKey()}>
            <button
              type="button"
              class="btn btn-primary btn-xs"
              onClick={() => copyToClipboard(failedPasteKey()!)}
            >
              复制
            </button>
          </Show>
        </div>
      </div>
    </Show>
  );

  return (
    <div
      class="helper-container flex h-full w-full flex-col overflow-hidden rounded-box border border-base-300 bg-base-100/95 shadow-xl backdrop-blur-xl"
    >
      <div
        class="relative h-16 shrink-0 border-b border-base-300"
        data-tauri-drag-region
      >
        <div class="absolute left-4 top-1/2 -translate-y-1/2 text-base-content/50">
          <SvgIcon name="search" size={20} aria-hidden />
        </div>

        <input
          ref={inputRef}
          value={query()}
          onInput={handleInput}
          onKeyDown={handleKeyDown}
          class="h-full w-full border-none bg-transparent pl-12 pr-40 text-lg text-base-content outline-none placeholder:text-base-content/35 focus:outline-none focus:ring-0 disabled:cursor-default disabled:opacity-100"
          type="text"
          placeholder={isSelectingBib()
            ? "搜索或选择文献库"
            : "搜索文献、作者、标题"}
        />

        <div class="absolute right-3 top-1/2 flex -translate-y-1/2 items-center gap-2">
          <Show
            when={currentBib()}
            fallback={
              <button
                type="button"
                class="badge bg-base-200 text-base-content border-base-300 cursor-pointer"
                onClick={handleBibSelectClick}
              >
                选择文献库
              </button>
            }
          >
            <button
              type="button"
              class="badge bg-primary/10 text-base-content border-primary/30 cursor-pointer gap-1 max-w-56"
              onClick={handleBibSelectClick}
            >
              <SvgIcon name="library" size={12} aria-hidden />
              <span class="truncate">{currentBib()!.name}</span>
            </button>
          </Show>
        </div>
      </div>

      <div class="w-full max-w-full overflow-hidden">
        <Switch>
          <Match when={isSelectingBib() && isLoadingSettings()}>
            <div class="flex h-32 flex-col items-center justify-center gap-3 text-base-content/45">
              <span class="loading loading-spinner loading-sm" />
              <span class="text-sm font-medium">正在读取文献库</span>
            </div>
            {renderError()}
          </Match>

          <Match when={isSelectingBib() && filteredBibs().length === 0}>
            <div class="flex h-32 flex-col items-center justify-center gap-3 text-base-content/45">
              <SvgIcon name="library" size={40} aria-hidden />
              <span class="text-sm font-medium">
                {bibs().length === 0
                  ? "未找到文献库，请先在主页添加文献库"
                  : "没有匹配的文献库"}
              </span>
            </div>
            {renderError()}
          </Match>

          <Match when={isSelectingBib()}>
            <div class="flex h-full w-full flex-col overflow-hidden">
              <div
                ref={bibListRef}
                class="flex-1 overflow-y-auto overflow-x-hidden"
                style={{ height: `${bibViewportHeight()}px` }}
                onScroll={(event) => setBibScrollTop(event.currentTarget.scrollTop)}
              >
                <div
                  class="relative"
                  style={{ height: `${filteredBibs().length * BIB_ROW_HEIGHT}px` }}
                >
                  <For each={visibleBibs()}>
                    {({ item: bib, index }) => (
                    <button
                      type="button"
                      data-item-index={index}
                      class={`absolute left-2 right-2 rounded-field border px-3 py-2 text-left transition-colors ${
                        index === bibSelectedIndex()
                          ? "border-primary/40 bg-primary/10"
                          : "border-transparent hover:border-base-300 hover:bg-base-200"
                      }`}
                      style={{
                        top: `${index * BIB_ROW_HEIGHT + 4}px`,
                        height: `${BIB_ROW_HEIGHT - 8}px`,
                      }}
                      onClick={() => selectBib(bib.name, bib.path)}
                      onMouseEnter={() => setBibSelectedIndex(index)}
                      aria-selected={index === bibSelectedIndex()}
                    >
                      <div class="flex items-center justify-between gap-2">
                        <div class="flex min-w-0 items-center gap-2">
                          <span
                            class={`badge badge-xs ${
                              currentBib()?.path === bib.path
                                ? "bg-primary/10 text-base-content border-primary/30"
                                : "bg-success/10 text-base-content border-success/30"
                            }`}
                          >
                            {currentBib()?.path === bib.path ? "当前" : "可用"}
                          </span>
                          <span class="truncate text-sm font-semibold">
                            {bib.name}
                          </span>
                        </div>
                        <span class="shrink-0 text-xs font-mono text-base-content/50">
                          {bib.updatedAt}
                        </span>
                      </div>
                      <Show when={bib.description}>
                        <p class="mt-1 truncate text-xs text-base-content/70">
                          {bib.description}
                        </p>
                      </Show>
                      <p class="mt-1 truncate font-mono text-xs text-base-content/45">
                        {loadingBibPath() === bib.path ? "正在加载..." : bib.path}
                      </p>
                    </button>
                    )}
                  </For>
                </div>
              </div>
              {renderError()}
            </div>
          </Match>

          <Match when={!isSelectingBib() && query().trim() === ""}>
            {renderError()}
          </Match>

          <Match when={!isSelectingBib() && results().length === 0}>
            <div class="flex h-32 flex-col items-center justify-center gap-3 text-base-content/45">
              <SvgIcon name="alert" size={42} aria-hidden />
              <span class="text-sm font-medium">未找到匹配的文献</span>
            </div>
            {renderError()}
          </Match>

          <Match when={!isSelectingBib() && results().length > 0}>
            <div
              ref={resultListRef}
              class="overflow-y-auto overflow-x-hidden"
              style={{ height: `${resultViewportHeight()}px` }}
              onScroll={(event) => setResultScrollTop(event.currentTarget.scrollTop)}
            >
              <div
                class="relative"
                style={{ height: `${results().length * RESULT_ROW_HEIGHT}px` }}
              >
                <For each={visibleResults()}>
                  {({ item: ref, index }) => {
                  const typeStyle = () => TYPE_STYLES[getReferenceTypeKey(ref.type_)];
                  const venue = () => getReferenceVenue(ref);
                  return (
                    <button
                      type="button"
                      data-item-index={index}
                      class={`absolute left-2 right-2 rounded-field border border-l-4 px-3 py-2 text-left transition-colors ${
                        typeStyle().borderClass
                      } ${
                        index === selectedIndex()
                          ? "border-primary/40 bg-primary/10 ring-1 ring-primary/30"
                          : "border-base-300 bg-base-100 hover:bg-base-200"
                      }`}
                      style={{
                        top: `${index * RESULT_ROW_HEIGHT + 4}px`,
                        height: `${RESULT_ROW_HEIGHT - 8}px`,
                      }}
                      onClick={() => handleSelect(ref)}
                      onMouseEnter={() => setSelectedIndex(index)}
                      aria-selected={index === selectedIndex()}
                    >
                      <div class="flex items-start gap-2">
                        <span class={`${typeStyle().badgeClass} badge-sm gap-1 shrink-0`}>
                          <SvgIcon name={typeStyle().icon} size={12} aria-hidden />
                          {typeStyle().label}
                        </span>
                        <div class="min-w-0 flex-1">
                          <div class="truncate font-medium leading-snug text-base-content">
                            <Show
                              when={ref.title}
                              fallback={<span class="italic">暂无标题</span>}
                            >
                              <ChunksComp chunks={ref.title!} citeKey={ref.cite_key} />
                            </Show>
                          </div>
                          <div class="mt-1 flex flex-wrap gap-1 text-xs">
                            <span class={`inline-flex items-center gap-1 rounded-field px-1.5 py-0.5 ${METADATA_STYLES.citeKey.chipClass}`}>
                              <SvgIcon name={METADATA_STYLES.citeKey.icon} size={11} aria-hidden />
                              {ref.cite_key}
                            </span>
                            <Show when={ref.author?.[0]}>
                              <span class={`inline-flex items-center gap-1 rounded-field px-1.5 py-0.5 ${METADATA_STYLES.author.chipClass}`}>
                                <SvgIcon name={METADATA_STYLES.author.icon} size={11} aria-hidden />
                                {ref.author!.slice(0, 2).join(", ")}
                              </span>
                            </Show>
                            <Show when={ref.year}>
                              <span class={`inline-flex items-center gap-1 rounded-field px-1.5 py-0.5 ${METADATA_STYLES.year.chipClass}`}>
                                <SvgIcon name={METADATA_STYLES.year.icon} size={11} aria-hidden />
                                {ref.year}
                              </span>
                            </Show>
                            <Show when={venue()}>
                              <span class={`inline-flex items-center gap-1 rounded-field px-1.5 py-0.5 ${METADATA_STYLES.venue.chipClass}`}>
                                <SvgIcon name={METADATA_STYLES.venue.icon} size={11} aria-hidden />
                                {venue()}
                              </span>
                            </Show>
                          </div>
                        </div>
                      </div>
                    </button>
                  );
                }}
              </For>
              </div>
            </div>
            {renderError()}
          </Match>
        </Switch>
      </div>
    </div>
  );
}

export default HelperPage;
