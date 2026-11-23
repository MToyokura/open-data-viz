<script lang="ts">
  let {
    id,
    title,
    level = "h2",
  }: { id: string; title: string; level?: "h1" | "h2" | "h3" } = $props();
</script>

<svelte:element
  this={level}
  {id}
  class={["article-heading", { "article-title": level === "h1" }]}
>
  {#if level === "h1"}
    {title}
  {:else}
    <a href="#{id}">
      {title}<span class="anchor-symbol" aria-hidden="true">#</span>
    </a>
  {/if}
</svelte:element>
