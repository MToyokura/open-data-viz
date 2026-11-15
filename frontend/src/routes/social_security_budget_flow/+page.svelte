<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import ReferenceTable from "#lib/ReferenceTable.svelte";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import LoadState from "#lib/LoadState.svelte";
  import { onMount } from "svelte";
  import SplitSankey from "./SplitSankey.svelte";
  import { fetchCatalog } from "./utils/fetchCatalog.js";

  /** @typedef {{id:string,label:string,group:string,value:number}} DiagramNode */
  /** @typedef {{id:string,name:string,income:DiagramNode[],expenditure:DiagramNode[]}} Program */
  /** @typedef {{programs:Program[]}} Catalog */
  const diagramNote =
    "収入を左側、支出を右側に分けて、項目ごとの金額を帯の太さで示しています。特定の収入の使途や、実際の資金移動、制度間の資金移転元・移転先を示すものではありません。";
  let catalog = $state.raw(/** @type {Catalog} */ ({ programs: [] }));
  let selectedId = $state("");
  let loading = $state(true);
  let loadError = $state("");
  const selected = $derived(
    catalog.programs.find((item) => item.id === selectedId),
  );
  const referenceRows = [
    { label: "統計名", value: "社会保障費用統計" },
    {
      label: "表題（収入）",
      value: "2024年度社会保障給付費収支表－収入（集計表2）",
    },
    {
      label: "統計表ID（収入）",
      id: "0004050702",
      url: "https://www.e-stat.go.jp/dbview?sid=0004050702",
      linkLabel: "収入の統計表をe-Statで見る",
    },
    {
      label: "表題（支出）",
      value: "2024年度社会保障給付費収支表－支出（集計表2）",
    },
    {
      label: "統計表ID（支出）",
      id: "0004050703",
      url: "https://www.e-stat.go.jp/dbview?sid=0004050703",
      linkLabel: "支出の統計表をe-Statで見る",
    },
    { label: "対象年／期間", value: "2024年度" },
  ];

  async function loadCatalog() {
    loading = true;
    loadError = "";
    try {
      const data = await fetchCatalog();
      catalog = data;
      selectedId = data.programs[0].id;
    } catch (error) {
      loadError =
        error instanceof Error
          ? error.message
          : "カタログを取得できませんでした。";
    } finally {
      loading = false;
    }
  }

  onMount(loadCatalog);
</script>

<svelte:head><title>社会保障給付費収支 | Open Data Viz</title></svelte:head>

<div class="visualization-page">
  <main class="page-content">
    <WorkspaceHeader title="社会保障給付費収支" />
    <div class="workspace">
      <section class="control-column" aria-label="制度の選択">
        {#if !loading && !loadError}
          <fieldset class="program-selector ui-panel">
            <legend>制度選択</legend>
            <label for="program">社会保障制度</label>
            <select id="program" bind:value={selectedId}>
              {#each catalog.programs as program (program.id)}
                <option value={program.id}>{program.name}</option>
              {/each}
            </select>
          </fieldset>
        {/if}
      </section>
      <aside class="details-column" aria-label="データ情報">
        <ReferencePanel title="データについて">
          <h3>利用上の注意</h3>
          <ul class="reference-notes">
            <li>
              2024年度社会保障給付費収支表に計上された制度別の収入・支出を、金額と分類で確認できます。
            </li>
            <li>{diagramNote}</li>
            <li>
              金額は元表の百万円単位で計算し、図中では概数で表示しています。総計・再掲の制度区分や、合計などの集計項目、ゼロ・特殊記号・負値は表示対象に含めていません。
            </li>
            <li>
              「表示項目計」は図に表示した項目の合計で、元表の合計とは異なります。収入と支出の表示項目計の差も、元表の「収支差」を示すものではありません。
            </li>
          </ul>
          <h3>出典</h3>
          <ReferenceTable rows={referenceRows} />
        </ReferencePanel>
        <LicenseLink />
      </aside>
      <section class="chart-panel" aria-label="社会保障給付費収支図">
        {#if loading}
          <LoadState state="loading" message="図のデータを読み込んでいます…" />
        {:else if loadError}
          <LoadState state="error" message={loadError} onretry={loadCatalog} />
        {:else if selected}
          <SplitSankey program={selected} />
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .workspace {
    --workspace-areas: "controls chart" "details chart";
    grid-template-rows: auto minmax(0, 1fr);
  }
  .control-column {
    grid-area: controls;
    display: grid;
    align-content: start;
    gap: var(--space-2);
    min-width: 0;
    padding: var(--space-4) var(--space-6);
    border-right: var(--border-default);
  }
  .program-selector {
    display: grid;
    gap: var(--space-2);
    min-inline-size: 0;
    margin: 0;
  }
  .program-selector legend {
    padding-inline: var(--space-1);
    font-weight: 600;
  }
  /* The two-column Sankey needs a wider viewport than the shared workspace layout. */
  @media (max-width: 80rem) {
    .visualization-page {
      height: auto;
      min-height: 100dvh;
    }
    .visualization-page > .page-content {
      display: block;
      height: auto;
      min-height: 0;
    }
    .visualization-page .workspace {
      display: flex;
      flex-direction: column;
      width: 100%;
      height: auto;
      min-height: 0;
    }
    .visualization-page .control-column {
      order: 0;
      padding: var(--space-4);
      border-right: 0;
      border-bottom: var(--border-default);
    }
    .visualization-page .chart-panel {
      order: 1;
      width: 100%;
      max-width: 100%;
      overflow: visible;
    }
    .visualization-page .details-column {
      order: 2;
      padding: var(--space-4);
      border-right: 0;
      border-bottom: 0;
      overflow: visible;
    }
  }
</style>
