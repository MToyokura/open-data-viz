<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import homeIcon from "#lib/assets/icons/home.svg";
  import { page } from "$app/state";
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  let { children } = $props();
  const isRelay = $derived(
    page.url.pathname.includes("/vegetable_production_relay"),
  );
  const isWorkspace = $derived(
    isRelay ||
      page.url.pathname.includes("/commodity_prices") ||
      page.url.pathname.includes("/asset_distribution") ||
      page.url.pathname.includes(
        "/workers_by_age_employment_type_education_sex",
      ),
  );
  const isHome = $derived(
    page.url.pathname.replace(/\/+$/, "") ===
      VITE_ROOT_PATH.replace(/\/+$/, ""),
  );
</script>

<svelte:head>
  <title>Open Data Viz</title>
  <meta
    name="description"
    content="データの加工から可視化までのすべての工程を公開して、自由に利用できるようにすることをコンセプトに趣味で始めたプロジェクトです"
  />
</svelte:head>

{#if !isWorkspace && !isHome}
  <nav class="site-nav">
    <a href={`${VITE_ROOT_PATH}/`} aria-label="Home">
      <img src={homeIcon} alt="" />
    </a>
  </nav>
{/if}

<div id="svelte-main">
  {@render children()}
</div>

{#if !isWorkspace}
  <footer class="site-footer">
    <LicenseLink variant="footer" />
  </footer>
{/if}

<style>
  .site-nav {
    padding: var(--space-3) var(--page-content-padding);
  }

  .site-nav img {
    display: block;
    width: 24px;
    height: 24px;
  }

  .site-footer {
    display: flex;
    justify-content: end;
    margin: var(--space-8) var(--space-4) var(--space-4);
  }
</style>
