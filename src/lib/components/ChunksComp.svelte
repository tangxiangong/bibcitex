<script lang="ts">
  import type { Chunk } from '$lib/types';
  import InlineMath from './InlineMath.svelte';

  interface Props {
    chunks: Chunk[];
    citeKey: string;
  }

  let { chunks = [], citeKey }: Props = $props();
</script>

{#each chunks as chunk, i (citeKey + '-' + i)}
  {#if 'Normal' in chunk}
    <span>{chunk.Normal}</span>
  {:else if 'Verbatim' in chunk}
    <span>{chunk.Verbatim}</span>
  {:else if 'Math' in chunk}
    <InlineMath content={chunk.Math} />
  {/if}
{/each}
