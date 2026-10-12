<script lang="ts">
  import deleteIcon from "#lib/assets/icons/delete.svg";
  import Spinner from "#lib/Spinner.svelte";
  import type { EChartsOption } from "echarts";
  import { untrack } from "svelte";
  import type { CommodityPrices } from "./constants/types";
  import LineChart from "./LineChart.svelte";
  import Reference from "./Reference.svelte";
  import Selectors from "./Selectors.svelte";
  import { fetchCommodityPrices } from "./utils/fetchCommodityPrices";
  import { transformData } from "./utils/transformData";
  import { useAreaLookup } from "./utils/useAreaLookup.svelte";
  import { useCat02Lookup } from "./utils/useCat02Lookup.svelte";

  const cat02LookupResult = useCat02Lookup();
  const areaLookupResult = useAreaLookup();

  let selectedCat02Code = $state<string>("");
  let selectedAreaCode = $state<string>("");
  let commodityPrices = $state<CommodityPrices[]>([]);
  let autoAddEnabled = $state<boolean>(true);
  let dataNotFound = $state<boolean>(false);

  let chartData = $derived<EChartsOption | null>(
    commodityPrices.length > 0 ? transformData(commodityPrices) : null,
  );

  // Fetch data when the selected area and cat02 code change
  $effect(() => {
    if (
      selectedCat02Code &&
      selectedAreaCode &&
      untrack(() => autoAddEnabled)
    ) {
      fetchAndAddData();
    }
  });

  async function fetchAndAddData() {
    if (selectedCat02Code && selectedAreaCode) {
      const data = await fetchCommodityPrices({
        cat02Code: selectedCat02Code,
        areaCode: selectedAreaCode,
      });

      if (!data) {
        dataNotFound = true;
        return;
      }

      if (data) {
        dataNotFound = false;
        commodityPrices.push(data);
      }
    }
  }

  function handleClear() {
    commodityPrices = [];
    dataNotFound = false;
  }

  function handleRemove(dataToRemove: CommodityPrices) {
    commodityPrices = commodityPrices.filter((item) => item !== dataToRemove);
  }
</script>

<svelte:head>
  <title>主要品目の都市別小売価格 | Open Data Viz</title>
</svelte:head>

<div class="page-shell">
  <main class="page-content">
    <h1>主要品目の都市別小売価格</h1>
    <div>
      {#if cat02LookupResult.isLoading || areaLookupResult.isLoading}
        <Spinner />
      {:else if cat02LookupResult.error || areaLookupResult.error}
        <p>Error loading lookup data</p>
      {:else if cat02LookupResult.cat02Lookup && areaLookupResult.areaLookup}
        <Selectors
          bind:autoAddEnabled
          bind:selectedCat02Code
          bind:selectedAreaCode
          cat02Lookup={cat02LookupResult.cat02Lookup}
          areaLookup={areaLookupResult.areaLookup}
          {fetchAndAddData}
        />
        {#if dataNotFound}
          <p class="data-not-found">データが見つかりませんでした</p>
        {/if}
        {#if commodityPrices.length > 0}
          <section class="added-items" aria-label="追加済みのデータ">
            <h2>追加済み</h2>
            <ul>
              {#each commodityPrices as item (item)}
                <li>
                  <span>{item.cat02_name}【{item.area_name}】</span>
                  <button
                    type="button"
                    class="delete-button"
                    aria-label="{item.cat02_name}【{item.area_name}】を削除"
                    onclick={() => handleRemove(item)}
                    ><img src={deleteIcon} alt="" aria-hidden="true" /></button
                  >
                </li>
              {/each}
            </ul>
          </section>
        {/if}
        <button onclick={handleClear} class="reset-button">リセット</button>
      {/if}

      {#if chartData}
        <LineChart {chartData} />
      {/if}
    </div>

    <Reference />
  </main>
</div>

<style>
  .data-not-found {
    color: var(--danger-color);
  }

  .added-items {
    max-width: 32rem;
    margin-top: var(--space-4);
  }

  .added-items h2 {
    margin: 0 0 var(--space-2);
    font-size: 1rem;
  }

  .added-items ul {
    display: flex;
    flex-direction: column;
    max-height: 12rem;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    border: var(--border-default);
    border-radius: var(--radius-md);
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
</style>
