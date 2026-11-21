<script>
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import LicenseLink from "#lib/LicenseLink.svelte";
  import LoadState from "#lib/LoadState.svelte";
  import ReferencePanel from "#lib/ReferencePanel.svelte";
  import ReferenceTable from "#lib/ReferenceTable.svelte";
  import { useResponsivePanels } from "#lib/useResponsivePanels.svelte.js";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import { onMount, untrack } from "svelte";
  import LineChart from "../commodity_prices/LineChart.svelte";
  import Selectors from "./Selectors.svelte";
  import { fetchLookups, fetchSeries } from "./utils/fetchData.js";
  import { transformData } from "./utils/transformData.js";

  /** @typedef {{ code: string, name: string, level?: string, parentCode?: string }} Item */
  /** @typedef {{ code: string, name: string }} Region */
  /** @typedef {import("./utils/transformData.js").CpiSeries} CpiSeries */
  /** @typedef {{ status: "loading"|"loaded"|"error"|"empty", error?: string }} SeriesState */

  let items = $state.raw(/** @type {Item[] | null} */ (null));
  let regions = $state.raw(/** @type {Region[] | null} */ (null));
  let lookupError = $state("");
  let lookupLoading = $state(true);
  let selectedItemCodes = $state(/** @type {string[]} */ ([]));
  let selectedAreaCodes = $state(/** @type {string[]} */ ([]));
  let excludedCombinationKeys = $state(/** @type {string[]} */ ([]));
  let selectedMonth = $state(/** @type {string | null} */ (null));
  /** @type {"index" | "normalized"} */
  let view = $derived(selectedMonth === null ? "index" : "normalized");
  let seriesByKey = $state.raw(/** @type {Record<string, CpiSeries>} */ ({}));
  let seriesStates = $state(/** @type {Record<string, SeriesState>} */ ({}));
  /** @type {Record<string, Promise<CpiSeries | undefined>>} */
  let inFlightRequests = Object.create(null);
  const panels = useResponsivePanels(() => selectedCombinations.length);

  let selectedCombinations = $derived.by(() => {
    const itemOptions = items;
    const regionOptions = regions;
    if (!itemOptions || !regionOptions) return [];
    return selectedItemCodes
      .flatMap((itemCode) => {
        const item = itemOptions.find((entry) => entry.code === itemCode);
        if (!item) return [];
        return selectedAreaCodes.flatMap((areaCode) => {
          const region = regionOptions.find((entry) => entry.code === areaCode);
          return region
            ? [
                {
                  itemCode,
                  areaCode,
                  itemName: item.name,
                  areaName: region.name,
                  itemIndex: itemOptions.findIndex(
                    (entry) => entry.code === itemCode,
                  ),
                },
              ]
            : [];
        });
      })
      .filter(
        ({ itemCode, areaCode }) =>
          !excludedCombinationKeys.includes(`${itemCode}:${areaCode}`),
      );
  });
  let loadedSeries = $derived(
    selectedCombinations
      .map(({ itemCode, areaCode }) => seriesByKey[`${itemCode}:${areaCode}`])
      .filter(Boolean),
  );
  let allSelectedLoaded = $derived(
    selectedCombinations.length > 0 &&
      selectedCombinations.every(({ itemCode, areaCode }) =>
        ["loaded", "empty"].includes(
          seriesStates[`${itemCode}:${areaCode}`]?.status,
        ),
      ),
  );
  let hasSeriesError = $derived(
    selectedCombinations.some(
      ({ itemCode, areaCode }) =>
        seriesStates[`${itemCode}:${areaCode}`]?.status === "error",
    ),
  );
  let chartResult = $derived(
    allSelectedLoaded && selectedCombinations.length <= 40
      ? transformData(loadedSeries, view, selectedMonth ?? "")
      : { chartData: null, referenceMonth: null, availableMonths: [] },
  );
  let sliderMonths = $derived(chartResult.availableMonths.toReversed());
  let sliderMax = $derived(Math.max(0, sliderMonths.length - 1));
  let sliderIndex = $derived(
    chartResult.referenceMonth
      ? sliderMonths.indexOf(chartResult.referenceMonth)
      : sliderMax,
  );

  /** @param {string} month */
  function formatMonth(month) {
    const [year, number] = month.split("-");
    return `${year}年${Number(number)}月`;
  }

  async function loadLookups() {
    lookupLoading = true;
    lookupError = "";
    try {
      ({ items, regions } = await fetchLookups());
    } catch (cause) {
      lookupError =
        cause instanceof Error ? cause.message : "選択肢を取得できませんでした";
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
      request = fetchSeries({
        itemCode,
        areaCode,
        itemIndex: combination.itemIndex,
        itemCount: items?.length ?? 0,
      }).then((series) =>
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
        error:
          cause instanceof Error
            ? cause.message
            : "データを取得できませんでした",
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
    excludedCombinationKeys = [];
    seriesStates = {};
    selectedMonth = null;
  }

  /** @param {{ itemCode: string, areaCode: string }} combination */
  function handleRemove(combination) {
    const key = `${combination.itemCode}:${combination.areaCode}`;
    if (!excludedCombinationKeys.includes(key)) {
      excludedCombinationKeys = [...excludedCombinationKeys, key];
    }
  }

  const rows = [
    { label: "統計名", value: "2025年基準 消費者物価指数" },
    {
      label: "統計表ID",
      id: "0004052037",
      url: "https://www.e-stat.go.jp/dbview?sid=0004052037",
      linkLabel: "e-Statで統計表を見る",
    },
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
          <LoadState
            state="error"
            message="品目または地域の選択肢を取得できませんでした。再試行してください。"
            onretry={() => void loadLookups()}
          />
        {:else if items && regions}
          <section class="control-panel" aria-label="品目と地域">
            <details class="selector-panel" bind:open={panels.isSelectorOpen}>
              <summary>品目・地域を選択</summary>
              <div class="selector-panel-content">
                <Selectors
                  bind:selectedItemCodes
                  bind:selectedAreaCodes
                  bind:excludedCombinationKeys
                  {selectedCombinations}
                  {items}
                  {regions}
                />
                {#if selectedCombinations.length > 40}
                  <p class="error" role="alert">
                    選択数が多いため、40系列以下になるよう品目または地域を減らしてください。
                  </p>
                {/if}
              </div>
            </details>
          </section>
          <section class="comparison-panel" aria-label="選択中の系列">
            <details class="added-items" bind:open={panels.isComparisonOpen}>
              <summary
                ><h2>
                  選択中の系列 <span>{selectedCombinations.length}系列</span>
                </h2></summary
              >
              <fieldset class="selected-list">
                <legend>選択中の系列</legend>
                {#if selectedCombinations.length > 0}
                  <ul>
                    {#each selectedCombinations as item (`${item.itemCode}:${item.areaCode}`)}
                      {@const key = `${item.itemCode}:${item.areaCode}`}
                      <li>
                        <span>{item.itemName}【{item.areaName}】</span>
                        {#if seriesStates[key]?.status === "loading"}<span
                            class="muted"
                            role="status">取得中…</span
                          >
                        {:else if seriesStates[key]?.status === "error"}<span
                            class="error"
                            role="alert">{seriesStates[key].error}</span
                          ><button
                            type="button"
                            onclick={() => retrySeries(item)}>再試行</button
                          >
                        {:else if seriesStates[key]?.status === "empty"}<span
                            class="muted"
                            role="status">該当データなし</span
                          >{/if}
                        <button
                          type="button"
                          class="icon-button"
                          aria-label="{item.itemName}【{item.areaName}】を選択中一覧から削除"
                          onclick={() => handleRemove(item)}
                          ><img
                            src={deleteIcon}
                            alt=""
                            aria-hidden="true"
                          /></button
                        >
                      </li>
                    {/each}
                  </ul>
                {:else}<p class="empty-comparison">
                    品目と地域を選択してください。
                  </p>{/if}
              </fieldset>
              <button
                type="button"
                onclick={handleReset}
                class="reset-button"
                disabled={selectedCombinations.length === 0 && !selectedMonth}
                >すべて消去</button
              >
            </details>
          </section>
        {/if}

        <ReferencePanel title="データについて">
          <h3>表示について</h3>
          <p>
            統計の指数は、2025年平均を100として作られています（価格そのものの円額ではありません）。
          </p>
          <p>
            長期の推移には公式の接続指数を使っています。接続指数の品目構成とウエイトは各時期の基準に従います。
          </p>
          {#if view === "normalized"}
            <p>
              基準月を選ぶと、各系列の公表指数をその系列の基準月の指数で割り、100を掛けます。105なら基準月から約5%の上昇です。都市間では物価の変化率を比較できます。価格水準の差の比較には使えません。単月を起点にすると、季節性や一時的な値動きが比較に影響します。
            </p>
          {/if}
          <h3>出典</h3>
          <ReferenceTable {rows} />
        </ReferencePanel>
        <LicenseLink />
      </aside>

      <section class="chart-panel" aria-label="消費者物価指数の推移">
        <div class="reference-month">
          <label for="cpi-reference-month">基準月</label>
          <select
            id="cpi-reference-month"
            bind:value={selectedMonth}
            disabled={!allSelectedLoaded ||
              selectedCombinations.length > 40 ||
              loadedSeries.length === 0}
          >
            <option value={null}>2025年平均＝100（公表値）</option>
            {#if selectedMonth && !chartResult.availableMonths.includes(selectedMonth)}
              <option value={selectedMonth} disabled
                >{formatMonth(selectedMonth)}</option
              >
            {/if}
            {#each chartResult.availableMonths as month (month)}
              <option value={month}>{formatMonth(month)}</option>
            {/each}
          </select>
          {#if chartResult.referenceMonth}
            <span
              >{formatMonth(
                chartResult.referenceMonth,
              )}＝100（公表CPIを換算）</span
            >
          {/if}
          {#if sliderMonths.length > 1}
            <input
              class="reference-month-slider"
              type="range"
              aria-label="基準月をスライダーで選ぶ"
              aria-valuetext={formatMonth(sliderMonths[sliderIndex])}
              min={0}
              max={sliderMax}
              step={1}
              value={sliderIndex}
              oninput={(event) => {
                selectedMonth = sliderMonths[event.currentTarget.valueAsNumber];
              }}
            />
          {/if}
        </div>
        {#if selectedCombinations.length > 40}
          <div class="empty-chart">
            <h2>選択数を減らしてください</h2>
            <p>一度に表示できるのは40系列までです。</p>
          </div>
        {:else if selectedCombinations.length === 0}
          <div class="empty-chart">
            <h2>指数の推移を比較</h2>
            <p>品目と地域を選ぶと、ここにグラフが表示されます。</p>
          </div>
        {:else if !allSelectedLoaded}
          <div class="empty-chart">
            <h2>
              {hasSeriesError
                ? "取得できなかった系列があります"
                : "系列を読み込んでいます"}
            </h2>
            <p>選択中の系列の状態は一覧で確認できます。</p>
          </div>
        {:else if loadedSeries.length === 0}
          <div class="empty-chart">
            <h2>表示できる系列がありません</h2>
            <p>選択した品目と地域の組み合わせにはデータがありません。</p>
          </div>
        {:else if view === "normalized" && !chartResult.referenceMonth}
          <div class="empty-chart">
            {#if chartResult.availableMonths.length === 0}
              <h2>共通して値がある月がありません</h2>
            {/if}
            {#if selectedMonth}
              <p role="status">
                選んだ月に値がない系列があります。別の月を選んでください
              </p>
            {/if}
          </div>
        {:else if chartResult.chartData}
          <LineChart chartData={chartResult.chartData} />
        {:else}
          <div class="empty-chart">
            <h2>表示できる系列がありません</h2>
            <p>別の品目または地域を選んでください。</p>
          </div>
        {/if}
      </section>
    </div>
  </main>
</div>

<style>
  .chart-panel {
    --chart-height: calc(100dvh - 12rem);
  }
  .reference-month {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin: var(--space-4) 0 var(--space-2);
  }
  .reference-month select {
    min-height: 2.75rem;
    max-width: 100%;
  }
  .reference-month-slider {
    flex: 0 0 100%;
    min-width: 0;
    min-height: 2.75rem;
    margin: 0;
  }
  .empty-chart {
    min-height: 40vh;
    display: grid;
    align-content: center;
    justify-items: center;
    text-align: center;
  }
  .empty-chart h2 {
    margin-bottom: var(--space-2);
  }
  .selected-list ul {
    margin: 0;
    padding-left: 1.25rem;
  }
  .selected-list li {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) 0;
  }
  .selected-list li > span:first-child {
    flex: 1 1 12rem;
  }
  .selected-list li > .icon-button {
    margin-right: var(--space-3);
  }
  .reset-button {
    padding: var(--space-1) var(--space-2);
  }
  .error {
    color: var(--error-color, #a00);
  }
  @media (max-width: 50rem) {
    .chart-panel {
      --chart-height: 55vh;
    }
  }
</style>
