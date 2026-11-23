<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import { page } from "$app/state";
  import { VITE_ROOT_PATH } from "#lib/paths.js";

  let { children } = $props();
  const isRelay = $derived(
    page.url.pathname.includes("/vegetable_production_relay"),
  );
  const isWorkspace = $derived(
    isRelay ||
      page.url.pathname.includes("/commodity_prices") ||
      page.url.pathname.includes("/consumer_price_index") ||
      page.url.pathname.includes("/marine_fishery_catch") ||
      page.url.pathname.includes("/asset_distribution") ||
      page.url.pathname.includes("/social_security_budget_flow") ||
      page.url.pathname.includes(
        "/workers_by_age_employment_type_education_sex",
      ),
  );
  const isHome = $derived(
    page.url.pathname.replace(/\/+$/, "") ===
      VITE_ROOT_PATH.replace(/\/+$/, ""),
  );
  const isArticlePage = $derived(
    page.route.id?.startsWith("/articles/") === true,
  );
</script>

<svelte:head>
  <title>Open Data Viz</title>
  <meta
    name="description"
    content="データの加工から可視化までのすべての工程を公開して、自由に利用できるようにすることをコンセプトに趣味で始めたプロジェクトです"
  />
</svelte:head>

{#if !isWorkspace && !isArticlePage && !isHome}
  <nav class="page-heading">
    <a class="wordmark" href={`${VITE_ROOT_PATH}/`} aria-label="ホームに戻る">
      Open Data Viz
    </a>
  </nav>
{/if}

<div id="svelte-main">
  {@render children()}
</div>

{#if !isWorkspace && !isArticlePage && !isHome}
  <footer class="site-footer">
    <LicenseLink variant="footer" />
  </footer>
{/if}
