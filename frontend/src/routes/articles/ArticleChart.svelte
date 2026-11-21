<script>
  import { onMount } from "svelte";
  import { fetchCommodityPrices } from "../commodity_prices/utils/fetchCommodityPrices.js";
  import { transformData } from "../commodity_prices/utils/transformData.js";

  /** @typedef {import("echarts").EChartsType} EChartsType */
  /** @typedef {import("echarts").EChartsOption} EChartsOption */

  /** @type {{ series: { cat02Code: string, areaCode: string }[], label: string }} */
  let { series, label } = $props();
  let container;
  let chart = $state.raw(/** @type {EChartsType | null} */ (null));
  let chartData = $state.raw(/** @type {EChartsOption | null} */ (null));
  let status = $state("loading");

  onMount(() => {
    let disposed = false;
    let resizeObserver;

    Promise.all(series.map((item) => fetchCommodityPrices(item)))
      .then((results) => {
        if (disposed) return;
        const valid = results.filter((item) => item?.time_value_pairs?.length);
        if (valid.length !== results.length) {
          status = "empty";
          return;
        }
        chartData = transformData(valid).chartData;
        status = chartData ? "loaded" : "empty";
      })
      .catch(() => {
        if (!disposed) status = "error";
      });

    import("echarts")
      .then(({ init }) => {
        if (disposed) return;
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
    <div bind:this={container} class="chart-canvas" role="img" aria-label={label}></div>
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

<style>
  .article-chart {
    margin: 1.5rem 0 2rem;
  }

  .chart-frame {
    position: relative;
    border: 1px solid #d8dcdf;
    border-radius: 0.25rem;
    background: #fff;
  }

  .chart-canvas {
    width: 100%;
    height: 19rem;
  }

  .chart-state {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgb(255 255 255 / 82%);
    color: #667078;
    font-size: 0.9rem;
  }

  figcaption {
    margin-top: 0.65rem;
    color: #48515a;
    font-size: 0.85rem;
    text-align: center;
  }

  @media (max-width: 40rem) {
    .chart-canvas {
      height: 16rem;
    }
  }
</style>
