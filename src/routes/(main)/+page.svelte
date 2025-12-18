<script lang="ts">
  import Bibliographies from '$lib/components/Bibliographies.svelte';
  import AddBibliography from '$lib/components/AddBibliography.svelte';

  let showModal = $state(false);
</script>

<svelte:head>
  <title>BibCiTeX - 文献库管理</title>
</svelte:head>

<Bibliographies bind:showModal />

{#if showModal}
  <AddBibliography bind:show={showModal} />
{/if}
