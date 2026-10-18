/** @typedef {import("echarts").EChartsOption} EChartsOption */
/** @typedef {import("../constants/types.js").CommodityPrices} CommodityPrices */

// Transform the fetched data into a format suitable for Apache ECharts.
/** @param {CommodityPrices[]} data @returns {EChartsOption} */
export function transformData(data) {
  const allDates = new Set();
  data.forEach((commodityPrices) => {
    commodityPrices.time_value_pairs.forEach((pair) => {
      allDates.add(pair.time);
    });
  });

  const sortedDates = Array.from(allDates).sort((a, b) => {
    return a > b ? 1 : a < b ? -1 : 0;
  });
  const labels = sortedDates.map((date) => {
    const year = date.slice(0, 4);
    const month = date.slice(8, 10);
    return `${year}-${month}`;
  });

  const series = data.map((commodityPrices) => {
    const timeValueMap = new Map(
      commodityPrices.time_value_pairs.map((pair) => [pair.time, pair.value]),
    );

    return {
      name: `${commodityPrices.cat02_name}【${commodityPrices.area_name}】`,
      type: /** @type {"line"} */ ("line"),
      showSymbol: false,
      connectNulls: false,
      data: sortedDates.map((date) => {
        const value = timeValueMap.get(date);
        if (value === undefined) return null;

        const parsedValue = parseFloat(value);
        return isNaN(parsedValue) ? null : parsedValue;
      }),
    };
  });

  return {
    animation: false,
    aria: { enabled: true },
    tooltip: { trigger: "axis" },
    legend: { type: "scroll", left: 56, top: 8 },
    grid: {
      left: 56,
      right: 24,
      top: 48,
      bottom: 88,
      outerBoundsMode: "same",
      outerBoundsContain: "axisLabel",
    },
    xAxis: { type: "category", data: labels },
    yAxis: { type: "value" },
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
    series,
  };
}
