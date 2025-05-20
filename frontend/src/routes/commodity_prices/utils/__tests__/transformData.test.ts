import { describe, expect, test } from "vitest";
import type { CommodityPrices } from "../../constants/types";
import { transformData } from "../transformData";

function returnMockedData(): CommodityPrices[] {
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

describe("transformData", () => {
  const dummyData = returnMockedData();

  test("transformData should handle empty array", () => {
    const result = transformData([]);
    expect(result.labels).toEqual([]);
    expect(result.datasets).toEqual([]);
  });

  test("transformData should transform data correctly with single commodity", () => {
    const singleItemData = [dummyData[0]];
    const result = transformData(singleItemData);

    // Check dates are collected and sorted correctly
    expect(result.labels).toEqual(["2024-12", "2025-01", "2025-02", "2025-03"]);

    // Check dataset structure
    expect(result.datasets).toHaveLength(1);
    expect(result.datasets[0].label).toBe(
      "1001 うるち米(単一原料米,「コシヒカリ」)【札幌市】",
    );

    // Check values are parsed correctly
    expect(result.datasets[0].data).toEqual([3948, 3948, 3948, 4221]);
  });

  test("transformData should handle multiple commodities with different dates", () => {
    const result = transformData(dummyData);

    // All unique dates from both commodities should be included and sorted
    expect(result.labels).toEqual([
      "2000-01",
      "2000-02",
      "2000-03",
      "2000-04",
      "2000-05",
      "2024-12",
      "2025-01",
      "2025-02",
      "2025-03",
    ]);

    // Should have two datasets
    expect(result.datasets).toHaveLength(2);

    // First commodity should have nulls for dates it doesn't have data for
    expect(result.datasets[0].data).toEqual([
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
    expect(result.datasets[1].data).toEqual([
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

    const result = transformData(dataWithInvalidValue);

    // The N/A value should be converted to null
    expect(result.datasets[0].data).toContain(null);

    // Check if the rest of the values are still correctly processed
    expect(result.labels).toContain("2025-04");
  });

  test("transformData should format labels correctly", () => {
    const result = transformData(dummyData);

    // Check that the date format is YYYY-MM
    expect(
      ((result.labels ?? []) as string[]).every((label) =>
        /^\d{4}-\d{2}$/.test(label),
      ),
    ).toBe(true);

    // Check specific examples
    expect(result.labels).toContain("2025-03");
    expect(result.labels).toContain("2000-01");
  });

  test("transformData should create correct dataset labels", () => {
    const result = transformData(dummyData);

    expect(result.datasets[0].label).toBe(
      "1001 うるち米(単一原料米,「コシヒカリ」)【札幌市】",
    );
    expect(result.datasets[1].label).toBe(
      "1002 うるち米(単一原料米,「コシヒカリ」以外)【函館市】",
    );
  });
});
