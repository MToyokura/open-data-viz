import { PlotData } from "../types";

/**
 * ApexCharts の series を作る
 */
export function createPlotDataSeries(plotData: PlotData[]) {
  return plotData.map((individualSeries) => {
    return {
      name: `
            年齢階級：${individualSeries["年齢階級(詳細集計）"]} |
            雇用形態：${individualSeries["雇用形態"]} |
            教育：${individualSeries["教育"]} |
            性別：${individualSeries["性別"]}
          `,
      data: Object.values(individualSeries.data).map((value) => {
        return Number(value);
      }),
    };
  });
}

/**
 * ApexCharts の options を作る
 */
export function createOptions(
  years: string[],
  isHorizontal: boolean
): ApexCharts.ApexOptions {
  return {
    chart: {
      // type, series, width, height は ReactApexChart 本体の props として渡さないと動かないっぽい
      animations: {
        easing: "easeout",
        speed: 150,
      },
      type: "bar",
    },
    plotOptions: {
      bar: { horizontal: isHorizontal },
    },
    dataLabels: {
      enabled: false,
    },
    stroke: {
      show: false,
    },
    xaxis: {
      categories: years, // なぜかこれは isHorizontal に関係ない
      title: { text: isHorizontal ? "万人" : "" },
    },
    yaxis: {
      title: { text: isHorizontal ? "" : "万人" },
    },
    fill: {
      opacity: 1,
    },
  };
}
