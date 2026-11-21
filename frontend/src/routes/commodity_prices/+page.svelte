<script>
  import { useResponsivePanels } from "#lib/useResponsivePanels.svelte.js";
  import LicenseLink from "#lib/LicenseLink.svelte";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import { untrack } from "svelte";
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import LoadState from "#lib/LoadState.svelte";
  import LineChart from "./LineChart.svelte";
  import Reference from "./Reference.svelte";
  import Selectors from "./Selectors.svelte";
  import { fetchCommodityPrices } from "./utils/fetchCommodityPrices";
  import { transformData } from "./utils/transformData";
  import { useAreaLookup } from "./utils/useAreaLookup.svelte";
  import { useCat02Lookup } from "./utils/useCat02Lookup.svelte";
  /** @typedef {import("./constants/types.js").CommodityPrices} CommodityPrices */
  const cat02LookupResult = useCat02Lookup();
  const areaLookupResult = useAreaLookup();
  let selectedCat02Codes = $state(/** @type {string[]} */ ([]));
  let selectedAreaCodes = $state(/** @type {string[]} */ ([]));
  let excludedPairKeys = $state(/** @type {string[]} */ ([]));
  let rebaseEnabled = $state(false);
  let selectedMonth = $state(/** @type {string|null} */ (null));
  let commodityPricesByKey = $state.raw(/** @type {Record<string, CommodityPrices>} */ ({}));
  let seriesStates = $state(/** @type {Record<string, { status: "loading"|"loaded"|"error"|"empty", error?: string }>} */ ({}));
  const inFlightRequests = Object.create(null);
  const panels = useResponsivePanels(() => selectedCombinations.length, { mobileOpen: true });
  let selectedCombinations = $derived.by(() => {
    const products = cat02LookupResult.cat02Lookup ?? [];
    const areas = areaLookupResult.areaLookup ?? [];
    return selectedCat02Codes.flatMap((cat02Code) => selectedAreaCodes.flatMap((areaCode) => {
      const key = `${cat02Code}:${areaCode}`;
      const product = products.find((entry) => entry.cat02_code === cat02Code);
      const area = areas.find((entry) => entry.area_code === areaCode);
      return product && area && !excludedPairKeys.includes(key) ? [{ cat02Code, areaCode, cat02Name: product.cat02_name, areaName: area.area_name }] : [];
    }));
  });
  let activeKeys = $derived(selectedCombinations.map(({cat02Code, areaCode}) => `${cat02Code}:${areaCode}`));
  let selectedCommodityPrices = $derived(activeKeys.map((key) => commodityPricesByKey[key]).filter(Boolean));
  let allResolved = $derived(selectedCombinations.length > 0 && selectedCombinations.every(({cat02Code, areaCode}) => ["loaded", "empty"].includes(seriesStates[`${cat02Code}:${areaCode}`]?.status)));
  let hasError = $derived(selectedCombinations.some(({cat02Code, areaCode}) => seriesStates[`${cat02Code}:${areaCode}`]?.status === "error"));
  let result = $derived(allResolved ? transformData(selectedCommodityPrices, selectedMonth ?? "") : null);
  let chartData = $derived(result?.chartData ?? null);
  let availableMonths = $derived(result?.availableMonths ?? []);
  let effectiveMonth = $derived(result?.referenceMonth ?? null);
  let sliderMonths = $derived(availableMonths);
  let sliderIndex = $derived(effectiveMonth ? sliderMonths.indexOf(effectiveMonth) : Math.max(0, sliderMonths.length - 1));
  /** @param {string} month */
  function formatMonth(month) { const [year, number] = month.split("-"); return `${year}年${Number(number)}月`; }
  $effect(() => { const pairs = selectedCombinations; untrack(() => pairs.forEach((pair) => void loadSeries(pair))); });
  /** @param {{cat02Code:string, areaCode:string}} pair @param {boolean} [retry] */
  async function loadSeries(pair, retry = false) {
    const {cat02Code, areaCode} = pair; const key = `${cat02Code}:${areaCode}`;
    if (commodityPricesByKey[key] && !retry) { seriesStates[key] = {status:"loaded"}; return; }
    if (seriesStates[key]?.status === "empty" && !retry) return;
    let request = inFlightRequests[key];
    if (!request) { seriesStates[key] = {status:"loading"}; request = fetchCommodityPrices({cat02Code, areaCode}); inFlightRequests[key] = request; }
    try {
      const data = await request;
      if (data && data.time_value_pairs?.some(/** @param {{value:string|number}} entry */ (entry) => Number.isFinite(Number(entry.value)))) { commodityPricesByKey[key] = data; seriesStates[key] = {status:"loaded"}; }
      else seriesStates[key] = {status:"empty"};
    } catch (cause) { seriesStates[key] = {status:"error", error:cause instanceof Error ? cause.message : "データの取得に失敗しました"}; }
    finally { if (inFlightRequests[key] === request) delete inFlightRequests[key]; }
  }
  /** @param {string} cat02Code @param {string} areaCode */
  function retrySeries(cat02Code, areaCode) { delete seriesStates[`${cat02Code}:${areaCode}`]; void loadSeries({cat02Code, areaCode}, true); }
  function handleReset() { selectedCat02Codes = []; selectedAreaCodes = []; excludedPairKeys = []; selectedMonth = null; rebaseEnabled = false; }
  /** @param {Event & {currentTarget: HTMLInputElement}} event */
  function toggleRebase(event) {
    rebaseEnabled = event.currentTarget.checked;
    if (rebaseEnabled && availableMonths.length > 0 && !availableMonths.includes(selectedMonth ?? "")) selectedMonth = availableMonths.at(-1) ?? null;
    if (!rebaseEnabled) selectedMonth = null;
  }
  /** @param {string} cat02Code @param {string} areaCode */
  function handleRemove(cat02Code, areaCode) { const key = `${cat02Code}:${areaCode}`; if (!excludedPairKeys.includes(key)) excludedPairKeys = [...excludedPairKeys, key]; }
  /** @param {Event & {currentTarget: HTMLInputElement}} event */
  function changeSlider(event) { selectedMonth = sliderMonths[Number(event.currentTarget.value)] ?? null; }
