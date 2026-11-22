<script lang="ts">
  let {
    id,
    title,
    level = "h2",
  }: { id: string; title: string; level?: "h1" | "h2" } = $props();
</script>

<svelte:element
  this={level}
  {id}
  class={["article-heading", { "article-title": level === "h1" }]}
>
  <a href="#{id}">
    {title}<span class="anchor-symbol" aria-hidden="true">#</span>
  </a>
</svelte:element>

<style>
  .article-heading {
    scroll-margin-top: 8em;
  }

  a {
    color: inherit;
    text-decoration: none;
  }

  .anchor-symbol {
    visibility: hidden;
    margin-inline-start: 0.25em;
    color: var(--text-muted-color);
    font-size: 0.8em;
  }

  .article-heading:hover .anchor-symbol,
  a:focus-visible .anchor-symbol {
    visibility: visible;
  }
</style>
