<script lang="ts">
  import Spinner from "$lib/Spinner.svelte";
  import { type ChartData } from "chart.js";
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

  let chartData = $derived<ChartData | null>(
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
</script>

<main>
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
        <p style="color: #cc0000;">データが見つかりませんでした</p>
      {/if}
      <button onclick={handleClear} class="reset-button">リセット</button>
    {/if}

    {#if chartData}
      <LineChart {chartData} />
    {/if}
  </div>

  <Reference />
</main>

<style>
  .reset-button {
    display: block;
    margin-top: 1rem;
  }
</style>
