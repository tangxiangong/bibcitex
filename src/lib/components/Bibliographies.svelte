<script lang="ts">
  import { settings, bibliographyList } from '$lib/stores/state';
  import { loadSettings, removeBibliography, loadBibliography } from '$lib/tauri';
  import { currentReferences, currentBibName } from '$lib/stores/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  interface Props {
    showModal: boolean;
  }

  let { showModal = $bindable() }: Props = $props();

  let errorMessage = $state<string | null>(null);
  let isFadingOut = $state(false);
  let progress = $state(100);

  onMount(async () => {
    try {
      const loadedSettings = await loadSettings();
      settings.set(loadedSettings);
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  });

  function openModal() {
    showModal = true;
  }

  async function handleSelect(name: string, path: string) {
    try {
      const refs = await loadBibliography(path);
      currentReferences.set(refs);
      currentBibName.set(name);
      goto('/detail');
    } catch (e) {
      errorMessage = `加载失败: ${e}`;
      startErrorTimer();
    }
  }

  async function handleDelete(name: string, event: MouseEvent) {
    event.stopPropagation();
    try {
      await removeBibliography(name);
      const loadedSettings = await loadSettings();
      settings.set(loadedSettings);
    } catch (e) {
      errorMessage = `删除失败: ${e}`;
      startErrorTimer();
    }
  }

  function handleOpenFile(path: string, event: MouseEvent) {
    event.stopPropagation();
    // TODO: 调用 Tauri 打开文件
    console.log('Open file:', path);
  }

  function startErrorTimer() {
    progress = 100;
    const timer = setInterval(() => {
      progress -= 1;
      if (progress <= 0) {
        clearInterval(timer);
        isFadingOut = true;
        setTimeout(() => {
          errorMessage = null;
          isFadingOut = false;
          progress = 100;
        }, 300);
      }
    }, 20);
  }

  function abbrPath(path: string, maxLen: number = 35): string {
    if (path.length <= maxLen) return path;
    const parts = path.split('/');
    if (parts.length <= 2) return '...' + path.slice(-maxLen + 3);
    return parts[0] + '/.../' + parts.slice(-2).join('/');
  }

  function formatDate(dateStr: string): string {
    try {
      return new Date(dateStr).toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
      });
    } catch {
      return dateStr;
    }
  }
</script>

