/** @typedef {{ itemCode: string, areaCode: string, itemName: string, areaName: string, timeValuePairs: [string, number | null][] }} CpiSeries */
/** @typedef {import("echarts").EChartsOption} EChartsOption */

/**
 * @param {CpiSeries[]} series
 * @param {"index"|"normalized"} view
 * @param {string} [selectedMonth] Empty string selects the latest common month.
 * @returns {{ chartData: EChartsOption | null, referenceMonth: string | null, availableMonths: string[] }}
 */
export function transformData(series, view, selectedMonth = "") {
  const dates = [
    ...new Set(
      series.flatMap((item) => item.timeValuePairs.map(([date]) => date)),
    ),
  ].sort();
  const valuesBySeries = series.map((item) => new Map(item.timeValuePairs));
  const availableMonths =
    view === "normalized"
      ? dates
          .filter((date) =>
            valuesBySeries.every((values) => {
              const value = values.get(date);
              return (
                typeof value === "number" && Number.isFinite(value) && value > 0
              );
            }),
          )
          .reverse()
      : [];
  const referenceMonth = selectedMonth
    ? availableMonths.includes(selectedMonth)
      ? selectedMonth
      : null
    : (availableMonths[0] ?? null);

  if (series.length === 0 || (view === "normalized" && !referenceMonth)) {
    return { chartData: null, referenceMonth, availableMonths };
  }

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
      if (typeof value !== "number" || !Number.isFinite(value)) return null;
      return view === "normalized"
        ? (value / /** @type {number} */ (referenceValues[index])) * 100
        : value;
    }),
  }));

  return {
    referenceMonth,
    availableMonths,
    chartData: {
      animation: false,
      aria: { enabled: true },
      tooltip: {
        trigger: "axis",
        ...(view === "normalized"
          ? {
              valueFormatter: (value) =>
                typeof value === "number" ? value.toFixed(1) : "–",
            }
          : {}),
      },
      legend: { type: "scroll", left: 56, top: 8 },
      grid: { left: 64, right: 24, top: 48, bottom: 88 },
      xAxis: { type: "category", data: labels },
      yAxis: {
        type: "value",
        name:
          view === "normalized" ? "共通基準月 = 100" : "指数（2025年 = 100）",
        nameLocation: "middle",
        nameGap: 46,
      },
      dataZoom: [
        {
          type: "slider",
          xAxisIndex: 0,
          show: true,
          realtime: true,
          start: 0,
          end: 100,
          height: 20,
          bottom: 8,
          showDetail: false,
          brushSelect: false,
        },
        {
          type: "inside",
          xAxisIndex: 0,
          zoomOnMouseWheel: true,
          moveOnMouseMove: true,
          moveOnMouseWheel: false,
          preventDefaultMouseMove: true,
        },
      ],
      series: chartSeries,
    },
  };
}
