<script lang="ts">
  import type { Reference } from '$lib/types';
  import { copyToClipboard, openUrl, openFile } from '$lib/tauri';
  import ChunksComp from '../ChunksComp.svelte';

  interface Props {
    entry: Reference;
  }

  let { entry }: Props = $props();

  const typeLabel = $derived.by(() => {
    const type = entry.type_;
    if (typeof type === 'string') {
      if (type === 'Article') return 'Journal Article';
      if (type === 'MastersThesis') return "Master's Thesis";
      if (type === 'PhdThesis') return 'PhD Thesis';
      return type;
    }
    if (typeof type === 'object' && 'Unknown' in type) return type.Unknown;
    return 'Unknown';
  });

  const pagesString = $derived.by(() => {
    if (!entry.pages) return '';
    if (entry.pages.start === entry.pages.end) return entry.pages.start.toString();
    return `${entry.pages.start}-${entry.pages.end}`;
  });

  const bibtexLines = $derived(entry.source.split('\n'));

  const doiUrl = $derived(entry.doi ? `https://doi.org/${entry.doi}` : '');

  async function handleCopy() {
    try {
      await copyToClipboard(entry.source);
    } catch (e) {
      console.error('Copy failed:', e);
    }
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

<div class="space-y-2">
  <!-- Info Collapse -->
  <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
    <input type="checkbox" checked />
    <div class="collapse-title font-medium">Info</div>
    <div class="collapse-content">
      <table class="table table-sm">
        <tbody>
          <tr>
            <td class="text-right opacity-70 font-semibold">Type</td>
            <td>{typeLabel}</td>
          </tr>
          <tr>
            <td class="text-right opacity-70 font-semibold">Key</td>
            <td>{entry.cite_key}</td>
          </tr>
          <tr>
            <td class="text-right opacity-70 font-semibold">Title</td>
            <td>
              {#if entry.title}
                <ChunksComp chunks={entry.title} citeKey="{entry.cite_key}-drawer-title" />
              {/if}
            </td>
          </tr>
          {#if entry.author}
            {#each entry.author as author}
              <tr>
                <td class="text-right">Author</td>
                <td>{author}</td>
              </tr>
            {/each}
          {:else}
            <tr>
              <td class="text-right">Author</td>
              <td></td>
            </tr>
          {/if}
          {#if entry.full_journal}
            <tr>
              <td class="text-right">Journal</td>
              <td>{entry.full_journal}</td>
            </tr>
            <tr>
              <td class="text-right">Journal Abbr</td>
              <td>{entry.journal || ''}</td>
            </tr>
          {:else if entry.journal}
            <tr>
              <td class="text-right">Journal</td>
              <td>{entry.journal}</td>
            </tr>
          {/if}
          <tr>
            <td class="text-right">Volume</td>
            <td>{entry.volume || ''}</td>
          </tr>
          <tr>
            <td class="text-right">Number</td>
            <td>{entry.number || ''}</td>
          </tr>
          <tr>
            <td class="text-right">Pages</td>
            <td>{pagesString}</td>
          </tr>
          <tr>
            <td class="text-right">Year</td>
            <td>{entry.year || ''}</td>
          </tr>
          <tr>
            <td class="text-right">DOI</td>
            <td class="break-all">
              {#if entry.doi}
                <button
                  class="tooltip cursor-pointer text-left break-all hover:text-primary"
                  data-tip="在浏览器中打开"
                  onclick={handleOpenDoi}
                >
                  {entry.doi}
                </button>
              {/if}
            </td>
          </tr>
          <tr>
            <td class="text-right">URL</td>
            <td class="break-all">
              {#if entry.url}
                <button
                  class="tooltip cursor-pointer text-left break-all hover:text-primary"
                  data-tip="在浏览器中打开"
                  onclick={handleOpenUrl}
                >
                  {entry.url}
                </button>
              {/if}
            </td>
          </tr>
          <tr>
            <td class="text-right">File</td>
            <td>
              {#if entry.file}
                <button
                  class="tooltip cursor-pointer text-left break-all hover:text-primary"
                  data-tip="打开"
                  onclick={handleOpenFile}
                >
                  {entry.file}
                </button>
              {/if}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

  <!-- Abstract Collapse -->
  <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
    <input type="checkbox" />
    <div class="collapse-title font-medium">Abstract</div>
    <div class="collapse-content">
      {#if entry.abstract_}
        <ChunksComp chunks={entry.abstract_} citeKey="{entry.cite_key}-abstract" />
      {/if}
    </div>
  </div>

  <!-- Note Collapse -->
  <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
    <input type="checkbox" />
    <div class="collapse-title font-medium">Note</div>
    <div class="collapse-content">
      {#if entry.note}
        <ChunksComp chunks={entry.note} citeKey="{entry.cite_key}-note" />
      {/if}
    </div>
  </div>

  <!-- BibTeX Collapse -->
  <div class="collapse collapse-arrow bg-base-200/30 hover:bg-base-200/50 transition-colors rounded-box">
    <input type="checkbox" />
    <div class="collapse-title font-medium">BibTeX</div>
    <div class="collapse-content">
      {#each bibtexLines as line}
        <p class="font-mono text-xs">{line}</p>
      {/each}
    </div>
  </div>
</div>