</script>

<svelte:head>
  <title>主要品目の都市別小売価格 | Open Data Viz</title>
</svelte:head>

<div class="visualization-page visualization-page--chart-first">
  <main class="page-content">
    <WorkspaceHeader title="主要品目の都市別小売価格" />
    <div class="workspace">
      <aside class="details-column" aria-label="比較条件">
        {#if cat02LookupResult.isLoading || areaLookupResult.isLoading}
          <LoadState state="loading" message="選択肢を読み込んでいます…" />
        {:else if cat02LookupResult.error || areaLookupResult.error}
          <LoadState
            state="error"
            message="品目または地域の選択肢を取得できませんでした。再試行してください。"
            onretry={() => {
              void Promise.all([
                cat02LookupResult.retry(),
                areaLookupResult.retry(),
              ]);
            }}
          />
        {:else if cat02LookupResult.cat02Lookup && areaLookupResult.areaLookup}
          <section class="control-panel" aria-label="品目と地域">
            <details class="selector-panel" bind:open={panels.isSelectorOpen}>
              <summary>品目・地域を選択</summary>
              <div class="selector-panel-content">
                <Selectors
                  bind:selectedCat02Codes
                  bind:selectedAreaCodes
                  bind:excludedPairKeys
                  {selectedCombinations}
                  cat02Lookup={cat02LookupResult.cat02Lookup}
                  areaLookup={areaLookupResult.areaLookup}
                />
              </div>
            </details>
          </section>
          <section class="comparison-panel" aria-label="選択中の系列">
            <details class="added-items" bind:open={panels.isComparisonOpen}>
              <summary>
                <h2>
                  選択中の系列 <span>{selectedCombinations.length}系列</span>
                </h2>
              </summary>
              <fieldset class="selected-list">
                <legend>選択中の系列</legend>
                {#if selectedCombinations.length > 0}
                  <ul>
                    {#each selectedCombinations as item (`${item.cat02Code}:${item.areaCode}`)}
                      {@const key = `${item.cat02Code}:${item.areaCode}`}
                      <li>
                        <span>{item.cat02Name}【{item.areaName}】</span>
                        {#if seriesStates[key]?.status === "loading"}
                          <span class="muted" role="status">取得中…</span>
                        {:else if seriesStates[key]?.status === "error"}
                          <span class="error" role="alert">
                            {seriesStates[key].error ??
                              "データを取得できませんでした"}
                          </span>
                          <button
                            type="button"
                            onclick={() =>
                              retrySeries(item.cat02Code, item.areaCode)}
                          >
                            再試行
                          </button>
                        {:else if seriesStates[key]?.status === "empty"}
                          <span class="muted" role="status">該当データなし</span
                          >
                        {/if}
                        <button
                          type="button"
                          class="icon-button"
                          aria-label="{item.cat02Name}【{item.areaName}】を選択中一覧から削除"
                          onclick={() =>
                            handleRemove(item.cat02Code, item.areaCode)}
                          ><img
                            src={deleteIcon}
                            alt=""
                            aria-hidden="true"
                          /></button
                        >
                      </li>
                    {/each}
                  </ul>
                {:else}
                  <p class="empty-comparison">選択されていません。</p>
                {/if}
              </fieldset>
              <button
                onclick={handleReset}
                class="reset-button"
                disabled={selectedCat02Codes.length === 0 && selectedAreaCodes.length === 0 && excludedPairKeys.length === 0 && selectedMonth === null}>すべて消去</button
              >
            </details>
          </section>
        {/if}
        <Reference {rebaseEnabled} />
        <LicenseLink />
      </aside>

      <section class="chart-panel" aria-label="価格推移グラフ">
        <div class="reference-controls">
          <label class="rebase-toggle">
            <input type="checkbox" checked={rebaseEnabled} disabled={availableMonths.length === 0} onchange={toggleRebase} />
            基準月で指数化
          </label>
          {#if rebaseEnabled && selectedCombinations.length > 0 && allResolved && availableMonths.length > 0}
            <label>基準月
              <select bind:value={selectedMonth}>
                {#each availableMonths as month (month)}<option value={month}>{formatMonth(month)}（100）</option>{/each}
              </select>
            </label>
            {#if availableMonths.length > 1}
              <input aria-label="基準月を選択" type="range" min="0" max={sliderMonths.length - 1} value={sliderIndex} oninput={changeSlider} />
            {/if}
          {/if}
        </div>
        {#if rebaseEnabled && selectedMonth && !availableMonths.includes(selectedMonth)}<p class="muted">系列の選択が変わったため、選択中の基準月は利用できません。共通の月を選び直してください。</p>{/if}
        {#if selectedCombinations.length > 0 && !allResolved}
          <LoadState state="loading" message="価格データを読み込んでいます…" />
        {:else if hasError && selectedCommodityPrices.length === 0}
          <LoadState state="error" message="価格データを取得できませんでした。選択中一覧から再試行してください。" />
        {:else if selectedCombinations.length > 0 && selectedCommodityPrices.length === 0}
          <div class="empty-chart"><h2>該当データがありません</h2><p>選択中の品目と地域の組み合わせに価格データがありません。</p></div>
        {:else if hasError}
          <p class="muted">取得できない系列があります。利用可能な系列を表示しています。</p>
        {/if}
        {#if chartData}
          <LineChart {chartData} />
        {:else if selectedCombinations.length > 0}
          <div class="empty-chart">
            <h2>価格の推移を比較</h2>
            <p>選択した系列の状態は「選択中一覧」で確認できます。</p>
          </div>
        {:else}
          <div class="empty-chart">
            <h2>価格の推移を比較</h2>
            <p>品目と地域を選ぶと、ここにグラフが表示されます。</p>
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
  .reference-controls { display:flex; align-items:center; gap:1rem; margin-bottom:.75rem; }
  .reference-controls label { display:flex; align-items:center; gap:.5rem; }
  .reference-controls input[type="range"] { flex:1; }

  @media (max-width: 50rem) {
    .chart-panel {
      --chart-height: 55vh;
    }
  }
</style>
