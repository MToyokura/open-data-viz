<script>
  import { onMount } from "svelte";

  /** @typedef {{ code: string, label: string, count: number|null, status: string, raw_value: string, annotations: string[] }} ChartRow */
  /** @typedef {{ key: string, label: string, rows: ChartRow[] }} ChartSeries */
  /** @type {{ categories: { code: string, label: string }[], series: ChartSeries[] }} */
  let { categories, series } = $props();
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
    `${Math.max(360, categories.length * (series.length > 1 ? 52 : 42) + 100)}px`,
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

<div
  bind:this={container}
  class="chart"
  style:height={chartHeight}
  role="img"
  aria-label="純資産額階級別世帯数の横棒グラフ"
></div>

<style>
  .chart {
    width: 100%;
    min-height: 360px;
    margin: var(--space-4) 0;
    border: var(--border-default);
    border-radius: var(--radius-md);
  }
</style>
