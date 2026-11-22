<script lang="ts">
  import katex from "katex";
  import "katex/dist/katex.min.css";

  let { tex, display = false }: { tex: string; display?: boolean } = $props();
  const markup = $derived(
    katex.renderToString(tex, {
      displayMode: display,
      trust: false,
      macros: display ? { "\\arraystretch": "2" } : undefined,
    }),
  );
</script>

<span class={display ? "display" : undefined}>
  <!-- eslint-disable-next-line svelte/no-at-html-tags -- KaTeX generates this markup from article formulas with trust disabled. -->
  {@html markup}
</span>

<style>
  .display {
    display: block;
    overflow-x: auto;
  }

  .display :global(.katex-display) {
    min-inline-size: max-content;
    margin-block: 0.5rem;
  }
</style>
