<script>
  import { onMount } from "svelte";
  /** @typedef {import("echarts").EChartsType} EChartsType */
  /** @typedef {import("echarts").EChartsOption} EChartsOption */

  /** @type {{ chartData: EChartsOption }} */
  let { chartData } = $props();

  /** @type {HTMLDivElement} */
  let scrollContainer;
  /** @type {HTMLDivElement} */
  let container;
  let chart = $state.raw(/** @type {EChartsType | null} */ (null));
  let viewMode = $state(/** @type {"fit" | "scroll"} */ ("fit"));

  /** @param {"fit" | "scroll"} mode */
  function setViewMode(mode) {
    viewMode = mode;
    requestAnimationFrame(() => {
      if (mode === "fit") scrollContainer.scrollLeft = 0;
      chart?.resize();
    });
  }

  onMount(() => {
    let disposed = false;
    /** @type {ResizeObserver | undefined} */
    let resizeObserver;

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
    if (!chart) return;

    const currentZoom = chart.getOption()?.dataZoom;
    const nextZoom = Array.isArray(option.dataZoom)
      ? option.dataZoom
      : option.dataZoom
        ? [option.dataZoom]
        : [];
    const zoomState = Array.isArray(currentZoom) ? currentZoom : [];

    chart.setOption(
      {
        ...option,
        dataZoom: nextZoom.map((zoom, index) => ({
          ...zoom,
          ...(zoomState[index]?.start != null
            ? { start: zoomState[index].start }
            : {}),
          ...(zoomState[index]?.end != null
            ? { end: zoomState[index].end }
            : {}),
        })),
      },
      { notMerge: true },
    );
  });
</script>

<div class="chart-view-controls" role="group" aria-label="グラフの表示方法">
  <button
    type="button"
    class={[
      "chart-view-button",
      { "chart-view-button--active": viewMode === "fit" },
    ]}
    aria-pressed={viewMode === "fit"}
    onclick={() => setViewMode("fit")}>画面幅表示</button
  >
  <button
    type="button"
    class={[
      "chart-view-button",
      { "chart-view-button--active": viewMode === "scroll" },
    ]}
    aria-pressed={viewMode === "scroll"}
    onclick={() => setViewMode("scroll")}>スクロール表示</button
  >
</div>
<div
  class={[
    "chart-scroll-container",
    { "chart-scroll-container--fit": viewMode === "fit" },
    { "chart-scroll-container--scroll": viewMode === "scroll" },
  ]}
  bind:this={scrollContainer}
  role="region"
  aria-label={viewMode === "fit"
    ? "価格推移グラフ（画面幅表示）"
    : "価格推移グラフ。左右にスクロールできます"}
>
  <div
    id="chart-container"
    bind:this={container}
    role="img"
    aria-label="価格の推移"
  ></div>
</div>

<style>
  .chart-view-controls {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-4);
  }

  .chart-view-button {
    padding: var(--space-1) var(--space-3);
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-color);
    font-size: 0.875rem;
  }

  .chart-view-button--active {
    border-color: var(--focus-color);
    background: var(--surface-muted-color);
    font-weight: 600;
  }

  .chart-scroll-container {
    box-sizing: border-box;
    width: 100%;
    max-width: 100%;
    margin: var(--space-8) 0;
    overflow-x: auto;
    overflow-y: hidden;
    border: var(--border-default);
    border-radius: var(--radius-md);
    -webkit-overflow-scrolling: touch;
  }

  .chart-scroll-container--fit {
    overflow-x: hidden;
  }

  .chart-scroll-container--scroll #chart-container {
    width: 1400px;
  }

  #chart-container {
    box-sizing: border-box;
    width: 100%;
    height: var(--chart-height, 60vh);
    min-height: 320px;
    padding: var(--space-8) 0;
  }

  @media (max-width: 48rem) {
    #chart-container {
      height: 55vh;
      padding: var(--space-4) 0;
    }

    .chart-scroll-container--scroll #chart-container {
      width: 960px;
    }

    .chart-scroll-container--fit #chart-container {
      width: 100%;
    }
  }
</style>
