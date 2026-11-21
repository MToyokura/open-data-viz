<script>
  import homeIcon from "#lib/assets/icons/home.svg";
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  let { title, asLabel = false } = $props();
</script>

<header class="page-heading">
  <a href={`${VITE_ROOT_PATH}/`} aria-label="ホームに戻る">
    <img src={homeIcon} alt="" />
  </a>
  {#if asLabel}
    <span class="page-heading-title">{title}</span>
  {:else}
    <h1 class="page-heading-title">{title}</h1>
  {/if}
</header>

<style>
  .page-heading {
    display: grid;
    grid-template-columns: 2rem minmax(0, 1fr) 2rem;
    gap: var(--space-2);
    align-items: center;
    padding: var(--space-4) var(--space-6);
    border-bottom: var(--border-default);
  }

  a {
    grid-column: 1;
    grid-row: 1;
    justify-self: start;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
    min-height: 2rem;
    color: inherit;
  }

  img {
    display: block;
    width: 24px;
    height: 24px;
  }

  .page-heading-title {
    grid-column: 2;
    grid-row: 1;
    min-width: 0;
    justify-self: stretch;
    margin: 0;
    font-family: inherit;
    font-size: 1.25rem;
    font-weight: 700;
    text-align: center;
    overflow-wrap: anywhere;
  }

  /* Matches the shared workspace column switch. */
  @media (max-width: 50rem) {
    .page-heading {
      padding: var(--space-3) var(--space-4);
    }

    .page-heading-title {
      font-size: 1.1rem;
    }
  }
</style>
