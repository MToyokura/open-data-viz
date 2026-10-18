<script>
  import { onMount } from "svelte";
  import Spinner from "#lib/Spinner.svelte";

  /** @typedef {{ code: string, label: string, count: number|null, status: string, raw_value: string, annotations: string[] }} ChartRow */
  /** @typedef {{ key: string, label: string, rows: ChartRow[] }} ChartSeries */
  /** @type {{ categories: { code: string, label: string }[], series: ChartSeries[], loading?: boolean, loadingMessage?: string, placeholderSeriesCount?: number }} */
  let {
    categories,
    series,
    loading = false,
    loadingMessage = "グラフを読み込んでいます…",
    placeholderSeriesCount = 1,
  } = $props();
  /** @type {HTMLDivElement} */
  let container;
  let chart = $state.raw(
    /** @type {import("echarts").EChartsType|null} */ (null),
  );

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
      left: 18,
      right: 56,
      top: 24,
      bottom: series.length > 1 ? 64 : 32,
      containLabel: true,
    },
    xAxis: {
      type: "value",
      name: "世帯",
      nameLocation: "end",
      axisLabel: {
        formatter: (/** @type {number} */ value) => value.toLocaleString(),
      },
    },
    yAxis: {
      type: "category",
      inverse: true,
      data: categories.map((item) => item.code),
      axisLabel: {
        formatter: (/** @type {string} */ code) =>
          categories.find((item) => item.code === code)?.label ?? code,
        width: 230,
        overflow: "break",
        lineHeight: 16,
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
        position: "right",
        formatter: (/** @type {{ value: number|null }} */ params) =>
          params.value === null ? "" : params.value.toLocaleString(),
      },
      emphasis: { focus: "series" },
    })),
  }));

  const chartHeight = $derived(
    `${Math.max(360, categories.length * ((loading ? placeholderSeriesCount : series.length) > 1 ? 52 : 42) + 100)}px`,
  );

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

<div class="chart-stage" style:height={chartHeight}>
  <div
    bind:this={container}
    class="chart"
    role="img"
    aria-label="純資産額階級別世帯数の横棒グラフ"
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

<style>
  .chart {
    box-sizing: border-box;
    width: 100%;
    height: 100%;
    border: var(--border-default);
    border-radius: var(--radius-md);
  }

  .chart-stage {
    position: relative;
    width: 100%;
    margin: var(--space-4) 0;
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
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--color-surface, #fff);
  }

  .placeholder-message {
    margin: 0;
    color: var(--color-text-muted, #6b7280);
    font-size: 0.9rem;
  }
</style>