<div class="relative container mx-auto p-6">
  <div class="flex items-center justify-between mb-8">
    <div>
      <h2 class="text-3xl font-bold gradient-text">Bibliographies</h2>
      <p class="text-base-content/60 text-sm mt-1">管理你的文献库</p>
    </div>
    <button class="btn btn-modern gap-2" onclick={openModal}>
      <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      新建文献库
    </button>
  </div>

  <!-- Bibliography Grid -->
  <div class="w-full">
    {#if $bibliographyList.length === 0}
      <div class="flex flex-col items-center justify-center h-64 text-base-content/50">
        <div class="text-4xl mb-4">📚</div>
        <p class="text-lg">未找到文献库</p>
        <p class="text-sm">点击 + 按钮添加一个文献库</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 p-4">
        {#each $bibliographyList as bib}
          {@const isExist = true}
          <div
            class="card-modern card-shine group relative overflow-hidden flex flex-col h-full min-h-50 transition-all duration-500 hover:-translate-y-2 hover:shadow-primary/10 border-white/5 cursor-pointer"
            onclick={() => handleSelect(bib.name, bib.path)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === 'Enter' && handleSelect(bib.name, bib.path)}
          >
            <!-- Decorative Background Elements -->
            <div class="absolute -top-20 -right-20 w-40 h-40 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-all duration-700 animate-blob"></div>
            <div class="absolute -bottom-20 -left-20 w-40 h-40 bg-secondary/5 rounded-full blur-3xl group-hover:bg-secondary/10 transition-all duration-700 animate-blob animation-delay-2000"></div>
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-full h-full bg-linear-to-br from-white/0 via-white/5 to-white/0 opacity-0 group-hover:opacity-100 transition-opacity duration-700 pointer-events-none"></div>

            <div class="card-body p-6 flex-1 relative z-10 backdrop-blur-[2px]">
              <!-- Header -->
              <div class="flex items-start justify-between mb-4">
                <div class="flex items-center gap-3 overflow-hidden">
                  <div class="w-10 h-10 rounded-xl bg-linear-to-br from-primary/10 to-secondary/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-500 shadow-inner border border-white/10">
                    <span class="text-2xl">📚</span>
                  </div>
                  <div>
                    <h3
                      class="text-xl font-bold gradient-text truncate leading-tight"
                      title={bib.name}
                    >
                      {bib.name}
                    </h3>
                    {#if isExist}
                      <div class="flex items-center gap-1 mt-1">
                        <div class="w-1.5 h-1.5 rounded-full bg-success animate-pulse"></div>
                        <span class="text-[10px] uppercase tracking-wider font-bold text-success/80">可用</span>
                      </div>
                    {:else}
                      <div class="flex items-center gap-1 mt-1">
                        <div class="w-1.5 h-1.5 rounded-full bg-error animate-pulse"></div>
                        <span class="text-[10px] uppercase tracking-wider font-bold text-error/80">缺失</span>
                      </div>
                    {/if}
                  </div>
                </div>
              </div>

              <!-- Content -->
              <div class="flex-1 pl-1">
                {#if bib.description}
                  <p
                    class="text-base-content/70 text-sm mb-6 line-clamp-2 font-light leading-relaxed"
                    title={bib.description}
                  >
                    {bib.description}
                  </p>
                {:else}
                  <p class="text-base-content/30 text-sm mb-6 italic font-light">暂无描述</p>
                {/if}

                <div class="flex flex-col gap-3 text-xs text-base-content/60">
                  <div class="flex items-center gap-2 group/link">
                    <span class="opacity-50 group-hover/link:text-primary transition-colors">📂</span>
                    <button
                      class="link link-hover truncate hover:text-primary transition-colors font-mono bg-base-200/50 px-2 py-1 rounded-md w-full text-left border border-transparent hover:border-primary/20 hover:bg-primary/5"
                      onclick={(e) => handleOpenFile(bib.path, e)}
                      title={bib.path}
                    >
                      {abbrPath(bib.path)}
                    </button>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="opacity-50">🕒</span>
                    <span class="font-mono opacity-80">{formatDate(bib.updated_at)}</span>
                  </div>
                </div>
              </div>

              <!-- Actions overlay (visible on hover) -->
              <div class="absolute bottom-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 translate-y-2 group-hover:translate-y-0 transition-all duration-300">
                <button
                  class="btn btn-sm btn-circle btn-ghost text-error hover:bg-error/10 tooltip tooltip-left shadow-sm border border-transparent hover:border-error/20"
                  data-tip="删除"
                  aria-label="删除"
                  onclick={(e) => handleDelete(bib.name, e)}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
                <button
                  class="btn btn-sm btn-primary shadow-lg shadow-primary/30 hover:shadow-primary/50 border-none bg-linear-to-r from-primary to-secondary text-white gap-2 px-4 rounded-full"
                  onclick={() => handleSelect(bib.name, bib.path)}
                >
                  <span>打开</span>
                  <span class="group-hover:translate-x-1 transition-transform">→</span>
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Error Alert -->
  {#if errorMessage}
    <div class="absolute top-2 right-2 w-1/3 z-50 {isFadingOut ? 'animate-fade-out' : 'animate-fade-in'}">
      <div role="alert" class="alert alert-error shadow-lg backdrop-blur-md bg-error/10 border-error/20 flex justify-between items-center">
        <div class="flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="font-medium">{errorMessage}</span>
        </div>
        <div
          class="radial-progress text-error text-xs"
          style="--value:{progress}; --size:1.2rem; --thickness:2px;"
          role="progressbar"
          aria-valuenow={progress}
        ></div>
      </div>
    </div>
  {/if}
</div>

<style>
  .animate-blob {
    animation: blob 7s infinite;
  }

  .animation-delay-2000 {
    animation-delay: 2s;
  }

  @keyframes blob {
    0% { transform: translate(0px, 0px) scale(1); }
    33% { transform: translate(30px, -50px) scale(1.1); }
    66% { transform: translate(-20px, 20px) scale(0.9); }
    100% { transform: translate(0px, 0px) scale(1); }
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
