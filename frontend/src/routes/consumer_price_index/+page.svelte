<script>
  import { onMount, untrack } from "svelte";
  import { useResponsivePanels } from "#lib/useResponsivePanels.svelte.js";
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import LicenseLink from "#lib/LicenseLink.svelte";
  import LoadState from "#lib/LoadState.svelte";
  import LineChart from "../commodity_prices/LineChart.svelte";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import ReferenceTable from "#lib/ReferenceTable.svelte";
  import Selectors from "./Selectors.svelte";
  import { fetchLookups, fetchSeries } from "./utils/fetchData.js";
  import { transformData } from "./utils/transformData.js";

  /** @typedef {{ code: string, name: string, level?: string, parentCode?: string }} Item */
  /** @typedef {{ code: string, name: string }} Region */
  /** @typedef {{ itemCode: string, areaCode: string, itemName: string, areaName: string, timeValuePairs: [string, number][] }} CpiSeries */
  /** @typedef {{ status: "loading"|"loaded"|"error"|"empty", error?: string }} SeriesState */

  let items = $state.raw(/** @type {Item[] | null} */ (null));
  let regions = $state.raw(/** @type {Region[] | null} */ (null));
  let lookupError = $state("");
  let lookupLoading = $state(true);
  let selectedItemCodes = $state(/** @type {string[]} */ ([]));
  let selectedAreaCodes = $state(/** @type {string[]} */ ([]));
  let view = $state(/** @type {"index" | "normalized"} */ ("index"));
  let seriesByKey = $state.raw(/** @type {Record<string, CpiSeries>} */ ({}));
  let seriesStates = $state(/** @type {Record<string, SeriesState>} */ ({}));
  /** @type {Record<string, Promise<CpiSeries | undefined>>} */
  let inFlightRequests = Object.create(null);
  const panels = useResponsivePanels(() => selectedCombinations.length);

  let selectedCombinations = $derived.by(() => {
    if (!items || !regions) return [];
    return selectedItemCodes.flatMap((itemCode) => {
      const item = items.find((entry) => entry.code === itemCode);
      if (!item) return [];
      return selectedAreaCodes.flatMap((areaCode) => {
        const region = regions.find((entry) => entry.code === areaCode);
        return region
          ? [{
              itemCode,
              areaCode,
              itemName: item.name,
              areaName: region.name,
              itemIndex: items.findIndex((entry) => entry.code === itemCode),
            }]
          : [];
      });
    });
  });
  let loadedSeries = $derived(
    selectedCombinations.map(({ itemCode, areaCode }) => seriesByKey[`${itemCode}:${areaCode}`]).filter(Boolean),
  );
  let allSelectedLoaded = $derived(
    selectedCombinations.length > 0 && selectedCombinations.every(({ itemCode, areaCode }) =>
      ["loaded", "empty"].includes(seriesStates[`${itemCode}:${areaCode}`]?.status),
    ),
  );
  let chartResult = $derived(
    allSelectedLoaded ? transformData(loadedSeries, view) : { chartData: null, referenceMonth: null },
  );

  async function loadLookups() {
    lookupLoading = true;
    lookupError = "";
    try {
      ({ items, regions } = await fetchLookups());
    } catch (cause) {
      lookupError = cause instanceof Error ? cause.message : "選択肢を取得できませんでした";
    } finally {
      lookupLoading = false;
    }
  }

  onMount(() => {
    void loadLookups();
  });

  $effect(() => {
    const combinations = selectedCombinations;
    if (combinations.length === 0 || combinations.length > 40) return;
    untrack(() => {
      for (const combination of combinations) void loadCombination(combination);
    });
  });

  /** @param {{ itemCode: string, areaCode: string, itemName: string, areaName: string, itemIndex: number }} combination */
  async function loadCombination(combination) {
    const { itemCode, areaCode, itemName, areaName } = combination;
    const key = `${itemCode}:${areaCode}`;
    if (seriesByKey[key]) {
      seriesStates[key] = { status: "loaded" };
      return;
    }
    let request = inFlightRequests[key];
    if (!request) {
      seriesStates[key] = { status: "loading" };
      request = fetchSeries({ itemCode, areaCode, itemIndex: combination.itemIndex, itemCount: items?.length ?? 0 }).then((series) =>
        series ? { ...series, itemName, areaName } : undefined,
      );
      inFlightRequests[key] = request;
    }
    try {
      const series = await request;
      if (series) {
        seriesByKey = { ...seriesByKey, [key]: series };
        seriesStates[key] = { status: "loaded" };
      } else {
        seriesStates[key] = { status: "empty" };
      }
    } catch (cause) {
      seriesStates[key] = {
        status: "error",
        error: cause instanceof Error ? cause.message : "データを取得できませんでした",
      };
    } finally {
      if (inFlightRequests[key] === request) delete inFlightRequests[key];
    }
  }

  /** @param {{ itemCode: string, areaCode: string, itemName: string, areaName: string, itemIndex: number }} combination */
  function retrySeries(combination) {
    const key = `${combination.itemCode}:${combination.areaCode}`;
    delete seriesStates[key];
    void loadCombination(combination);
  }

  function handleReset() {
    selectedItemCodes = [];
    selectedAreaCodes = [];
    seriesStates = {};
  }

  /** @param {{ itemCode: string, areaCode: string }} combination */
  function handleRemove(combination) {
    if (selectedItemCodes.length === 1) {
      selectedAreaCodes = selectedAreaCodes.filter((code) => code !== combination.areaCode);
    } else {
      selectedItemCodes = selectedItemCodes.filter((code) => code !== combination.itemCode);
    }
  }

  const rows = [
    { label: "統計名", value: "2025年基準 消費者物価指数" },
    { label: "統計表ID", id: "0004052037", url: "https://www.e-stat.go.jp/dbview?sid=0004052037", linkLabel: "e-Statで統計表を見る" },
    { label: "単位", value: "指数（円で表す小売価格ではありません）" },
  ];

