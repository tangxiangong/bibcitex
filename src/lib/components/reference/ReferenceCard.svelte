<script lang="ts">
  import type { Reference, Chunk } from '$lib/types';
  import { openDrawer } from '$lib/stores/state';
  import { copyToClipboard, openUrl, openFile } from '$lib/tauri';
  import ChunksComp from '../ChunksComp.svelte';

  interface Props {
    entry: Reference;
  }

  let { entry }: Props = $props();

  let copied = $state(false);
  let copySuccess = $state(true);

  // Get border color based on entry type
  const borderColorClass = $derived.by(() => {
    const type = entry.type_;
    if (type === 'Article') return 'border-l-blue-500';
    if (type === 'Book') return 'border-l-green-500';
    if (type === 'Thesis' || type === 'MastersThesis' || type === 'PhdThesis') return 'border-l-purple-500';
    if (type === 'InProceedings') return 'border-l-orange-500';
    if (type === 'TechReport') return 'border-l-yellow-500';
    if (type === 'Misc') return 'border-l-gray-500';
    return 'border-l-primary';
  });

  // Get badge color based on entry type
  const badgeClass = $derived.by(() => {
    const type = entry.type_;
    if (type === 'Article') return 'badge-info';
    if (type === 'Book') return 'badge-success';
    if (type === 'Thesis' || type === 'MastersThesis' || type === 'PhdThesis') return 'badge-secondary';
    if (type === 'InProceedings') return 'badge-warning';
    if (type === 'TechReport') return 'badge-accent';
    return 'badge-ghost';
  });

  const typeLabel = $derived.by(() => {
    const type = entry.type_;
    if (typeof type === 'string') return type;
    if (typeof type === 'object' && 'Unknown' in type) return type.Unknown;
    return 'Unknown';
  });

  const doiUrl = $derived(entry.doi ? `https://doi.org/${entry.doi}` : '');

  async function handleCopyKey() {
    copied = true;
    try {
      await copyToClipboard(entry.cite_key);
      copySuccess = true;
    } catch {
      copySuccess = false;
    }
    setTimeout(() => { copied = false; }, 1500);
  }

  function handleOpenDrawer() {
    openDrawer(entry);
  }

  async function handleOpenDoi() {
    if (doiUrl) {
      await openUrl(doiUrl);
    }
  }

  async function handleOpenUrl() {
    if (entry.url) {
      await openUrl(entry.url);
    }
  }

  async function handleOpenFile() {
    if (entry.file) {
      await openFile(entry.file);
    }
  }
</script>

<div class="card-modern card-shine group hover:-translate-y-1 transition-all duration-300 m-4 border-l-4 {borderColorClass}">
  <div class="card-body p-5">
    <!-- Header: Type + Title + Actions -->
    <div class="flex justify-between items-start gap-4">
      <div class="flex-1">
        <div class="flex items-center gap-2 mb-2">
          <span class="badge {badgeClass} badge-soft badge-sm font-bold">
            {typeLabel}
          </span>
          <span class="text-xs font-mono opacity-50 select-all">
            {entry.cite_key}
          </span>
        </div>
        {#if entry.title}
          <h3 class="text-xl font-bold leading-snug gradient-text">
            <ChunksComp chunks={entry.title} citeKey={entry.cite_key} />
          </h3>
        {:else}
          <span class="text-lg text-base-content/50 italic">No title available</span>
        {/if}
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
        <button
          class="btn btn-ghost btn-sm btn-circle tooltip tooltip-left"
          data-tip="Copy Key"
          onclick={handleCopyKey}
        >
          {#if !copied}
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          {:else if copySuccess}
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          {/if}
        </button>
        <button
          class="btn btn-ghost btn-sm btn-circle tooltip tooltip-left"
          data-tip="Details"
          aria-label="Details"
          onclick={handleOpenDrawer}
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-70" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Authors -->
    <div class="mt-3 flex flex-wrap gap-2">
      {#if entry.author}
        {#each entry.author as author}
          <span class="badge badge-ghost hover:badge-info transition-colors cursor-default bg-base-200/50">
            {author}
          </span>
        {/each}
      {:else}
        <span class="text-sm text-base-content/50 italic">Unknown Author</span>
      {/if}
    </div>

    <!-- Metadata Row -->
    <div class="mt-4 flex flex-wrap items-center gap-4 text-sm text-base-content/70 border-t border-base-content/5 pt-3">
      {#if entry.journal}
        <div class="flex items-center gap-1">
          <span class="font-semibold text-primary">📖</span>
          <span class="italic">{entry.journal}</span>
        </div>
      {/if}
      {#if entry.year}
        <div class="flex items-center gap-1">
          <span class="font-semibold text-secondary">📅</span>
          <span>{entry.year}</span>
        </div>
      {/if}

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- Links -->
      {#if entry.doi}
        <button
          class="btn btn-xs btn-ghost gap-1 hover:text-info"
          onclick={handleOpenDoi}
        >
          DOI
        </button>
      {/if}
      {#if entry.url}
        <button
          class="btn btn-xs btn-ghost gap-1 hover:text-info"
          onclick={handleOpenUrl}
        >
          URL
        </button>
      {/if}
      {#if entry.file}
        <button
          class="btn btn-xs btn-primary btn-soft gap-1"
          onclick={handleOpenFile}
        >
          PDF
        </button>
      {/if}
    </div>
  </div>
</div>
