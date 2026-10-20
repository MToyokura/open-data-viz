<script>
  import ChartViewControls from "#lib/ChartViewControls.svelte";
  import { onMount } from "svelte";

  /** @typedef {import("echarts").EChartsType} EChartsType */
  /** @typedef {{ key: string, label: string, status: string, data?: import("./constants/types.js").WorkersSeries }} DisplaySeries */
  /** @type {{ items: DisplaySeries[], orientation: "vertical"|"horizontal" }} */
  let { items, orientation } = $props();
  /** @type {HTMLDivElement} */
  let scrollContainer;
  /** @type {HTMLDivElement} */
  let container;
  let chart = $state.raw(/** @type {EChartsType|null} */ (null));
  let viewMode = $state(/** @type {"fit"|"scroll"} */ ("fit"));

  /** @param {"fit"|"scroll"} mode */
  function handleViewModeChange(mode) {
    requestAnimationFrame(() => {
      if (mode === "fit") scrollContainer.scrollLeft = 0;
      chart?.resize();
    });
  }

  let years = $derived(
    [
      ...new Set(
        items.flatMap(
          (item) => item.data?.time_value_pairs.map((row) => row.year) ?? [],
        ),
      ),
    ].sort(),
  );
  let option = $derived.by(() => {
    const horizontal = orientation === "horizontal";
    const categories = items.filter((item) => item.data);
    return {
      animation: false,
      tooltip: {
        trigger: "axis",
        formatter: (/** @type {any[]|any} */ params) => {
          const values = Array.isArray(params) ? params : [params];
          const year = String(values[0]?.axisValue ?? "");
          const rows = categories.map((item) => {
            const row = item.data?.time_value_pairs.find(
              (entry) => entry.year === year,
            );
            const marker =
              values.find((entry) => entry.seriesId === item.key)?.marker ?? "";
            const value = !row
              ? "この年のデータなし"
              : row.value === null
                ? row.reason
                  ? "注記あり（「データについて」の「利用上の注意」を参照）"
                  : "欠損"
                : `${row.value.toLocaleString()} ${item.data?.unit ?? ""}`;
            return `${marker}${item.label}: ${value}`;
          });
          return [year, ...rows].join("<br>");
        },
      },
      legend: {
        type: "scroll",
        bottom: 0,
        formatter: (/** @type {string} */ name) =>
          name.replace(/\s+\[[^\]]+\]$/, ""),
      },
      grid: {
        left: horizontal ? 120 : 68,
        right: 24,
        top: 40,
        bottom: 72,
        containLabel: true,
      },
      xAxis: horizontal
        ? {
            type: "value",
            name: categories[0]?.data?.unit ?? "",
            nameLocation: "middle",
            nameGap: 32,
            nameRotate: 0,
          }
        : { type: "category", data: years, axisLabel: { rotate: 45 } },
      yAxis: horizontal
        ? { type: "category", data: years }
        : {
            type: "value",
            name: categories[0]?.data?.unit ?? "",
            nameLocation: "middle",
            nameGap: 42,
            nameRotate: 0,
          },
      series: categories.map((item) => ({
        id: item.key,
        // ECharts treats series names as legend identities, so include the stable codes.
        name: `${item.label} [${item.key.replaceAll("|", "/")}]`,
        type: "bar",
        data: years.map(
          (year) =>
            item.data?.time_value_pairs.find((row) => row.year === year)
              ?.value ?? null,
        ),
        barMaxWidth: 28,
        ...(horizontal ? { barMinHeight: 1 } : {}),
      })),
    };
  });

  onMount(() => {
    let disposed = false;
    /** @type {ResizeObserver|undefined} */
    let observer;
    import("echarts").then(({ init }) => {
      if (disposed) return;
      chart = init(container);
      observer = new ResizeObserver(() => chart?.resize());
      observer.observe(container);
      chart.setOption(option, { notMerge: true });
    });
    return () => {
      disposed = true;
      observer?.disconnect();
      chart?.dispose();
      chart = null;
    };
  });

  $effect(() => {
    if (chart) chart.setOption(option, { notMerge: true });
  });
</script>

<ChartViewControls bind:mode={viewMode} onchange={handleViewModeChange} />
<div
  class={[
    "chart-scroll-container",
    { "chart-scroll-container--fit": viewMode === "fit" },
    { "chart-scroll-container--scroll": viewMode === "scroll" },
  ]}
  bind:this={scrollContainer}
  role="region"
  aria-label={viewMode === "fit"
    ? "雇用者数グラフ（画面幅表示）"
    : "雇用者数グラフ。左右にスクロールできます"}
>
  <div
    bind:this={container}
    class="chart chart-scroll-content"
    role="img"
    aria-label="雇用者数の年次推移"
  ></div>
</div>

<style>
  .chart {
    height: 62vh;
    min-height: 360px;
  }
  @media (max-width: 48rem) {
    .chart {
      height: 55vh;
      min-height: 320px;
    }
  }
</style>
