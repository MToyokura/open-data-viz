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

  let selectedCat02Code = $state("");
  let selectedAreaCodesByCat02 = $state(
    /** @type {Record<string, string[]>} */ ({}),
  );
  let commodityPrices = $state(/** @type {CommodityPrices[]} */ ([]));
  let seriesStates = $state(
    /** @type {Record<string, { status: "loading"|"loaded"|"error"|"empty", error?: string }>} */
    ({}),
  );
  const panels = useResponsivePanels(() => selectedCombinations.length);
  let fetchGeneration = 0;
  /** @type {Record<string, Promise<CommodityPrices | undefined>>} */
  let inFlightRequests = Object.create(null);

  let selectedCommodityPrices = $derived.by(() => {
    const selectedSeries = Object.entries(selectedAreaCodesByCat02).flatMap(
      ([cat02Code, areaCodes]) =>
        areaCodes.map((areaCode) => `${cat02Code}:${areaCode}`),
    );
    return commodityPrices.filter((item) =>
      selectedSeries.includes(`${item.cat02_code}:${item.area_code}`),
    );
  });
  let chartData = $derived(
    selectedCommodityPrices.length > 0
      ? transformData(selectedCommodityPrices)
      : null,
  );
  let selectedCombinations = $derived.by(() => {
    const cat02Lookup = cat02LookupResult.cat02Lookup ?? [];
    const areaLookup = areaLookupResult.areaLookup ?? [];
    return Object.entries(selectedAreaCodesByCat02).flatMap(
      ([cat02Code, areaCodes]) => {
        const cat02 = cat02Lookup.find((item) => item.cat02_code === cat02Code);
        return areaCodes.flatMap((areaCode) => {
          const area = areaLookup.find((item) => item.area_code === areaCode);
          return cat02 && area
            ? [
                {
                  cat02Code,
                  areaCode,
                  cat02Name: cat02.cat02_name,
                  areaName: area.area_name,
                },
              ]
            : [];
        });
      },
    );
  });

  // Fetch data when the selected items and areas change
  $effect(() => {
    const selectedAreaCodes = selectedAreaCodesByCat02[selectedCat02Code] ?? [];
    const generation = ++fetchGeneration;

    if (selectedCat02Code && selectedAreaCodes.length > 0) {
      untrack(() => {
        void Promise.all(
          selectedAreaCodes.map((areaCode) =>
            loadSeries(selectedCat02Code, areaCode, generation),
          ),
        );
      });
    }
  });

  /**
   * @param {string} cat02Code
   * @param {string} areaCode
   * @param {number} generation
   */
  async function loadSeries(cat02Code, areaCode, generation = fetchGeneration) {
    const key = `${cat02Code}:${areaCode}`;
    if (
      commodityPrices.some(
        (item) => `${item.cat02_code}:${item.area_code}` === key,
      )
    ) {
      seriesStates[key] = { status: "loaded" };
      return;
    }

    let request = inFlightRequests[key];
    if (!request) {
      seriesStates[key] = { status: "loading" };
      request = fetchCommodityPrices({ cat02Code, areaCode });
      inFlightRequests[key] = request;
    }

    try {
      const data = await request;
      if (generation !== fetchGeneration) return;
      if (data) {
        const alreadyAdded = commodityPrices.some(
          (item) =>
            item.cat02_code === data.cat02_code &&
            item.area_code === data.area_code,
        );
        if (!alreadyAdded) commodityPrices.push(data);
        seriesStates[key] = { status: "loaded" };
      } else {
        seriesStates[key] = { status: "empty" };
      }
    } catch (cause) {
      if (generation !== fetchGeneration) return;
      seriesStates[key] = {
        status: "error",
        error:
          cause instanceof Error ? cause.message : "データの取得に失敗しました",
      };
    } finally {
      if (inFlightRequests[key] === request) delete inFlightRequests[key];
    }
  }

  /** @param {string} cat02Code @param {string} areaCode */
  function retrySeries(cat02Code, areaCode) {
    const key = `${cat02Code}:${areaCode}`;
    delete seriesStates[key];
    void loadSeries(cat02Code, areaCode);
  }

  function handleReset() {
    fetchGeneration += 1;
    inFlightRequests = Object.create(null);
    selectedAreaCodesByCat02 = {};
    commodityPrices = [];
    seriesStates = {};
  }

  /** @param {string} cat02Code @param {string} areaCode */
  function handleRemove(cat02Code, areaCode) {
    const selectedAreaCodes = selectedAreaCodesByCat02[cat02Code] ?? [];
    const remainingAreaCodes = selectedAreaCodes.filter(
      (code) => code !== areaCode,
    );
    selectedAreaCodesByCat02 = { ...selectedAreaCodesByCat02 };
    if (remainingAreaCodes.length > 0) {
      selectedAreaCodesByCat02[cat02Code] = remainingAreaCodes;
    } else {
      delete selectedAreaCodesByCat02[cat02Code];
    }
  }
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
                  bind:selectedCat02Code
                  bind:selectedAreaCodesByCat02
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
                disabled={selectedCombinations.length === 0}>すべて消去</button
              >
            </details>
          </section>
        {/if}
        <Reference />
        <LicenseLink />
      </aside>

      <section class="chart-panel" aria-label="価格推移グラフ">
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

  @media (max-width: 50rem) {
    .chart-panel {
      --chart-height: 55vh;
    }
  }
</style>
