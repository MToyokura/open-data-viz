<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import { onMount, untrack } from "svelte";
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import Spinner from "#lib/Spinner.svelte";
  import { VITE_ROOT_PATH } from "#lib/paths.js";
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
  let dataNotFound = $state(false);
  let isMobile = $state(false);
  let isSelectorOpen = $state(true);
  let isComparisonOpen = $state(true);
  let fetchGeneration = 0;
  /** @type {Record<string, Promise<CommodityPrices | undefined>>} */
  let inFlightRequests = Object.create(null);

  onMount(() => {
    const mobileQuery = window.matchMedia("(max-width: 50rem)");
    const updateViewport = () => {
      isMobile = mobileQuery.matches;
      isSelectorOpen = !isMobile;
      isComparisonOpen = !isMobile || selectedCombinations.length === 0;
    };

    updateViewport();
    mobileQuery.addEventListener("change", updateViewport);
    return () => mobileQuery.removeEventListener("change", updateViewport);
  });

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
    dataNotFound = false;

    if (selectedCat02Code && selectedAreaCodes.length > 0) {
      untrack(() =>
        fetchAndAddData(selectedCat02Code, selectedAreaCodes, generation),
      );
    }
  });

  /**
   * @param {string} cat02Code
   * @param {string[]} selectedAreaCodes
   * @param {number} generation
   */
  async function fetchAndAddData(cat02Code, selectedAreaCodes, generation) {
    /** @type {Record<string, true>} */
    const cachedKeys = Object.create(null);
    commodityPrices.forEach((item) => {
      cachedKeys[`${item.cat02_code}:${item.area_code}`] = true;
    });
    const expectedResults = selectedAreaCodes.filter(
      (areaCode) => !cachedKeys[`${cat02Code}:${areaCode}`],
    ).length;
    const results = await Promise.all(
      selectedAreaCodes.map((areaCode) => {
        const key = `${cat02Code}:${areaCode}`;
        if (cachedKeys[key]) return Promise.resolve(undefined);

        let request = inFlightRequests[key];
        if (!request) {
          request = fetchCommodityPrices({ cat02Code, areaCode });
          inFlightRequests[key] = request;
          void request.then(() => {
            if (inFlightRequests[key] === request) delete inFlightRequests[key];
          });
        }
        return request;
      }),
    );

    if (generation !== fetchGeneration) return;

    const foundData = results.filter((data) => data !== undefined);
    dataNotFound = foundData.length < expectedResults;
    /** @type {Record<string, true>} */
    const existingKeys = Object.create(null);
    commodityPrices.forEach((item) => {
      existingKeys[`${item.cat02_code}:${item.area_code}`] = true;
    });
    commodityPrices.push(
      ...foundData.filter((item) => {
        const key = `${item.cat02_code}:${item.area_code}`;
        if (existingKeys[key]) return false;
        existingKeys[key] = true;
        return true;
      }),
    );
  }

  function handleReset() {
    fetchGeneration += 1;
    inFlightRequests = Object.create(null);
    selectedAreaCodesByCat02 = {};
    commodityPrices = [];
    dataNotFound = false;
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

<div class="page-shell">
  <main class="page-content">
    <header class="page-heading">
      <a class="home-link" href={`${VITE_ROOT_PATH}/`}>Home</a>
      <h1>主要品目の都市別小売価格</h1>
    </header>
    <div class="workspace">
      <aside class="details-column" aria-label="比較条件">
        {#if cat02LookupResult.isLoading || areaLookupResult.isLoading}
          <Spinner />
        {:else if cat02LookupResult.error || areaLookupResult.error}
          <p>Error loading lookup data</p>
        {:else if cat02LookupResult.cat02Lookup && areaLookupResult.areaLookup}
          <section class="control-panel" aria-label="品目と地域">
            <details class="selector-panel" bind:open={isSelectorOpen}>
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
          <section class="comparison-panel" aria-label="比較中のデータ">
            <details class="added-items" bind:open={isComparisonOpen}>
              <summary>
                <h2>比較中 <span>{selectedCombinations.length}系列</span></h2>
              </summary>
              <fieldset class="selected-list">
                <legend>選択中一覧</legend>
                {#if selectedCombinations.length > 0}
                  <ul>
                    {#each selectedCombinations as item (`${item.cat02Code}:${item.areaCode}`)}
                      <li>
                        <span>{item.cat02Name}【{item.areaName}】</span>
                        <button
                          type="button"
                          class="delete-button"
                          aria-label="{item.cat02Name}【{item.areaName}】を削除"
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
              <button onclick={handleReset} class="reset-button"
                >リセット</button
              >
              {#if dataNotFound}
                <p class="data-not-found">データが見つかりませんでした</p>
              {/if}
            </details>
          </section>
        {/if}
        <Reference />
        <LicenseLink />
      </aside>

      <section class="chart-panel" aria-label="価格推移グラフ">
        {#if chartData}
          <LineChart {chartData} />
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
  .page-shell {
    min-width: 0;
    height: 100dvh;
    min-height: 0;
    overflow-x: clip;
  }

  .page-content {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: 100%;
    min-width: 0;
    min-height: 0;
    max-width: none;
    margin: 0;
    padding: 0;
  }

  .page-heading {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: var(--space-4) var(--space-6);
    border-bottom: var(--border-default);
  }

  .page-heading h1 {
    grid-column: 2;
    grid-row: 1;
    justify-self: center;
    margin: 0;
    font-size: 1.25rem;
  }

  .home-link {
    grid-column: 1;
    grid-row: 1;
    justify-self: start;
    font-size: 0.8rem;
    color: inherit;
  }

  .workspace {
    display: grid;
    grid-template-columns: minmax(20rem, 0.42fr) minmax(0, 1fr);
    grid-template-areas: "details chart";
    width: 100%;
    min-width: 0;
    min-height: 0;
  }

  .details-column,
  .chart-panel {
    min-width: 0;
    min-height: 0;
    overflow: auto;
    overscroll-behavior: contain;
  }

  .details-column {
    grid-area: details;
    padding: var(--space-4) var(--space-6);
    border-right: var(--border-default);
  }

  .chart-panel {
    grid-area: chart;
    padding: var(--space-4);
    --chart-height: calc(100dvh - 12rem);
  }

  .control-panel,
  .comparison-panel {
    margin-bottom: var(--space-4);
  }

  .selector-panel > summary {
    display: none;
  }

  .selector-panel-content {
    display: block;
  }

  .empty-chart {
    display: grid;
    min-height: 24rem;
    place-content: center;
    padding: var(--space-6);
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-muted-color);
    text-align: center;
  }

  .empty-chart h2 {
    margin: 0 0 var(--space-2);
    font-size: 1.25rem;
  }

  .empty-chart p {
    margin: 0;
  }

  .data-not-found {
    color: var(--danger-color);
  }

  .added-items {
    margin: 0;
  }

  .added-items > summary {
    display: none;
  }

  .added-items h2 {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    margin: 0 0 var(--space-2);
    font-size: 1rem;
  }

  .added-items h2 span {
    color: #555;
    font-size: 0.875rem;
    font-weight: normal;
  }

  .empty-comparison {
    margin: 0;
    padding: var(--space-3);
    color: #555;
  }

  .selected-list {
    min-inline-size: 0;
    margin: 0;
    padding: 0;
    border: var(--border-default);
    border-radius: var(--radius-md);
  }

  .selected-list legend {
    margin-inline: var(--space-3);
    padding-inline: var(--space-1);
    font-weight: 600;
  }

  .added-items ul {
    display: flex;
    flex-direction: column;
    max-height: 12rem;
    margin: 0.25rem 0 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .added-items li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-2) var(--space-3);
    border-bottom: var(--border-default);
  }

  .added-items li:last-child {
    border-bottom: 0;
  }

  .delete-button {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    justify-content: center;
    padding: var(--space-1);
    border: 0;
    border-radius: var(--radius-round);
    background: transparent;
    cursor: pointer;
  }

  .delete-button:hover {
    background: var(--surface-hover-color);
  }

  .delete-button img {
    display: block;
    width: 24px;
    height: 24px;
  }

  .reset-button {
    display: block;
    margin-top: var(--space-4);
  }

  @media (max-width: 50rem) {
    .page-shell {
      height: auto;
    }

    .page-content {
      height: auto;
      min-height: 0;
      display: block;
    }

    .workspace {
      display: flex;
      flex-direction: column;
      width: 100%;
      min-width: 0;
    }

    .details-column,
    .chart-panel {
      overflow: visible;
    }

    .details-column {
      border-right: 0;
      border-bottom: var(--border-default);
      padding: var(--space-4);
    }

    .chart-panel {
      order: -1;
      width: 100%;
      max-width: 100%;
      box-sizing: border-box;
      overflow-x: clip;
      --chart-height: 55vh;
    }

    .selector-panel > summary {
      display: list-item;
      font-weight: 600;
      cursor: pointer;
    }

    .selector-panel-content {
      padding-top: var(--space-4);
    }

    .added-items > summary {
      display: list-item;
      cursor: pointer;
    }

    .added-items > summary h2 {
      display: inline-flex;
      margin-bottom: 0;
    }

    .added-items[open] > summary h2 {
      margin-bottom: var(--space-2);
    }

    .page-heading {
      padding: var(--space-3) var(--space-4);
    }

    .page-heading h1 {
      font-size: 1.1rem;
    }

    .empty-chart {
      min-height: 16rem;
    }
  }
</style>
