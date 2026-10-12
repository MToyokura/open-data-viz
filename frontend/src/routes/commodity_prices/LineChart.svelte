<script lang="ts">
  import { onMount } from "svelte";
  import type { EChartsType, EChartsOption } from "echarts";

  let { chartData }: { chartData: EChartsOption } = $props();

  let container: HTMLDivElement;
  let chart = $state.raw<EChartsType | null>(null);

  onMount(() => {
    let disposed = false;
    let resizeObserver: ResizeObserver | undefined;

    import("echarts")
      .then(({ init }) => {
        if (disposed) return;

        chart = init(container);
        resizeObserver = new ResizeObserver(() => chart?.resize());
        resizeObserver.observe(container);
      })
      .catch((error) => {
        console.error("Failed to load Apache ECharts:", error);
      });

    return () => {
      disposed = true;
      resizeObserver?.disconnect();
      chart?.dispose();
      chart = null;
    };
  });

  $effect(() => {
    const option = chartData;
    if (chart) chart.setOption(option, { notMerge: true });
  });
</script>

<div
  id="chart-container"
  bind:this={container}
  role="img"
  aria-label="価格の推移"
></div>

<style>
  #chart-container {
    width: auto;
    height: 60vh;
    min-height: 320px;
    margin: var(--space-8) 0;
    padding: var(--space-8) 0;
    border: var(--border-default);
    border-radius: var(--radius-md);
  }

  @media (max-width: 48rem) {
    #chart-container {
      height: 55vh;
      margin: var(--space-8) 0;
      padding: var(--space-4) 0;
    }
  }
</style>
