<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import { onMount } from "svelte";
  import SplitSankey from "./SplitSankey.svelte";
  import { fetchCatalog } from "./utils/fetchCatalog.js";

  /** @typedef {{id:string,label:string,group:string,value:number}} DiagramNode */
  /** @typedef {{id:string,name:string,income:DiagramNode[],expenditure:DiagramNode[]}} Program */
  /** @typedef {{programs:Program[]}} Catalog */
  const diagramNote =
    "本図は、e-Stat「2024年度社会保障給付費収支表」において、各制度に計上された収入・支出の項目と金額を示しています。矢印は収入項目・制度・支出項目を整理したもので、特定の収入と個別支出の対応、実際の資金移動、制度間移転の相手先を示すものではありません。";
  let catalog = $state.raw(/** @type {Catalog} */ ({ programs: [] }));
  let selectedId = $state("");
  let loading = $state(true);
  let loadError = $state("");
  const selected = $derived(
    catalog.programs.find((item) => item.id === selectedId),
  );

  onMount(() => {
    fetchCatalog()
      .then((data) => {
        catalog = data;
        selectedId = data.programs[0].id;
      })
      .catch((error) => {
        loadError =
          error instanceof Error
            ? error.message
            : "カタログを取得できませんでした。";
      })
      .finally(() => {
        loading = false;
      });
  });
</script>

<svelte:head
  ><title>社会保障制度の財源・支出 | Open Data Viz</title></svelte:head
>

<div class="visualization-page">
  <main class="page-content">
    <WorkspaceHeader title="社会保障制度の財源・支出" />
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
              金額は元表の百万円単位で計算しています。正の計上値を確認した項目を図示し、特殊記号や保留中の負値は表示対象に含めていません。
            </li>
          </ul>
          <h3>出典</h3>
          <table class="reference-table ui-table">
            <tbody>
              <tr>
                <th>統計名</th>
                <td>社会保障費用統計</td>
              </tr>
              <tr>
                <th>表題</th>
                <td>
                  <div>2024年度社会保障給付費収支表－収入（集計表2）</div>
                  <div>2024年度社会保障給付費収支表－支出（集計表2）</div>
                </td>
              </tr>
              <tr>
                <th>統計表ID</th>
                <td>
                  <div>収入: 0004050702</div>
                  <div>
                    <a
                      href="https://www.e-stat.go.jp/dbview?sid=0004050702"
                      target="_blank"
                      rel="noopener noreferrer">e-Statで統計表を見る</a
                    >
                  </div>
                  <div>支出: 0004050703</div>
                  <div>
                    <a
                      href="https://www.e-stat.go.jp/dbview?sid=0004050703"
                      target="_blank"
                      rel="noopener noreferrer">e-Statで統計表を見る</a
                    >
                  </div>
                </td>
              </tr>
              <tr>
                <th>対象年／期間</th>
                <td>2024年度</td>
              </tr>
            </tbody>
          </table>
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
