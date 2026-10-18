<script>
  import { onMount } from "svelte";
  import Spinner from "#lib/Spinner.svelte";

  /** @typedef {{ code: string, label: string, count: number|null, status: string, raw_value: string, annotations: string[] }} ChartRow */
  /** @typedef {{ key: string, label: string, rows: ChartRow[] }} ChartSeries */
  /** @type {{ categories: { code: string, label: string }[], series: ChartSeries[], orientation: "vertical"|"horizontal", loading?: boolean, loadingMessage?: string, placeholderSeriesCount?: number }} */
  let {
    categories,
    series,
    orientation,
    loading = false,
    loadingMessage = "グラフを読み込んでいます…",
    placeholderSeriesCount = 1,
  } = $props();
  /** @type {HTMLDivElement} */
  let container;
  /** @type {HTMLDivElement} */
  let scrollContainer;
  let chart = $state.raw(
    /** @type {import("echarts").EChartsType|null} */ (null),
  );
  let viewMode = $state(/** @type {"fit"|"scroll"} */ ("fit"));
  let chartWidth = $state(0);
  const categoryLabelWidth = $derived(
    chartWidth > 0 && chartWidth < 520
      ? Math.max(88, Math.round(chartWidth * 0.28))
      : 230,
  );

  /** @param {"fit"|"scroll"} mode */
  function setViewMode(mode) {
    viewMode = mode;
    requestAnimationFrame(() => {
      if (mode === "fit") scrollContainer.scrollLeft = 0;
      chart?.resize();
    });
  }

  let option = $derived.by(() => ({
    animation: false,
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "shadow" },
      formatter: (/** @type {any[]|any} */ params) => {
        const values = Array.isArray(params) ? params : [params];
        const code = values[0]?.axisValue;
        const rows = series.map((entry) => {
          const row = entry.rows.find((item) => item.code === code);
          if (!row) return `${entry.label}: データなし`;
          if (row.status !== "observed")
            return `${entry.label}: ${row.status === "suppressed" ? "秘匿" : "欠損"}${row.raw_value ? `（${row.raw_value}）` : ""}${row.annotations.length ? ` — ${row.annotations.join("、")}` : ""}`;
          return `${entry.label}: ${row.count?.toLocaleString() ?? "数値なし"} 世帯`;
        });
        const categoryLabel =
          categories.find((item) => item.code === code)?.label ??
          String(values[0]?.name ?? "");
        return [categoryLabel, ...rows].join("<br>");
      },
    },
    legend: { type: "scroll", bottom: 0 },
    grid: {
      left: orientation === "horizontal" ? 18 : 56,
      right: orientation === "horizontal" ? 56 : 24,
      top: 24,
      bottom:
        orientation === "vertical"
          ? series.length > 1
            ? 104
            : 84
          : series.length > 1
            ? 64
            : 32,
      containLabel: true,
    },
    xAxis:
      orientation === "horizontal"
        ? {
            type: "value",
            name: "世帯",
            nameLocation: "end",
            axisLabel: {
              hideOverlap: true,
              formatter: (/** @type {number} */ value) =>
                value.toLocaleString(),
            },
          }
        : {
            type: "category",
            data: categories.map((item) => item.code),
            axisLabel: {
              interval: 0,
              rotate: 45,
              formatter: (/** @type {string} */ code) =>
                categories.find((item) => item.code === code)?.label ?? code,
            },
          },
    yAxis:
      orientation === "horizontal"
        ? {
            type: "category",
            inverse: true,
            data: categories.map((item) => item.code),
            axisLabel: {
              formatter: (/** @type {string} */ code) =>
                categories.find((item) => item.code === code)?.label ?? code,
              width: categoryLabelWidth,
              overflow: "break",
              lineHeight: 16,
            },
          }
        : {
            type: "value",
            name: "世帯",
            nameLocation: "end",
            axisLabel: {
              hideOverlap: true,
              formatter: (/** @type {number} */ value) =>
                value.toLocaleString(),
            },
          },
    series: series.map((entry) => ({
      id: entry.key,
      name: entry.label,
      type: "bar",
      barMaxWidth: 24,
      data: categories.map((category) => {
        const row = entry.rows.find((item) => item.code === category.code);
        return row?.status === "observed" ? row.count : null;
      }),
      label: {
        show: series.length === 1,
        position: orientation === "horizontal" ? "right" : "top",
        formatter: (/** @type {{ value: number|null }} */ params) =>
          params.value === null ? "" : params.value.toLocaleString(),
      },
      emphasis: { focus: "series" },
    })),
  }));

  const chartHeight = $derived(
    orientation === "vertical"
      ? "max(360px, 62vh)"
      : `${Math.max(360, categories.length * ((loading ? placeholderSeriesCount : series.length) > 1 ? 52 : 42) + 100)}px`,
  );

  onMount(() => {
    let disposed = false;
    /** @type {ResizeObserver|undefined} */
    let observer;
    import("echarts").then(({ init }) => {
      if (disposed) return;
      chart = init(container);
      observer = new ResizeObserver(() => {
        chartWidth = container.clientWidth;
        chart?.resize();
      });
      observer.observe(container);
      chartWidth = container.clientWidth;
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
    ? "純資産分布グラフ（画面幅表示）"
    : "純資産分布グラフ。左右にスクロールできます"}
>
  <div class="chart-stage" style:height={chartHeight}>
    <div
      bind:this={container}
      class="chart"
      role="img"
      aria-label="純資産額階級別世帯数の{orientation === 'horizontal'
        ? '横棒'
        : '縦棒'}グラフ"
      aria-busy={loading}
      aria-hidden={loading}
    ></div>
    {#if loading}
      <div class="chart-placeholder" role="status">
        <span aria-hidden="true"><Spinner /></span>
        <p class="placeholder-message">{loadingMessage}</p>
      </div>
    {/if}
  </div>
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
    cursor: pointer;
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
    margin: var(--space-4) 0;
    overflow-x: auto;
    overflow-y: hidden;
    border: var(--border-default);
    border-radius: var(--radius-md);
    -webkit-overflow-scrolling: touch;
  }
  .chart-scroll-container--fit {
    overflow-x: hidden;
  }
  .chart-scroll-container--scroll .chart-stage {
    width: 1400px;
  }
  .chart-stage {
    position: relative;
    width: 100%;
  }
  .chart {
    box-sizing: border-box;
    width: 100%;
    height: 100%;
  }

  .chart-placeholder {
    box-sizing: border-box;
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: 28px 24px;
    background: var(--color-surface, #fff);
  }

  .placeholder-message {
    margin: 0;
    color: var(--color-text-muted, #6b7280);
    font-size: 0.9rem;
  }
  @media (max-width: 48rem) {
    .chart-scroll-container--scroll .chart-stage {
      width: 960px;
    }
  }
</style>
