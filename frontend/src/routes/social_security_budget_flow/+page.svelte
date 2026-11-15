<script>
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import LicenseLink from "#lib/LicenseLink.svelte";
  import { onMount } from "svelte";
  import { fetchCatalog } from "./utils/fetchCatalog.js";
  import SplitSankey from "./SplitSankey.svelte";

  /** @typedef {{id:string,label:string,group:string,value:number}} DiagramNode */
  /** @typedef {{id:string,name:string,income:DiagramNode[],expenditure:DiagramNode[]}} Program */
  /** @typedef {{programs:Program[]}} Catalog */
  const diagramNote = "本図は、e-Stat「2024年度社会保障給付費収支表」において、各制度に計上された収入・支出の項目と金額を示しています。矢印は収入項目・制度・支出項目を整理したもので、特定の収入と個別支出の対応、実際の資金移動、制度間移転の相手先を示すものではありません。";
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

<div class="visualization-page visualization-page--chart-first">
  <main class="page-content">
    <WorkspaceHeader title="社会保障制度の財源・支出" />
    <div class="workspace">
      <aside class="details-column" aria-label="制度の選択とデータ情報">
        <p>2024年度社会保障給付費収支表に計上された制度別の収入・支出を、金額と分類で確認できます。</p>
        {#if !loading && !loadError}
          <section class="controls" aria-label="制度の選択">
            <label for="program">社会保障制度</label>
            <select id="program" bind:value={selectedId}>
              {#each catalog.programs as program (program.id)}
                <option value={program.id}>{program.name}</option>
              {/each}
            </select>
          </section>
        {/if}
        <ReferencePanel title="データについて">
          {#if selected}
            <h3>この図について</h3>
            <p>{diagramNote}</p>
          {/if}
          <h3>出典</h3>
          <ul>
            <li><a href="https://www.e-stat.go.jp/dbview?sid=0004050702" target="_blank" rel="noreferrer">2024年度社会保障給付費収支表－収入（e-Stat）</a></li>
            <li><a href="https://www.e-stat.go.jp/dbview?sid=0004050703" target="_blank" rel="noreferrer">2024年度社会保障給付費収支表－支出（e-Stat）</a></li>
          </ul>
          <p>金額は元表の百万円単位で計算しています。正の計上値を確認した項目を図示し、特殊記号や保留中の負値は表示対象に含めていません。</p>
        </ReferencePanel>
        <LicenseLink />
      </aside>
      <section class="chart-panel" aria-label="社会保障制度の財源・支出図">
        {#if loading}
          <p role="status">図のデータを読み込んでいます…</p>
        {:else if loadError}
          <p role="alert">{loadError} ページを再読み込みしてください。</p>
        {:else if selected}
          <SplitSankey program={selected} />
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .controls { display: grid; gap: var(--space-2); margin: var(--space-5) 0; }
</style>
