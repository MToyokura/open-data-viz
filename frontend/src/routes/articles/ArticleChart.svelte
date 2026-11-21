<script>
  import { onMount } from "svelte";
  import { fetchCommodityPrices } from "../commodity_prices/utils/fetchCommodityPrices.js";
  import { transformData } from "../commodity_prices/utils/transformData.js";

  /** @typedef {import("echarts").EChartsType} EChartsType */
  /** @typedef {import("echarts").EChartsOption} EChartsOption */

  /** @type {{ series: { cat02Code: string, areaCode: string }[], label: string, initialStartMonth?: string }} */
  let { series, label, initialStartMonth = "" } = $props();
  /** @type {HTMLDivElement | undefined} */
  let container;
  let chart = $state.raw(/** @type {EChartsType | null} */ (null));
  let chartData = $state.raw(/** @type {EChartsOption | null} */ (null));
  let status = $state("loading");

  onMount(() => {
    let disposed = false;
    /** @type {ResizeObserver | undefined} */
    let resizeObserver;

    Promise.all(series.map((item) => fetchCommodityPrices(item)))
      .then((results) => {
        if (disposed) return;
        const valid = results.filter((item) => item !== undefined);
        if (
          valid.length !== results.length ||
          valid.some((item) => !item.time_value_pairs?.length)
        ) {
          status = "empty";
          return;
        }
        chartData = transformData(valid, "", initialStartMonth).chartData;
        status = chartData ? "loaded" : "empty";
      })
      .catch(() => {
        if (!disposed) status = "error";
      });

    import("echarts")
      .then(({ init }) => {
        if (disposed || !container) return;
        chart = init(container);
        resizeObserver = new ResizeObserver(() => chart?.resize());
        resizeObserver.observe(container);
      })
      .catch(() => {
        if (!disposed) status = "error";
      });

    return () => {
      disposed = true;
      resizeObserver?.disconnect();
      chart?.dispose();
      chart = null;
    };
  });

  $effect(() => {
    if (chart && chartData) chart.setOption(chartData, { notMerge: true });
  });
</script>

<figure class="article-chart" aria-label={label}>
  <div class="chart-frame">
    <div
      bind:this={container}
      class="chart-canvas"
      role="img"
      aria-label={label}
    ></div>
    {#if status !== "loaded"}
      <div class="chart-state" role="status">
        {#if status === "loading"}
          データを読み込んでいます…
        {:else if status === "error"}
          データを取得できませんでした。
        {:else}
          表示できるデータがありません。
        {/if}
      </div>
    {/if}
  </div>
  <figcaption>{label}</figcaption>
</figure>
