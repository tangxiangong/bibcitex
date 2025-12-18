<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { searchReferences, copyToClipboard, pasteToApp, loadBibliography, loadSettings, resizeHelperWindow, getHelperBib, setHelperBib } from '$lib/tauri';
  import type { Reference, Setting, BibliographyInfo, EntryType } from '$lib/types';
  import { getChunkValue } from '$lib/types';
  import ChunksComp from '$lib/components/ChunksComp.svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const MIN_HEIGHT = 70;  // Must match Rust side: 70px for search bar (64px) + padding
  const MAX_HEIGHT = 800;

  // 状态
  let query = $state('');
  let results = $state<Reference[]>([]);
  let selectedIndex = $state<number | null>(0);
  let isSelectingBib = $state(true);
  let bibSelectedIndex = $state<number | null>(0);
  let errorMessage = $state<string | null>(null);

  // 文献库
  let bibs = $state<{ name: string; path: string; updatedAt: string; description?: string; exists: boolean }[]>([]);
  let currentBib = $state<{ name: string; refs: Reference[] } | null>(null);

  let inputRef: HTMLInputElement | null = null;
  let containerRef: HTMLDivElement | null = null;
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;
  let lastHeight = MIN_HEIGHT;

  // 调整窗口高度 - 只有高度真正变化时才调用 Tauri 命令
  async function updateWindowHeight() {
    if (!containerRef) return;

    await tick(); // 等待 DOM 更新

    const rect = containerRef.getBoundingClientRect();
    const measuredHeight = Math.round(rect.height);
    const finalHeight = Math.min(Math.max(measuredHeight, MIN_HEIGHT), MAX_HEIGHT);

    console.log('[Helper] Container height:', measuredHeight, 'Final height:', finalHeight);

    // 只有高度变化超过阈值才真正调整（避免频繁调用）
    if (Math.abs(finalHeight - lastHeight) > 10) {
      lastHeight = finalHeight;
      console.log('[Helper] Resizing window to:', finalHeight);
      await resizeHelperWindow(finalHeight);
    }
  }

  // 搜索函数 - 搜索完成后更新高度
  async function doSearch(q: string) {
    if (searchTimeout) clearTimeout(searchTimeout);

    if (!currentBib || q.trim() === '') {
      results = [];
      selectedIndex = null;
      // 结果变化，更新高度
      setTimeout(updateWindowHeight, 50);
      return;
    }

    searchTimeout = setTimeout(async () => {
      try {
        const r = await searchReferences(currentBib!.refs, q);
        results = r;
        selectedIndex = r.length > 0 ? 0 : null;
        // 结果变化，更新高度
        setTimeout(updateWindowHeight, 50);
      } catch (e) {
        console.error('Search failed:', e);
        results = [];
      }
    }, 100);
  }

  // 处理输入事件 - 不触发高度调整，只更新 query 和搜索
  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    query = target.value;
    if (!isSelectingBib) {
      doSearch(query);
    }
  }

  onMount(async () => {
    // 立即尝试聚焦
    inputRef?.focus();

    try {
      const settings = await loadSettings();
      const bibEntries = Object.entries(settings.bibliographies);

      bibs = bibEntries.map(([name, info]) => ({
        name,
        path: info.path,
        updatedAt: info.updated_at,
        description: info.description,
        exists: true
      })).sort((a, b) => b.updatedAt.localeCompare(a.updatedAt));

      // 加载完 bibs 后更新高度
      await tick();
      updateWindowHeight();

      // 从 Rust 端获取之前选择的文献库
      const storedBib = await getHelperBib();
      if (storedBib && bibs.some(b => b.name === storedBib[0] && b.path === storedBib[1])) {
        await selectBib(storedBib[0], storedBib[1]);
      }
      // 否则保持 isSelectingBib = true，显示选择界面
    } catch (e) {
      console.error('Failed to load settings:', e);
    }

    // 多次尝试确保聚焦（窗口可能需要时间完全就绪）
    const focusInput = () => inputRef?.focus();
    focusInput();
    setTimeout(focusInput, 50);
    setTimeout(focusInput, 150);
    setTimeout(focusInput, 300);
  });

  onDestroy(() => {
    if (searchTimeout) clearTimeout(searchTimeout);
  });

  async function selectBib(name: string, path: string) {
    try {
      const refs = await loadBibliography(path);
      currentBib = { name, refs };
      isSelectingBib = false;
      query = '';
      results = [];
      selectedIndex = null;
      errorMessage = null;

      // 保存选择到 Rust 端
      await setHelperBib(name, path);

      // 切换模式后更新高度
      setTimeout(updateWindowHeight, 100);

      // 选择文献库后设置焦点到输入框
      setTimeout(() => inputRef?.focus(), 150);
    } catch (e) {
      errorMessage = `加载失败: ${e}`;
    }
  }

  async function handleKeyDown(e: KeyboardEvent) {
    // ESC 关闭窗口 - 始终优先处理
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      try {
        const win = getCurrentWindow();
        await win.close();
      } catch (err) {
        console.error('Failed to close window:', err);
      }
      return;
    }

    if (isSelectingBib) {
      // 文献库选择模式
      const len = bibs.length;
      if (len === 0) return;

      if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (bibSelectedIndex === null) {
          bibSelectedIndex = 0;
        } else {
          bibSelectedIndex = Math.min(bibSelectedIndex + 1, len - 1);
        }
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (bibSelectedIndex === null) {
          bibSelectedIndex = len - 1;
        } else {
          bibSelectedIndex = Math.max(bibSelectedIndex - 1, 0);
        }
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (bibSelectedIndex !== null && bibs[bibSelectedIndex]) {
          await selectBib(bibs[bibSelectedIndex].name, bibs[bibSelectedIndex].path);
        }
      }
    } else {
      // 搜索模式
      const len = results.length;
      if (len === 0) return;

      if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (selectedIndex === null) {
          selectedIndex = 0;
        } else {
          selectedIndex = Math.min(selectedIndex + 1, len - 1);
        }
        scrollToItem(selectedIndex);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (selectedIndex === null) {
          selectedIndex = len - 1;
        } else {
          selectedIndex = Math.max(selectedIndex - 1, 0);
        }
        scrollToItem(selectedIndex);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (selectedIndex !== null && results[selectedIndex]) {
          await handleSelect(results[selectedIndex]);
        }
      }
    }
  }

  function scrollToItem(index: number) {
    const item = document.querySelector(`[data-item-index="${index}"]`) as HTMLElement;
    if (!item) return;

    const container = item.closest('.overflow-y-auto') as HTMLElement;
    if (!container) return;

    const containerRect = container.getBoundingClientRect();
    const itemRect = item.getBoundingClientRect();

    // 计算项目相对于容器的位置
    const itemTop = itemRect.top - containerRect.top + container.scrollTop;
    const itemHeight = itemRect.height;
    const containerHeight = containerRect.height;

    // 计算居中位置（除了前几个项目）
    let targetScrollTop;
    if (index < 3) {
      // 前3个项目保持在顶部
      targetScrollTop = Math.max(0, itemTop - 8); // 8px padding
    } else {
      // 其他项目居中显示
      targetScrollTop = itemTop - (containerHeight / 2) + (itemHeight / 2);
    }

    // 平滑滚动到目标位置
    container.scrollTo({
      top: targetScrollTop,
      behavior: 'smooth'
    });
  }

  async function handleSelect(ref: Reference) {
    try {
      // 复制 cite_key 而不是整个 source
      await copyToClipboard(ref.cite_key);
      await pasteToApp(ref.cite_key);
      const window = getCurrentWindow();
      await window.close();
    } catch (e) {
      console.error('Paste failed:', e);
    }
  }

  function handleBibSelectClick() {
    isSelectingBib = true;
    query = '';
    results = [];
    selectedIndex = null;
    bibSelectedIndex = 0;
    errorMessage = null;
    setTimeout(() => inputRef?.focus(), 50);
  }

  function getEntryTypeLabel(type_: EntryType): string {
    if (typeof type_ === 'string') {
      if (type_ === 'MastersThesis') return 'Master Thesis';
      if (type_ === 'PhdThesis') return 'PhD Thesis';
      return type_;
    }
    if (typeof type_ === 'object' && 'Unknown' in type_) return type_.Unknown;
    return 'Unknown';
  }

  // 检查是否为 ArXiv 文献
  function isArXiv(ref: Reference): boolean {
    return ref.archive_prefix === 'arXiv';
  }

  // 获取 ArXiv 标识
  function getArXivLabel(ref: Reference): string {
    const eprint = ref.eprint;
    const primaryClass = ref.arxiv_primary_class;
    if (eprint && primaryClass) {
      return `arXiv:${eprint} [${primaryClass}]`;
    } else if (eprint) {
      return `arXiv:${eprint}`;
    } else if (primaryClass) {
      return `arXiv [${primaryClass}]`;
    }
    return 'arXiv';
  }

  // 获取学校和地址
  function getSchoolAddress(ref: Reference): string {
    if (ref.school) {
      if (ref.address) {
        return `${ref.school} (${ref.address})`;
      }
      return ref.school;
    }
    return '';
  }

  function getBorderColor(type_: EntryType): string {
    const typeStr = typeof type_ === 'string' ? type_ : 'Unknown';
    switch (typeStr) {
      case 'Article': return 'border-l-info';
      case 'Book': return 'border-l-success';
      case 'MastersThesis':
      case 'PhdThesis':
      case 'Thesis': return 'border-l-secondary';
      case 'InProceedings': return 'border-l-primary';
      case 'TechReport': return 'border-l-warning';
      case 'Misc': return 'border-l-neutral';
      case 'Booklet': return 'border-l-info';
      case 'InBook': return 'border-l-accent';
      case 'InCollection': return 'border-l-secondary';
      default: return 'border-l-base-content/20';
    }
  }

  function getBadgeColor(type_: EntryType): string {
    const typeStr = typeof type_ === 'string' ? type_ : 'Unknown';
    switch (typeStr) {
      case 'Article': return 'badge-info badge-soft';
      case 'Book': return 'badge-success badge-soft';
      case 'MastersThesis':
      case 'PhdThesis':
      case 'Thesis': return 'badge-secondary badge-soft';
      case 'InProceedings': return 'badge-primary badge-soft';
      case 'TechReport': return 'badge-warning badge-soft';
      case 'Misc': return 'badge-neutral badge-soft';
      case 'Booklet': return 'badge-info badge-soft';
      case 'InBook': return 'badge-accent badge-soft';
      case 'InCollection': return 'badge-secondary badge-soft';
      default: return 'badge-ghost';
    }
  }
