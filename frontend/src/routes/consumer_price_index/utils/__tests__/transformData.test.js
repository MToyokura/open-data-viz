import { expect, test } from "vitest";
import { transformData } from "../transformData.js";

/** @param {[string, number | null][]} timeValuePairs */
function series(timeValuePairs, areaCode = "00000") {
  return {
    itemCode: "0001",
    itemName: "総合",
    areaCode,
    areaName: areaCode,
    timeValuePairs,
  };
}

/** @param {ReturnType<typeof transformData>} result */
function plottedValues(result) {
  const lines = result.chartData?.series;
  if (!Array.isArray(lines)) throw new Error("Expected line series");
  return lines.map((line) => line.data);
}

test("rebases every series using its own value in the selected month", () => {
  const input = [
    series([
      ["2024-12", 101],
      ["2025-01", 105],
      ["2025-02", 110.2],
      ["2025-03", null],
    ]),
    series(
      [
        ["2025-01", 80],
        ["2025-02", 84],
        ["2025-03", 88],
      ],
      "13100",
    ),
  ];
  const result = transformData(input, "normalized", "2025-01");

  expect(result.referenceMonth).toBe("2025-01");
  expect(result.availableMonths).toEqual(["2025-02", "2025-01"]);
  expect(plottedValues(result)).toEqual([
    [(101 / 105) * 100, 100, (110.2 / 105) * 100, null],
    [null, 100, 105, (88 / 80) * 100],
  ]);
  const single = transformData([input[0]], "normalized", "2025-01");
  const tooltip =
    /** @type {{ valueFormatter: (value: unknown) => string }} */ (
      single.chartData?.tooltip
    );
  const [values] = plottedValues(single);
  if (!Array.isArray(values)) throw new Error("Expected plotted values");
  expect(tooltip.valueFormatter(values[2])).toBe("105.0");
  expect(transformData(input, "normalized").referenceMonth).toBe("2025-02");
  const published = transformData(input, "index", "2025-01");
  expect(published.availableMonths).toEqual(result.availableMonths);
  expect(published.referenceMonth).toBeNull();
  expect(plottedValues(published)).toEqual([
    [101, 105, 110.2, null],
    [null, 80, 84, 88],
  ]);
});

test("excludes missing, non-positive, and non-finite denominators", () => {
  const input = [
    series([
      ["2025-01", 100],
      ["2025-02", 0],
      ["2025-03", -1],
      ["2025-04", NaN],
      ["2025-05", Infinity],
      ["2025-06", null],
    ]),
    series(
      [
        ["2025-01", 90],
        ["2025-02", 91],
        ["2025-03", 92],
        ["2025-04", 93],
        ["2025-05", 94],
        ["2025-06", 95],
        ["2025-07", 96],
      ],
      "13100",
    ),
  ];

  expect(transformData(input, "normalized").availableMonths).toEqual([
    "2025-01",
  ]);
  expect(transformData(input, "normalized", "2025-02")).toEqual({
    chartData: null,
    referenceMonth: null,
    availableMonths: ["2025-01"],
  });
  for (const noCommon of [[], [input[0], series([["2025-07", 95]], "13100")]]) {
    expect(transformData(noCommon, "normalized")).toEqual({
      chartData: null,
      referenceMonth: null,
      availableMonths: [],
    });
  }
});
