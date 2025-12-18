<script lang="ts">
  import katex from 'katex';

  interface Props {
    content: string;
  }

  let { content }: Props = $props();

  let html = $derived.by(() => {
    try {
      return katex.renderToString(content, {
        output: 'mathml',
        throwOnError: false
      });
    } catch {
      return content;
    }
  });
</script>

<span class="inline-math">{@html html}</span>
