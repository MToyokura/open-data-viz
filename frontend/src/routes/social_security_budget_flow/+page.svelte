<script>
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import LicenseLink from "#lib/LicenseLink.svelte";
  import { onMount } from "svelte";
  import { fetchCatalog } from "./utils/fetchCatalog.js";
  import SplitSankey from "./SplitSankey.svelte";

  /** @typedef {{id:string,label:string,group:string,value:number}} DiagramNode */
  /** @typedef {{id:string,name:string,income:DiagramNode[],expenditure:DiagramNode[],note:string}} Program */
  /** @typedef {{programs:Program[]}} Catalog */
  let catalog = $state.raw(/** @type {Catalog} */ ({ programs: [] }));
  let selectedId = $state("");
  let loading = $state(true);
  let loadError = $state("");
  const selected = $derived(catalog.programs.find((item) => item.id === selectedId));

  onMount(() => {
    fetchCatalog().then((data) => {
      catalog = data;
      selectedId = data.programs[0].id;
    }).catch((error) => {
      loadError = error instanceof Error ? error.message : "カタログを取得できませんでした。";
    }).finally(() => { loading = false; });
  });

</script>

<svelte:head><title>社会保障制度の財源・支出 | Open Data Viz</title></svelte:head>

<div class="page-shell">
  <main class="page-content">
    <WorkspaceHeader title="社会保障制度の財源・支出" />
    <p>2024年度社会保障給付費収支表に計上された制度別の収入・支出を、金額と分類で確認できます。</p>
    {#if loading}<p role="status">図のデータを読み込んでいます…</p>
    {:else if loadError}<p role="alert">{loadError} ページを再読み込みしてください。</p>
    {:else}
    <section class="controls" aria-label="制度の選択">
      <label for="program">社会保障制度</label>
      <select id="program" bind:value={selectedId}>
        {#each catalog.programs as program (program.id)}
          <option value={program.id}>{program.name}</option>
        {/each}
      </select>
    </section>
    {#if selected}
      <SplitSankey program={selected} />
      <details><summary>この図について</summary><p>{selected.note}</p></details>
    {/if}
    {/if}
    <ReferencePanel title="データについて">
      <h3>出典</h3>
      <ul>
        <li><a href="https://www.e-stat.go.jp/dbview?sid=0004050702" target="_blank" rel="noreferrer">2024年度社会保障給付費収支表－収入（e-Stat）</a></li>
        <li><a href="https://www.e-stat.go.jp/dbview?sid=0004050703" target="_blank" rel="noreferrer">2024年度社会保障給付費収支表－支出（e-Stat）</a></li>
      </ul>
      <p>金額は元表の百万円単位で計算しています。正の計上値を確認した項目を図示し、特殊記号や保留中の負値は表示対象に含めていません。</p>
    </ReferencePanel>
    <LicenseLink />
  </main>
</div>

<style>
  .controls { display: grid; gap: var(--space-2); max-width: 32rem; margin: var(--space-5) 0; }
  select { padding: var(--space-2); font: inherit; }
</style>
