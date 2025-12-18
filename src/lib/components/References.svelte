<script lang="ts">
  import { currentReferences } from '$lib/stores/state';
  import { searchReferences, searchByField } from '$lib/tauri';
  import type { Reference, FilterField, FilterType } from '$lib/types';
  import ReferenceCard from './reference/ReferenceCard.svelte';

  let query = $state('');
  let isSearching = $state(false);
  let searchResult = $state<Reference[]>([]);
  let filterField = $state<FilterField>('All');
  let filterType = $state<FilterType>('All');

  const allRefs = $derived($currentReferences || []);
  const totalNum = $derived(allRefs.length);

  // Filter by type
  const filteredByType = $derived.by(() => {
    if (filterType === 'All') return allRefs;
    return allRefs.filter(ref => {
      const type = typeof ref.type_ === 'string' ? ref.type_ : 'Unknown';
      switch (filterType) {
        case 'Book': return type === 'Book';
        case 'Article': return type === 'Article';
        case 'Thesis': return type === 'Thesis' || type === 'MastersThesis' || type === 'PhdThesis';
        case 'TechReport': return type === 'TechReport';
        case 'Misc': return type === 'Misc';
        case 'Booklet': return type === 'Booklet';
        case 'InBook': return type === 'InBook';
        case 'InCollection': return type === 'InCollection';
        case 'InProceedings': return type === 'InProceedings';
        default: return true;
      }
    });
  });

  const displayRefs = $derived(isSearching ? searchResult : filteredByType);

  const showType = $derived.by(() => {
    switch (filterType) {
      case 'All': return 'References';
      case 'Article': return 'Articles';
      case 'Book': return 'Books';
      case 'Thesis': return 'Thesis';
      case 'TechReport': return 'TechReports';
      case 'Misc': return 'Misc';
      case 'Booklet': return 'Booklets';
      case 'InBook': return 'InBooks';
      case 'InCollection': return 'InCollections';
      case 'InProceedings': return 'InProceedings';
      default: return 'References';
    }
  });

  async function handleSearch(e: Event) {
    const target = e.target as HTMLInputElement;
    query = target.value;

    if (query.trim() === '') {
      isSearching = false;
      searchResult = [];
      return;
    }

    isSearching = true;
    try {
      if (filterField === 'All') {
        searchResult = await searchReferences(filteredByType, query);
      } else {
        searchResult = await searchByField(filteredByType, query, filterField);
      }
    } catch (e) {
      console.error('Search failed:', e);
      searchResult = [];
    }
  }

  const filterTypes: FilterType[] = ['All', 'Article', 'Book', 'Thesis', 'TechReport', 'Misc', 'Booklet', 'InBook', 'InCollection', 'InProceedings'];
  const filterFields: FilterField[] = ['All', 'Author', 'Title', 'Journal', 'Year'];
</script>

<div class="flex flex-col h-full overflow-hidden">
  <!-- Fixed search bar at top -->
  <div class="shrink-0 p-4 bg-base-100 border-b border-base-300 overflow-hidden">
    <div class="join w-full max-w-full overflow-hidden">
      <!-- Type Filter -->
      <select
        class="select select-bordered join-item"
        bind:value={filterType}
      >
        {#each filterTypes as type}
          <option value={type}>{type === 'All' ? 'Type' : type}</option>
        {/each}
      </select>

      <!-- Field Filter -->
      <select
        class="select select-bordered join-item"
        bind:value={filterField}
      >
        {#each filterFields as field}
          <option value={field}>{field === 'All' ? 'Field' : field}</option>
        {/each}
      </select>

      <!-- Search Input -->
      <input
        type="search"
        class="input input-primary join-item flex-1 min-w-0"
        placeholder="搜索文献..."
        value={query}
        oninput={handleSearch}
      />
    </div>
  </div>

  <!-- Scrollable content area -->
  <div class="flex-1 overflow-y-auto overflow-x-hidden">
    <h2 class="text-lg p-2">
      {showType} ({displayRefs.length}/{totalNum})
    </h2>

    {#if !isSearching}
      {#each filteredByType as entry (entry.cite_key)}
        <ReferenceCard {entry} />
      {/each}
    {:else}
      {#if searchResult.length > 0}
        {#each searchResult as entry (entry.cite_key)}
          <ReferenceCard {entry} />
        {/each}
      {:else}
        <p class="p-2 text-lg text-red-500">No results</p>
      {/if}
    {/if}
  </div>
</div>
