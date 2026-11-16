/** @typedef {{ itemCode: string, areaCode: string, itemName: string, areaName: string, timeValuePairs: [string, number][] }} CpiSeries */
/** @typedef {import("echarts").EChartsOption} EChartsOption */

/**
 * @param {CpiSeries[]} series
 * @param {"index"|"normalized"} view
 * @returns {{ chartData: EChartsOption | null, referenceMonth: string | null }}
 */
export function transformData(series, view) {
  if (series.length === 0) return { chartData: null, referenceMonth: null };

  const dates = [...new Set(series.flatMap((item) => item.timeValuePairs.map(([date]) => date)))].sort();
  const valuesBySeries = series.map(
    (item) => new Map(item.timeValuePairs.map(([date, value]) => [date, value])),
  );
  const referenceMonth =
    view === "normalized"
      ? dates
          .slice()
          .reverse()
          .find((date) => valuesBySeries.every((values) => values.has(date) && values.get(date) !== 0)) ?? null
      : null;
  const referenceValues = referenceMonth
    ? valuesBySeries.map((values) => values.get(referenceMonth))
    : [];
  const labels = dates.map((date) => date.replace("-", "年") + "月");
  const chartSeries = series.map((item, index) => ({
    name: `${item.itemName}【${item.areaName}】`,
    type: /** @type {"line"} */ ("line"),
    showSymbol: false,
    connectNulls: false,
    data: dates.map((date) => {
      const value = valuesBySeries[index].get(date);
      if (value === undefined) return null;
      return view === "normalized"
        ? (value / /** @type {number} */ (referenceValues[index])) * 100
        : value;
    }),
  }));

  return {
    referenceMonth,
    chartData: {
      animation: false,
      aria: { enabled: true },
      tooltip: {
        trigger: "axis",
        ...(view === "normalized"
          ? { valueFormatter: (value) => Number(value).toFixed(1) }
          : {}),
      },
      legend: { type: "scroll", left: 56, top: 8 },
      grid: { left: 64, right: 24, top: 48, bottom: 88 },
      xAxis: { type: "category", data: labels },
      yAxis: {
        type: "value",
        name: view === "normalized" ? "共通基準月 = 100" : "指数（2025年 = 100）",
        nameLocation: "middle",
        nameGap: 46,
      },
      dataZoom: [
        { type: "slider", xAxisIndex: 0, show: true, realtime: true, start: 0, end: 100, height: 20, bottom: 8, showDetail: false, brushSelect: false },
        { type: "inside", xAxisIndex: 0, zoomOnMouseWheel: true, moveOnMouseMove: true, moveOnMouseWheel: false, preventDefaultMouseMove: true },
      ],
      series: chartSeries,
    },
  };
}
