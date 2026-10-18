<script>
  /** @type {{ title: string, marginTop?: string, children: import("svelte").Snippet }} */
  let { title, marginTop = "var(--space-4)", children } = $props();
</script>

<section
  class="reference ui-panel ui-panel--subtle"
  style:margin-top={marginTop}
>
  {#if title}<h2>{title}</h2>{/if}
  {@render children()}
</section>

<style>
  .reference h2 {
    margin: 0 0 var(--space-4);
    font-size: 1.1rem;
  }

  :global(.reference-table) {
    margin-top: var(--space-4);
  }

  :global(.reference-table a) {
    word-break: break-all;
  }
</style>
