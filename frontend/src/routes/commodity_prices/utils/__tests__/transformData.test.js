import { describe, expect, test } from "vitest";
import { commodityMonth, transformData } from "../transformData.js";

/** @typedef {import("../../constants/types.js").CommodityPrices} CommodityPrices */
/** @typedef {import("echarts").EChartsOption} EChartsOption */

/** @returns {CommodityPrices[]} */
function returnMockedData() {
  return [
    {
      cat02_code: "01001",
      cat02_name: "1001 うるち米(単一原料米,「コシヒカリ」)",
      area_code: "01100",
      area_name: "札幌市",
      time_value_pairs: [
        { time: "2025000303", value: "4221" },
        { time: "2025000202", value: "3948" },
        { time: "2025000101", value: "3948" },
        { time: "2024001212", value: "3948" },
      ],
    },
    {
      cat02_code: "01002",
      cat02_name: "1002 うるち米(単一原料米,「コシヒカリ」以外)",
      area_code: "01202",
      area_name: "函館市",
      time_value_pairs: [
        { time: "2025000303", value: "4169" },
        { time: "2025000202", value: "3934" },
        { time: "2025000101", value: "3934" },
        { time: "2000000505", value: "4092" },
        { time: "2000000404", value: "..." },
        { time: "2000000303", value: "4078" },
        { time: "2000000202", value: "4230" },
        { time: "2000000101", value: "4319" },
      ],
    },
  ];
}

/** @param {EChartsOption | null} option */
function getLabels(option) {
  if (!option) return [];
  return /** @type {{ data: string[] }} */ (
    /** @type {unknown} */ (option.xAxis)
  ).data;
}

/** @param {EChartsOption | null} option */
function getSeries(option) {
  if (!option) return [];
  return /** @type {{ name: string, data: (number | null)[] }[]} */ (
    /** @type {unknown} */ (option.series)
  );
}

describe("transformData", () => {
  const dummyData = returnMockedData();

  test("transformData should handle empty array", () => {
    const result = transformData([]);
    expect(getLabels(result.chartData)).toEqual([]);
    expect(getSeries(result.chartData)).toEqual([]);
  });

  test("transformData should transform data correctly with single commodity", () => {
    const singleItemData = [dummyData[0]];
    const result = transformData(singleItemData).chartData;

    // Check dates are collected and sorted correctly
    expect(getLabels(result)).toEqual([
      "2024年12月",
      "2025年01月",
      "2025年02月",
      "2025年03月",
    ]);

    // Check dataset structure
    expect(getSeries(result)).toHaveLength(1);
    expect(getSeries(result)[0].name).toBe(
      "1001 うるち米(単一原料米,「コシヒカリ」)【札幌市】",
    );

    // Check values are parsed correctly
    expect(getSeries(result)[0].data).toEqual([3948, 3948, 3948, 4221]);
  });

  test("transformData should handle multiple commodities with different dates", () => {
    const result = transformData(dummyData).chartData;

    // All unique dates from both commodities should be included and sorted
    expect(getLabels(result)).toEqual([
      "2000年01月",
      "2000年02月",
      "2000年03月",
      "2000年04月",
      "2000年05月",
      "2024年12月",
      "2025年01月",
      "2025年02月",
      "2025年03月",
    ]);

    // Should have two datasets
    expect(getSeries(result)).toHaveLength(2);

    // First commodity should have nulls for dates it doesn't have data for
    expect(getSeries(result)[0].data).toEqual([
      null,
      null,
      null,
      null,
      null,
      3948,
      3948,
      3948,
      4221,
    ]);

    // Second commodity should have values for all dates
    expect(getSeries(result)[1].data).toEqual([
      4319,
      4230,
      4078,
      null,
      4092,
      null,
      3934,
      3934,
      4169,
    ]);
  });

  test("transformData should handle non-numeric values", () => {
    const dataWithInvalidValue = [
      {
        ...dummyData[0],
        time_value_pairs: [
          ...dummyData[0].time_value_pairs,
          { time: "2025000404", value: "N/A" },
        ],
      },
    ];

    const result = transformData(dataWithInvalidValue).chartData;

    // The N/A value should be converted to null
    expect(getSeries(result)[0].data).toContain(null);

    // Check if the rest of the values are still correctly processed
    expect(getLabels(result)).toContain("2025年04月");
  });

  test("transformData should format labels correctly", () => {
    const result = transformData(dummyData).chartData;

    // Check that the date format is YYYY-MM
    expect(
      (getLabels(result) ?? []).every((label) =>
        /^\d{4}年\d{2}月$/.test(label),
      ),
    ).toBe(true);

    // Check specific examples
    expect(getLabels(result)).toContain("2025年03月");
    expect(getLabels(result)).toContain("2000年01月");
  });

  test("transformData should create correct dataset labels", () => {
    const result = transformData(dummyData).chartData;

    expect(getSeries(result)[0].name).toBe(
      "1001 うるち米(単一原料米,「コシヒカリ」)【札幌市】",
    );
    expect(getSeries(result)[1].name).toBe(
      "1002 うるち米(単一原料米,「コシヒカリ」以外)【函館市】",
    );
  });

  test("decodes e-Stat months and offers only positive common months", () => {
    expect(commodityMonth("2025000303")).toBe("2025-03");
    expect(commodityMonth("2024001212")).toBe("2024-12");
    expect(commodityMonth("2025001301")).toBeNull();
    const result = transformData(dummyData);
    expect(result.availableMonths).toEqual(["2025-01", "2025-02", "2025-03"]);
  });

  test("rebases each series to 100 and retains missing values as gaps", () => {
    const result = transformData(dummyData, "2025-02");
    expect(result.referenceMonth).toBe("2025-02");
    expect(result.availableMonths).toContain("2025-02");
    const series = getSeries(result.chartData);
    expect(series[0].data.slice(-3)).toEqual([100, 100, (4221 / 3948) * 100]);
    expect(series[0].data[0]).toBeNull();
    expect(series[1].data.at(-1)).toBeCloseTo((4169 / 3934) * 100);
  });

  test("falls back to yen when the chosen month is not common", () => {
    const result = transformData(dummyData, "2000-01");
    expect(result.referenceMonth).toBeNull();
    expect(getSeries(result.chartData)[0].data.at(-1)).toBe(4221);
  });
});