</script>

<div
  class="helper-container flex flex-col overflow-hidden rounded-xl h-fit w-full max-w-full box-border"
  style="max-height: {MAX_HEIGHT}px;"
  bind:this={containerRef}
>
  <!-- 搜索输入框 -->
  <div class="relative w-full max-w-full h-16 bg-transparent z-20 border-b border-base-content/10 shrink-0 overflow-hidden box-border" data-tauri-drag-region>
    <!-- Search Icon -->
    <div class="absolute left-4 top-1/2 -translate-y-1/2 text-base-content/40" data-tauri-drag-region>
      <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>

    <input
      bind:this={inputRef}
      value={query}
      oninput={handleInput}
      onkeydown={handleKeyDown}
      class="w-full max-w-full h-full pl-12 pr-36 text-lg bg-transparent border-none outline-none ring-0 focus:border-none focus:outline-none focus:ring-0 placeholder:text-base-content/30 text-base-content disabled:cursor-default disabled:opacity-100 overflow-hidden"
      type="text"
      placeholder={isSelectingBib ? "选择文献库..." : "搜索文献、作者、标题..."}
      readonly={isSelectingBib}
    />

    <div class="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-2">
      {#if currentBib}
        <button
          class="badge badge-primary badge-soft gap-1 cursor-pointer hover:scale-105 transition-transform font-medium"
          onclick={handleBibSelectClick}
        >
          <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
          {currentBib.name}
        </button>
      {:else}
        <button
          class="badge badge-ghost cursor-pointer hover:bg-base-200"
          onclick={handleBibSelectClick}
        >
          选择文献库
        </button>
      {/if}
      <img class="opacity-80 hover:opacity-100 transition-opacity duration-300 w-6 h-6" src="/logo.svg" alt="logo" />
    </div>
  </div>

  <!-- 内容区域 -->
  <div class="w-full max-w-full overflow-hidden box-border">
    {#if isSelectingBib}
      <!-- 文献库选择 -->
      {#if bibs.length === 0}
        <div class="shrink-0 px-5 py-10 text-center text-base-content/60 text-sm">
          未找到文献库，请先在主页添加文献库
        </div>
      {:else}
        <div class="flex flex-col h-full w-full max-w-full overflow-hidden box-border">
          <div class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-2 scroll-smooth w-full max-w-full box-border" style="max-height: {MAX_HEIGHT - MIN_HEIGHT - (errorMessage ? 60 : 0)}px;">
            {#each bibs as bib, i}
              <button
                data-item-index={i}
                class="w-full max-w-full text-left rounded-lg transition-all duration-200 cursor-pointer mx-2 overflow-hidden {i === bibSelectedIndex ? 'bg-primary/10 text-primary shadow-sm' : 'hover:bg-base-200/50 hover:shadow-sm border border-transparent'}"
                onclick={() => selectBib(bib.name, bib.path)}
                onmouseenter={() => bibSelectedIndex = i}
              >
                <div class="p-3">
                  <div class="flex items-center justify-between gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      {#if bib.exists}
                        <div class="badge badge-success badge-xs gap-1 border-none shrink-0">
                          <div class="w-1.5 h-1.5 rounded-full bg-white animate-pulse"></div>
                          Ready
                        </div>
                      {:else}
                        <div class="badge badge-error badge-xs gap-1 border-none shrink-0">
                          <div class="w-1.5 h-1.5 rounded-full bg-white"></div>
                          Error
                        </div>
                      {/if}
                      <h3 class="font-bold text-base truncate">{bib.name}</h3>
                    </div>
                    <span class="text-xs opacity-60 font-mono shrink-0">{bib.updatedAt}</span>
                  </div>
                  {#if bib.description}
                    <p class="text-sm opacity-80 mt-1">{bib.description}</p>
                  {/if}
                  <p class="text-xs opacity-50 truncate mt-1 font-mono">{bib.path}</p>
                </div>
              </button>
            {/each}
          </div>
          {#if errorMessage}
            <div class="alert alert-error shadow-lg m-2">
              <span>{errorMessage}</span>
            </div>
          {/if}
        </div>
      {/if}
    {:else if query.trim() === ''}
      <!-- 空查询提示 -->
      <div class="flex flex-col items-center justify-center h-32 text-base-content/40 gap-4">
        <svg class="w-12 h-12 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <span class="text-sm font-medium">开始输入以搜索文献...</span>
      </div>
    {:else if results.length === 0}
      <!-- 无结果 -->
      <div class="flex flex-col items-center justify-center h-32 text-base-content/40 gap-4">
        <svg class="w-12 h-12 opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-sm font-medium">未找到匹配的文献</span>
      </div>
    {:else}
      <!-- 搜索结果 -->
      <div class="overflow-y-auto overflow-x-hidden p-2 space-y-2 w-full max-w-full box-border" style="max-height: {MAX_HEIGHT - MIN_HEIGHT}px;">
        {#each results as ref, i}
          <button
            data-item-index={i}
            class="w-full max-w-full text-left group relative rounded-lg px-1 transition-all duration-200 cursor-pointer border-l-[3px] mx-2 overflow-hidden box-border {getBorderColor(ref.type_)} {i === selectedIndex ? 'bg-primary/15 shadow-md ring-1 ring-primary/30 scale-[1.01]' : 'hover:bg-base-200/50 border-opacity-50 hover:border-opacity-100'}"
            onclick={() => handleSelect(ref)}
            onmouseenter={() => selectedIndex = i}
          >
            <div class="py-3 px-3 w-full max-w-full overflow-hidden box-border">
              {#if ref.type_ === 'Article'}
                <!-- Article -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge badge-info badge-soft badge-sm font-bold shrink-0">Article</div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif font-medium truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif italic truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-xs font-mono opacity-50">{ref.cite_key}</div>
                    </div>
                  </div>
                  <div class="mt-1 flex flex-wrap gap-1">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-ghost badge-xs hover:badge-info transition-colors cursor-default">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-ghost badge-xs hover:badge-info transition-colors cursor-default">et al.</span>
                      {/if}
                    {:else}
                      <span class="text-xs text-base-content/50 italic">Unknown Author</span>
                    {/if}
                  </div>
                  <div class="mt-1 flex flex-wrap items-center gap-2 text-xs">
                    {#if ref.journal}
                      <span class="flex items-center gap-1">📖 {ref.journal}</span>
                    {/if}
                    {#if ref.year}
                      <span class="flex items-center gap-1 text-secondary">📅 {ref.year}</span>
                    {/if}
                  </div>
                </div>
              {:else if ref.type_ === 'Book'}
                <!-- Book -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge badge-success badge-soft badge-sm font-bold shrink-0">Book</div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif font-medium truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif italic truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-xs font-mono opacity-50">{ref.cite_key}</div>
                    </div>
                  </div>
                  <div class="mt-1 flex flex-wrap gap-1">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-ghost badge-xs hover:badge-success transition-colors cursor-default">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-ghost badge-xs hover:badge-success transition-colors cursor-default">et al.</span>
                      {/if}
                    {:else}
                      <span class="text-xs text-base-content/50 italic">Unknown Author</span>
                    {/if}
                  </div>
                  <div class="mt-1 flex flex-wrap items-center gap-2 text-xs">
                    {#if ref.publisher}
                      {#each ref.publisher as pub}
                        <span class="flex items-center gap-1">🏢 {pub}</span>
                      {/each}
                    {/if}
                    {#if ref.year}
                      <span class="flex items-center gap-1 text-secondary">📅 {ref.year}</span>
                    {/if}
                  </div>
                </div>
              {:else if ref.type_ === 'Thesis' || ref.type_ === 'MastersThesis' || ref.type_ === 'PhdThesis'}
                <!-- Thesis -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge badge-outline shrink-0 {ref.type_ === 'MastersThesis' ? 'text-pink-800 dark:text-pink-200' : 'text-rose-800 dark:text-rose-200'}">
                        {getEntryTypeLabel(ref.type_)}
                      </div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-gray-600 dark:text-gray-400 text-xs font-mono">{ref.cite_key}</div>
                    </div>
                  </div>
                  <p class="text-xs mt-2 break-all">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">et al.</span>
                      {/if}
                    {:else}
                      <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">Unknown</span>
                    {/if}
                  </p>
                  <p class="text-xs mt-2 break-all">
                    {#if getSchoolAddress(ref)}
                      <span class="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">{getSchoolAddress(ref)}</span>
                    {:else}
                      <span class="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">Unknown</span>
                    {/if}
                    {#if ref.year}
                      <span class="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">{ref.year}</span>
                    {:else}
                      <span class="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">year</span>
                    {/if}
                  </p>
                </div>
              {:else if ref.type_ === 'InProceedings'}
                <!-- InProceedings -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge badge-primary badge-soft badge-sm font-bold shrink-0">InProceedings</div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif font-medium truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif italic truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-xs font-mono opacity-50">{ref.cite_key}</div>
                    </div>
                  </div>
                  <div class="mt-1 flex flex-wrap gap-1">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-ghost badge-xs hover:badge-primary transition-colors cursor-default">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-ghost badge-xs hover:badge-primary transition-colors cursor-default">et al.</span>
                      {/if}
                    {:else}
                      <span class="text-xs text-base-content/50 italic">Unknown Author</span>
                    {/if}
                  </div>
                  {#if ref.book_title}
                    <div class="mt-1 text-xs text-base-content/70 flex items-center gap-2">
                      <span class="italic">
                        <ChunksComp chunks={ref.book_title} citeKey={`booktitle-helper-${ref.cite_key}`} />
                      </span>
                    </div>
                  {/if}
                  <div class="mt-1 flex flex-wrap items-center gap-2 text-xs">
                    {#if ref.year}
                      <span class="flex items-center gap-1 text-secondary">📅 {ref.month ? `${ref.year}-${ref.month}` : ref.year}</span>
                    {/if}
                  </div>
                </div>
              {:else if ref.type_ === 'TechReport'}
                <!-- TechReport -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge badge-outline shrink-0 text-amber-800 dark:text-amber-200">TechReport</div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-gray-600 dark:text-gray-400 text-xs font-mono">{ref.cite_key}</div>
                    </div>
                  </div>
                  <p class="text-xs mt-2 break-all">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">et al.</span>
                      {/if}
                    {:else}
                      <span class="badge badge-outline text-blue-700 dark:text-blue-300 font-semibold mr-2">Unknown</span>
                    {/if}
                  </p>
                  <p class="text-xs mt-2 break-all">
                    {#if ref.institution}
                      <span class="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">{ref.institution}</span>
                    {:else}
                      <span class="badge badge-outline text-purple-600 dark:text-purple-300 mr-2">Unknown</span>
                    {/if}
                    {#if ref.year}
                      <span class="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">{ref.year}</span>
                    {:else}
                      <span class="badge badge-outline text-emerald-700 dark:text-emerald-300 mr-2">year</span>
                    {/if}
                  </p>
                </div>
              {:else if ref.type_ === 'Misc' && isArXiv(ref)}
                <!-- ArXiv (Misc) -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge badge-error badge-soft badge-sm font-bold shrink-0">Misc</div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif font-medium truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif italic truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-xs font-mono opacity-50">{ref.cite_key}</div>
                    </div>
                  </div>
                  <div class="mt-1 flex flex-wrap gap-1">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-ghost badge-xs hover:badge-error transition-colors cursor-default">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-ghost badge-xs hover:badge-error transition-colors cursor-default">et al.</span>
                      {/if}
                    {:else}
                      <span class="text-xs text-base-content/50 italic">Unknown Author</span>
                    {/if}
                  </div>
                  <div class="mt-1 flex flex-wrap items-center gap-2 text-xs">
                    <span class="flex items-center gap-1 text-error">📜 {getArXivLabel(ref)}</span>
                    {#if ref.year}
                      <span class="flex items-center gap-1 text-secondary">📅 {ref.year}</span>
                    {/if}
                  </div>
                </div>
              {:else if ref.type_ === 'Misc'}
                <!-- Misc (non-ArXiv) -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge badge-neutral badge-soft badge-sm font-bold shrink-0">Misc</div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif font-medium truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif italic truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-xs font-mono opacity-50">{ref.cite_key}</div>
                    </div>
                  </div>
                  <div class="mt-1 flex flex-wrap gap-1">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-ghost badge-xs hover:badge-neutral transition-colors cursor-default">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-ghost badge-xs hover:badge-neutral transition-colors cursor-default">et al.</span>
                      {/if}
                    {:else}
                      <span class="text-xs text-base-content/50 italic">Unknown Author</span>
                    {/if}
                  </div>
                  <div class="mt-1 flex flex-wrap items-center gap-2 text-xs">
                    {#if ref.archive_prefix}
                      <span class="flex items-center gap-1">📦 {ref.archive_prefix}</span>
                    {:else if ref.how_published}
                      <span class="flex items-center gap-1">📢 {ref.how_published}</span>
                    {/if}
                    {#if ref.year}
                      <span class="flex items-center gap-1 text-secondary">📅 {ref.year}</span>
                    {/if}
                  </div>
                </div>
              {:else}
                <!-- Default / Other types (Booklet, InBook, InCollection, etc.) -->
                <div class="w-full">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex items-center gap-2 flex-1 min-w-0">
                      <div class="badge {getBadgeColor(ref.type_)} badge-sm font-bold shrink-0">
                        {getEntryTypeLabel(ref.type_)}
                      </div>
                      {#if ref.title}
                        <span class="text-gray-900 dark:text-gray-100 font-serif font-medium truncate">
                          <ChunksComp chunks={ref.title} citeKey={ref.cite_key} />
                        </span>
                      {:else}
                        <span class="text-gray-900 dark:text-gray-100 font-serif italic truncate">No title available</span>
                      {/if}
                    </div>
                    <div class="flex items-center shrink-0">
                      <div class="text-xs font-mono opacity-50">{ref.cite_key}</div>
                    </div>
                  </div>
                  <div class="mt-1 flex flex-wrap gap-1">
                    {#if ref.author}
                      {#each ref.author.slice(0, 3) as author}
                        <span class="badge badge-ghost badge-xs hover:badge-info transition-colors cursor-default">{author}</span>
                      {/each}
                      {#if ref.author.length > 3}
                        <span class="badge badge-ghost badge-xs hover:badge-info transition-colors cursor-default">et al.</span>
                      {/if}
                    {:else}
                      <span class="text-xs text-base-content/50 italic">Unknown Author</span>
                    {/if}
                  </div>
                  <div class="mt-1 flex flex-wrap items-center gap-2 text-xs">
                    {#if ref.journal}
                      <span class="flex items-center gap-1">📖 {ref.journal}</span>
                    {/if}
                    {#if ref.publisher}
                      {#each ref.publisher as pub}
                        <span class="flex items-center gap-1">🏢 {pub}</span>
                      {/each}
                    {/if}
                    {#if ref.year}
                      <span class="flex items-center gap-1 text-secondary">📅 {ref.year}</span>
                    {/if}
                  </div>
                </div>
              {/if}
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .helper-container {
    background: oklch(var(--b1) / 0.95);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    box-shadow: 0 25px 50px -12px oklch(var(--bc) / 0.25);
    border: 1px solid oklch(var(--bc) / 0.1);
    border-radius: 16px;
    width: 100%;
    max-width: 100%;
    overflow-x: hidden;
  }

  /* 跨平台圆角效果 */
  :global(html), :global(body) { background: transparent !important;
    margin: 0;
    padding: 0;
    overflow: visible;
    height: 100%;
    min-height: 100vh;
  }

  /* 为不同平台优化圆角效果 */
  @media (prefers-color-scheme: dark) {
    .helper-container {
      background: oklch(var(--b1) / 0.98);
      border: 1px solid oklch(var(--bc) / 0.15);
    }
  }

  /* Windows 特定优化 */
  @media screen and (-ms-high-contrast: active), (-ms-high-contrast: none) {
    .helper-container {
      border-radius: 12px;
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    }
  }

  /* Linux 特定优化 - 删除损坏的 @supports 规则 */

  input:focus {
    box-shadow: none !important;
  }

  /* 确保所有 flex 容器正确处理溢出 */
  .helper-container .flex {
    min-width: 0;
  }

  /* 确保文本截断正常工作 */
  .helper-container .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* 防止 badge 和其他元素导致水平滚动 */
  .helper-container .badge {
    flex-shrink: 0;
  }

  /* 确保搜索结果项不会溢出 */
  .helper-container button {
    max-width: 100%;
    overflow: hidden;
    box-sizing: border-box;
  }

  /* 强制所有子元素不超出容器宽度 */
  .helper-container * {
    max-width: 100%;
    box-sizing: border-box;
  }

  /* 特别处理 flex 容器中的文本 */
  .helper-container .flex > span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  /* 确保 badge 不会导致溢出 */
  .helper-container .badge {
    flex-shrink: 0;
    max-width: fit-content;
  }

  /* 强制防止水平溢出 */
  .helper-container {
    width: 100% !important;
    max-width: 100% !important;
    overflow-x: hidden !important;
  }

  .helper-container * {
    max-width: 100% !important;
    box-sizing: border-box !important;
  }

  /* 特别处理搜索结果项 */
  .helper-container button {
    width: 100% !important;
    max-width: 100% !important;
    overflow: hidden !important;
  }

  /* 处理 flex 布局中的文本溢出 */
  .helper-container .flex {
    min-width: 0 !important;
  }

  .helper-container .flex > * {
    min-width: 0 !important;
  }

  /* 确保标题文本正确截断 */
  .helper-container .truncate {
    overflow: hidden !important;
    text-overflow: ellipsis !important;
    white-space: nowrap !important;
    max-width: 100% !important;
  }

</style>
