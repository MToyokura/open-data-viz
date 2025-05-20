import type { ChartData } from "chart.js";
import type { CommodityPrices } from "../constants/types";

// Transform the fetched data into a format suitable for Chart.js
export function transformData(data: CommodityPrices[]): ChartData {
  const allDates = new Set<string>();
  data.forEach((commodityPrices) => {
    commodityPrices.time_value_pairs.forEach((pair) => {
      allDates.add(pair.time);
    });
  });

  // Convert the Set to an array and sort it
  const sortedDates = Array.from(allDates).sort((a, b) => {
    return a > b ? 1 : a < b ? -1 : 0;
  });

  const transformedData = data.map((commodityPrices) => {
    const timeValueMap = new Map(
      commodityPrices.time_value_pairs.map((pair) => [pair.time, pair.value]),
    );

    // Create an array of values for each date in sorted order
    const values = sortedDates.reduce(
      (acc, date) => {
        const value = timeValueMap.get(date);
        if (value !== undefined) {
          const parsedValue = parseFloat(value);
          acc.push(isNaN(parsedValue) ? null : parsedValue);
        } else {
          acc.push(null);
        }
        return acc;
      },
      [] as (number | null)[],
    );

    return {
      label: `${commodityPrices.cat02_name}【${commodityPrices.area_name}】`,
      data: values,
    };
  });

  return {
    labels: sortedDates.map((date) => {
      // example: "2021001010"
      const year = date.slice(0, 4);
      const month = date.slice(8, 10);
      return `${year}-${month}`;
    }),
    datasets: transformedData,
  };
}
