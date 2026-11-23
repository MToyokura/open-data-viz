<script>
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  let { title, asLabel = false } = $props();
</script>

<header class="page-heading">
  <a class="wordmark" href={`${VITE_ROOT_PATH}/`} aria-label="ホームに戻る">
    Open Data Viz
  </a>
  {#if asLabel}
    <span class="page-heading-title">{title}</span>
  {:else}
    <h1 class="page-heading-title">{title}</h1>
  {/if}
</header>
