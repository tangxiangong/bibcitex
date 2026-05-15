import {
  createEffect,
  createSignal,
  For,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";
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
  const [failedPasteKey, setFailedPasteKey] = createSignal<string | null>(null);
  const [bibs, setBibs] = createSignal<BibInfo[]>([]);
  const [currentBib, setCurrentBib] = createSignal<
    { name: string; refs: Reference[] } | null
  >(null);

  let inputRef: HTMLInputElement | undefined;
  let containerRef: HTMLDivElement | undefined;
  let contentRef: HTMLDivElement | undefined;
  let searchTimeoutRef: ReturnType<typeof setTimeout> | null = null;
  let resizeRafId: number | null = null;
  let lastHeight = MIN_HEIGHT;

  const measureContentHeight = (): number => {
    if (!contentRef) return MIN_HEIGHT;
    if (contentRef.children.length === 0) return HEADER_HEIGHT;

    let contentHeight = 0;
    for (let i = 0; i < contentRef.children.length; i++) {
      contentHeight += (contentRef.children[i] as HTMLElement).scrollHeight;
    }
    return HEADER_HEIGHT + Math.min(contentHeight, MAX_CONTENT_HEIGHT);
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

  const updateWindowHeight = () => {
    if (resizeRafId !== null) cancelAnimationFrame(resizeRafId);
    resizeRafId = requestAnimationFrame(() => {
      resizeRafId = requestAnimationFrame(() => {
        resizeRafId = null;
        doResize();
      });
    });
  };

  const doSearch = (value: string) => {
    if (searchTimeoutRef) clearTimeout(searchTimeoutRef);

    const bib = currentBib();
    if (!bib || value.trim() === "") {
      setResults([]);
      setSelectedIndex(null);
      updateWindowHeight();
      return;
    }

    searchTimeoutRef = setTimeout(async () => {
      try {
        const found = await searchReferences(bib.refs, value);
        setResults(found);
        setSelectedIndex(found.length > 0 ? 0 : null);
        updateWindowHeight();
      } catch (e) {
        console.error("Search failed:", e);
        setResults([]);
      }
    }, 100);
  };

  const handleInput = (event: Event & { currentTarget: HTMLInputElement }) => {
    const value = event.currentTarget.value;
    setQuery(value);
    setFailedPasteKey(null);
    if (!isSelectingBib()) doSearch(value);
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
      setFailedPasteKey(null);
      await setHelperBib(name, path);
      updateWindowHeight();
      setTimeout(() => inputRef?.focus(), 50);
    } catch (e) {
      setErrorMessage(`加载失败: ${e}`);
      updateWindowHeight();
    }
  };

  const scrollToItem = (index: number) => {
    const item = document.querySelector(
      `[data-item-index="${index}"]`,
    ) as HTMLElement | null;
    const container = item?.closest(".overflow-y-auto") as HTMLElement | null;
    if (!item || !container) return;

    const containerRect = container.getBoundingClientRect();
    const itemRect = item.getBoundingClientRect();
    const itemTop = itemRect.top - containerRect.top + container.scrollTop;
    const itemHeight = itemRect.height;
    const targetScrollTop = index < 3
      ? Math.max(0, itemTop - 8)
      : itemTop - containerRect.height / 2 + itemHeight / 2;

    container.scrollTo({ top: targetScrollTop, behavior: "smooth" });
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
      updateWindowHeight();
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
      const len = bibs().length;
      if (len === 0) return;

      if (event.key === "ArrowDown") {
        event.preventDefault();
        const index = bibSelectedIndex();
        setBibSelectedIndex(index === null ? 0 : Math.min(index + 1, len - 1));
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        const index = bibSelectedIndex();
        setBibSelectedIndex(index === null ? len - 1 : Math.max(index - 1, 0));
      } else if (event.key === "Enter") {
        event.preventDefault();
        const index = bibSelectedIndex();
        if (index !== null && bibs()[index]) {
          await selectBib(bibs()[index].name, bibs()[index].path);
        }
      }
      return;
    }

    const len = results().length;
    if (len === 0) return;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      const index = selectedIndex();
      const nextIndex = index === null ? 0 : Math.min(index + 1, len - 1);
      setSelectedIndex(nextIndex);
      scrollToItem(nextIndex);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      const index = selectedIndex();
      const nextIndex = index === null ? len - 1 : Math.max(index - 1, 0);
      setSelectedIndex(nextIndex);
      scrollToItem(nextIndex);
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
    setTimeout(() => inputRef?.focus(), 50);
    updateWindowHeight();
  };

  onMount(async () => {
    inputRef?.focus();
    try {
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
      updateWindowHeight();

      const storedBib = await getHelperBib();
      if (
        storedBib &&
        loadedBibs.some((bib) => bib.name === storedBib[0] && bib.path === storedBib[1])
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

  onMount(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
  });

  onMount(() => {
    if (!contentRef) return;
    const observer = new MutationObserver(() => updateWindowHeight());
    observer.observe(contentRef, {
      childList: true,
      subtree: true,
      characterData: true,
    });
    onCleanup(() => observer.disconnect());
  });

  onCleanup(() => {
    document.documentElement.style.background = "";
    document.body.style.background = "";
    if (searchTimeoutRef) clearTimeout(searchTimeoutRef);
    if (resizeRafId !== null) cancelAnimationFrame(resizeRafId);
  });

  createEffect(() => {
    results().length;
    bibs().length;
    isSelectingBib();
    errorMessage();
    failedPasteKey();
    updateWindowHeight();
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
      ref={containerRef}
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
            ? "选择文献库..."
            : "搜索文献、作者、标题..."}
          readOnly={isSelectingBib()}
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
              class="badge bg-primary/10 text-base-content border-primary/30 cursor-pointer gap-1"
              onClick={handleBibSelectClick}
            >
              <SvgIcon name="library" size={12} aria-hidden />
              {currentBib()!.name}
            </button>
          </Show>
        </div>
      </div>

      <div ref={contentRef} class="w-full max-w-full overflow-hidden">
        <Switch>
          <Match when={isSelectingBib() && bibs().length === 0}>
            <div class="shrink-0 px-5 py-10 text-center text-sm text-base-content/60">
              未找到文献库，请先在主页添加文献库
            </div>
          </Match>

          <Match when={isSelectingBib()}>
            <div class="flex h-full w-full flex-col overflow-hidden">
              <div
                class="flex-1 space-y-1 overflow-y-auto overflow-x-hidden p-2"
                style={`max-height: ${MAX_CONTENT_HEIGHT}px`}
              >
                <For each={bibs()}>
                  {(bib, index) => (
                    <button
                      type="button"
                      data-item-index={index()}
                      class={`w-full rounded-field border px-3 py-2 text-left transition-colors ${
                        index() === bibSelectedIndex()
                          ? "border-primary/40 bg-primary/10"
                          : "border-transparent hover:border-base-300 hover:bg-base-200"
                      }`}
                      onClick={() => selectBib(bib.name, bib.path)}
                      onMouseEnter={() => setBibSelectedIndex(index())}
                    >
                      <div class="flex items-center justify-between gap-2">
                        <div class="flex min-w-0 items-center gap-2">
                          <span class="badge bg-success/10 text-base-content border-success/30 badge-xs">
                            可用
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
                        {bib.path}
                      </p>
                    </button>
                  )}
                </For>
              </div>
              {renderError()}
            </div>
          </Match>

          <Match when={!isSelectingBib() && query().trim() === ""}>
            <div class="flex h-32 flex-col items-center justify-center gap-3 text-base-content/45">
              <SvgIcon name="search" size={42} aria-hidden />
              <span class="text-sm font-medium">开始输入以搜索文献...</span>
            </div>
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
              class="space-y-1 overflow-y-auto overflow-x-hidden p-2"
              style={`max-height: ${MAX_CONTENT_HEIGHT}px`}
            >
              <For each={results()}>
                {(ref, index) => {
                  const typeStyle = () => TYPE_STYLES[getReferenceTypeKey(ref.type_)];
                  const venue = () => getReferenceVenue(ref);
                  return (
                    <button
                      type="button"
                      data-item-index={index()}
                      class={`w-full rounded-field border border-l-4 px-3 py-2 text-left transition-colors ${
                        typeStyle().borderClass
                      } ${
                        index() === selectedIndex()
                          ? "border-primary/40 bg-primary/10 ring-1 ring-primary/30"
                          : "border-base-300 bg-base-100 hover:bg-base-200"
                      }`}
                      onClick={() => handleSelect(ref)}
                      onMouseEnter={() => setSelectedIndex(index())}
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
            {renderError()}
          </Match>
        </Switch>
      </div>
    </div>
  );
}

export default HelperPage;
