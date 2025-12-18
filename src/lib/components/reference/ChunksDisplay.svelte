<script lang="ts">
  import type { Chunk } from '$lib/types';
  import katex from 'katex';

  interface Props {
    chunks: Chunk[];
  }

  let { chunks }: Props = $props();

  function renderMath(content: string): string {
    try {
      return katex.renderToString(content, {
        throwOnError: false,
        displayMode: false
      });
    } catch {
      return content;
    }
  }
</script>

<span>
  {#each chunks as chunk}
    {#if 'Math' in chunk}
      {@html renderMath(chunk.Math)}
    {:else if 'Normal' in chunk}
      {chunk.Normal}
    {:else if 'Verbatim' in chunk}
      {chunk.Verbatim}
    {/if}
  {/each}
</span>
