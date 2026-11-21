/** @typedef {import("echarts").EChartsOption} EChartsOption */
/** @typedef {import("../constants/types.js").CommodityPrices} CommodityPrices */

/**
 * Convert an e-Stat commodity time code (for example 2025000303) to YYYY-MM.
 * The year occupies the first four digits and the month is encoded at digits 7–8.
 * @param {string} time
 * @returns {string | null}
 */
export function commodityMonth(time) {
  if (!/^\d{10}$/.test(time)) return null;
  const year = time.slice(0, 4);
  const month = Number(time.slice(6, 8));
  return month >= 1 && month <= 12 ? `${year}-${String(month).padStart(2, "0")}` : null;
}

/** @param {CommodityPrices} item */
function valuesByMonth(item) {
  const values = new Map();
  for (const pair of item.time_value_pairs) {
    const month = commodityMonth(pair.time);
    if (!month) continue;
    const value = Number(pair.value);
    values.set(month, Number.isFinite(value) ? value : null);
  }
  return values;
}

/**
 * Align commodity series by calendar month. With no reference month, values stay
 * in yen. Passing a common YYYY-MM month rebases each series to 100 at that month.
 * @param {CommodityPrices[]} data
 * @param {string} [selectedMonth]
 * @returns {{ chartData: EChartsOption | null, availableMonths: string[], referenceMonth: string | null }}
 */
export function transformData(data, selectedMonth = "") {
  const maps = data.map(valuesByMonth);
  const dates = [...new Set(maps.flatMap((map) => [...map.keys()]))].sort();
  const availableMonths = dates.filter((month) =>
    maps.length > 0 && maps.every((map) => {
      const value = map.get(month);
      return typeof value === "number" && Number.isFinite(value) && value > 0;
    }),
  );
  const referenceMonth = availableMonths.includes(selectedMonth) ? selectedMonth : null;
  const refValues = referenceMonth ? maps.map((map) => map.get(referenceMonth)) : [];
  const series = data.map((item, index) => ({
    name: `${item.cat02_name}【${item.area_name}】`,
    type: /** @type {"line"} */ ("line"),
    showSymbol: false,
    connectNulls: false,
    data: dates.map((date) => {
      const value = maps[index].get(date);
      if (typeof value !== "number" || !Number.isFinite(value)) return null;
      return referenceMonth
        ? (value / /** @type {number} */ (refValues[index])) * 100
        : value;
    }),
  }));

  return {
    availableMonths,
    referenceMonth,
    chartData: data.length === 0 ? null : {
      animation: false,
      aria: { enabled: true },
      tooltip: {
        trigger: "axis",
        ...(referenceMonth ? { valueFormatter: (value) => typeof value === "number" ? value.toFixed(1) : "–" } : {}),
      },
      legend: { type: "scroll", left: 56, top: 8 },
      grid: { left: 64, right: 24, top: 48, bottom: 88 },
      xAxis: { type: "category", data: dates.map((date) => date.replace("-", "年") + "月") },
      yAxis: {
        type: "value",
        name: referenceMonth ? "共通基準月＝100" : "価格（円）",
        nameLocation: "middle",
        nameGap: 46,
      },
      dataZoom: [
        { type: "slider", xAxisIndex: 0, show: true, realtime: true, start: 0, end: 100, height: 20, bottom: 8, showDetail: false, brushSelect: false },
        { type: "inside", xAxisIndex: 0, zoomOnMouseWheel: true, moveOnMouseMove: true, moveOnMouseWheel: false, preventDefaultMouseMove: true },
      ],
      series,
    },
  };
}