</script>

<svelte:head>
  <title>消費者物価指数 | Open Data Viz</title>
</svelte:head>

<div class="visualization-page visualization-page--chart-first">
  <main class="page-content">
    <WorkspaceHeader title="消費者物価指数" />
    <div class="workspace">
      <aside class="details-column" aria-label="比較条件">
        {#if lookupLoading}
          <LoadState state="loading" message="選択肢を読み込んでいます…" />
        {:else if lookupError}
          <LoadState state="error" message="品目または地域の選択肢を取得できませんでした。再試行してください。" onretry={() => void loadLookups()} />
        {:else if items && regions}
          <section class="control-panel" aria-label="品目と地域">
            <details class="selector-panel" bind:open={panels.isSelectorOpen}>
              <summary>品目・地域を選択</summary>
              <div class="selector-panel-content">
                <Selectors bind:selectedItemCodes bind:selectedAreaCodes {items} {regions} />
                {#if selectedCombinations.length > 40}
                  <p class="error" role="alert">選択数が多いため、40系列以下になるよう品目または地域を減らしてください。</p>
                {/if}
              </div>
            </details>
          </section>
          <section class="comparison-panel" aria-label="選択中の系列">
            <details class="added-items" bind:open={panels.isComparisonOpen}>
              <summary><h2>選択中の系列 <span>{selectedCombinations.length}系列</span></h2></summary>
              <fieldset class="selected-list">
                <legend>選択中の系列</legend>
                {#if selectedCombinations.length > 0}
                  <ul>
                    {#each selectedCombinations as item (`${item.itemCode}:${item.areaCode}`)}
                      {@const key = `${item.itemCode}:${item.areaCode}`}
                      <li>
                        <span>{item.itemName}【{item.areaName}】</span>
                        {#if seriesStates[key]?.status === "loading"}<span class="muted" role="status">取得中…</span>
                        {:else if seriesStates[key]?.status === "error"}<span class="error" role="alert">{seriesStates[key].error}</span><button type="button" onclick={() => retrySeries(item)}>再試行</button>
                        {:else if seriesStates[key]?.status === "empty"}<span class="muted" role="status">該当データなし</span>{/if}
                        <button type="button" class="icon-button" aria-label="{item.itemName}【{item.areaName}】を選択中一覧から削除" onclick={() => handleRemove(item)}><img src={deleteIcon} alt="" aria-hidden="true" /></button>
                      </li>
                    {/each}
                  </ul>
                {:else}<p class="empty-comparison">品目と地域を選択してください。</p>{/if}
              </fieldset>
              <button type="button" onclick={handleReset} class="reset-button" disabled={selectedCombinations.length === 0}>すべて消去</button>
            </details>
          </section>
        {/if}

        <ReferencePanel title="データについて">
          <h3>表示について</h3>
          <p>統計の指数は、2025年平均を100として作られています（価格そのものの円額ではありません）。</p>
          {#if view === "normalized"}
            <p>最新月基準では、選択した系列すべてにデータがある最も新しい月を比較の起点にし、その月の指数を各系列で100にそろえます。ほかの月は起点に対する変化を表します（110は10％高く、90は10％低い値です）。</p>
            {#if chartResult.referenceMonth}<p>比較の起点にした月：{chartResult.referenceMonth}</p>{/if}
          {/if}
          <h3>出典</h3>
          <ReferenceTable {rows} />
        </ReferencePanel>
        <LicenseLink />
      </aside>

      <section class="chart-panel" aria-label="消費者物価指数の推移">
        <div class="view-toggle" role="group" aria-label="指数の表示方法">
          <button type="button" aria-pressed={view === "index"} class={{ active: view === "index" }} onclick={() => (view = "index")}>2025年基準</button>
          <button type="button" aria-pressed={view === "normalized"} class={{ active: view === "normalized" }} onclick={() => (view = "normalized")}>最新月基準</button>
        </div>
        {#if selectedCombinations.length > 40}
          <div class="empty-chart"><h2>選択数を減らしてください</h2><p>一度に表示できるのは40系列までです。</p></div>
        {:else if selectedCombinations.length === 0}
          <div class="empty-chart"><h2>指数の推移を比較</h2><p>品目と地域を選ぶと、ここにグラフが表示されます。</p></div>
        {:else if !allSelectedLoaded}
          <div class="empty-chart"><h2>系列を読み込んでいます</h2><p>選択中の系列の状態は一覧で確認できます。</p></div>
        {:else if loadedSeries.length === 0}
          <div class="empty-chart"><h2>表示できる系列がありません</h2><p>選択した品目と地域の組み合わせにはデータがありません。</p></div>
        {:else if view === "normalized" && !chartResult.referenceMonth}
          <div class="empty-chart"><h2>共通して値がある月がありません</h2><p>選択した系列すべてに値がある月がないため、表示できません。</p></div>
        {:else if chartResult.chartData}
          <LineChart chartData={chartResult.chartData} />
        {:else}
          <div class="empty-chart"><h2>表示できる系列がありません</h2><p>別の品目または地域を選んでください。</p></div>
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .chart-panel { --chart-height: calc(100dvh - 12rem); }
  .view-toggle { display: flex; justify-content: flex-end; gap: var(--space-2); margin: var(--space-4) 0; }
  .view-toggle button { padding: var(--space-1) var(--space-3); border: var(--border-default); border-radius: var(--radius-md); background: var(--surface-color); cursor: pointer; }
  .view-toggle button.active { border-color: var(--focus-color); background: var(--surface-muted-color); font-weight: 600; }
  .empty-chart { min-height: 40vh; display: grid; align-content: center; justify-items: center; text-align: center; }
  .empty-chart h2 { margin-bottom: var(--space-2); }
  .selected-list ul { margin: 0; padding-left: 1.25rem; }
  .selected-list li { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-2); padding: var(--space-2) 0; }
  .selected-list li > span:first-child { flex: 1 1 12rem; }
  .selected-list li > .icon-button { margin-right: var(--space-3); }
  .reset-button { padding: var(--space-1) var(--space-2); }
  .error { color: var(--error-color, #a00); }
  @media (max-width: 50rem) { .chart-panel { --chart-height: 55vh; } }
</style>
