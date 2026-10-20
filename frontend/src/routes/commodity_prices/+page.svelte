<script>
  import LicenseLink from "#lib/LicenseLink.svelte";
  import WorkspaceHeader from "#lib/WorkspaceHeader.svelte";
  import { onMount, untrack } from "svelte";
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import Spinner from "#lib/Spinner.svelte";
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

<div class="visualization-page visualization-page--chart-first">
  <main class="page-content">
    <WorkspaceHeader title="主要品目の都市別小売価格" />
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
                          class="icon-button"
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
                >すべて消去</button
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
  .chart-panel {
    --chart-height: calc(100dvh - 12rem);
  }

  .data-not-found {
    color: var(--danger-color);
  }

  @media (max-width: 50rem) {
    .chart-panel {
      --chart-height: 55vh;
    }
  }
</style>
