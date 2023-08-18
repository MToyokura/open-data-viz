import {
  createMonthCodeNamePairs,
  createNullFilledTimeSeries,
} from "./dataShaping";

const sampleResponseJsons = [
  {
    cat02_code: "01001",
    cat02_name: "1001 うるち米(単一原料米,「コシヒカリ」)",
    area_code: "01100",
    area_name: "札幌市",
    time_series: [
      {
        time_code: "2000000101",
        time_name: "2000年1月",
        value: "5468",
      },
      {
        time_code: "2000000202",
        time_name: "2000年2月",
        value: "5372",
      },
      {
        time_code: "2000000303",
        time_name: "2000年3月",
        value: "5469",
      },
      {
        time_code: "2000000404",
        time_name: "2000年4月",
        value: "5477",
      },
    ],
  },
  {
    cat02_code: "01001",
    cat02_name: "1001 うるち米(単一原料米,「コシヒカリ」)",
    area_code: "01202",
    area_name: "函館市",
    time_series: [
      {
        time_code: "2022001111",
        time_name: "2022年11月",
        value: "2441",
      },
      {
        time_code: "2022001212",
        time_name: "2022年12月",
        value: "2441",
      },
      {
        time_code: "2023000101",
        time_name: "2023年1月",
        value: "2441",
      },
      {
        time_code: "2023000202",
        time_name: "2023年2月",
        value: "2441",
      },
    ],
  },
];

describe("createMonthCodeNamePairs", () => {
  test("should return Map<string, string> with key: time_code, value: time_name, in chronological order", () => {
    const result = createMonthCodeNamePairs(sampleResponseJsons);
    expect(result.get("2000000101")).toBe("2000年1月");
    expect(result.get("2000000202")).toBe("2000年2月");
    expect(result.get("2000000303")).toBe("2000年3月");
    expect(result.get("2000000404")).toBe("2000年4月");
    expect(result.get("2022001111")).toBe("2022年11月");
    expect(result.get("2022001212")).toBe("2022年12月");
    expect(result.get("2023000101")).toBe("2023年1月");
    expect(result.get("2023000202")).toBe("2023年2月");

    // check if the order is chronological
    const keys = Array.from(result.keys()).map((key) => parseInt(key));
    for (const key of keys) {
      const index = keys.indexOf(key);
      if (index > 0) {
        const previousKey = keys[index - 1];
        expect(key).toBeGreaterThan(previousKey);
      }
    }
  });
});

describe("createNullFilledTimeSeries", () => {
  test("should return an array of IndividualTimeSeries with null", () => {
    const resultTimeSeries = createNullFilledTimeSeries(
      createMonthCodeNamePairs(sampleResponseJsons)
    );
    expect(resultTimeSeries.length).toBe(8);
    for (const individualTimeSeries of resultTimeSeries) {
      expect(individualTimeSeries.value).toBe(null);
    }
  });
});

// TODO: jsdom で structuredClone がまだ使えない
// https://github.com/jsdom/jsdom/issues/3363

// describe("fillNullFilledTimeSeriesWithResponseData", () => {
//   const sampleNullFilledTimeSeries = [
//     { time_code: "2000000101", time_name: "2000年1月", value: null },
//     { time_code: "2000000202", time_name: "2000年2月", value: null },
//     { time_code: "2000000303", time_name: "2000年3月", value: null },
//     { time_code: "2000000404", time_name: "2000年4月", value: null },
//     { time_code: "2010000101", time_name: "2010年1月", value: null },
//     { time_code: "2010000202", time_name: "2010年2月", value: null },
//     { time_code: "2022001111", time_name: "2022年11月", value: null },
//     { time_code: "2022001212", time_name: "2022年12月", value: null },
//     { time_code: "2023000101", time_name: "2023年1月", value: null },
//     { time_code: "2023000202", time_name: "2023年2月", value: null },
//   ];
//   test("should return an array of IndividualTimeSeries with values filled in", () => {
//     const result1 = fillNullFilledTimeSeriesWithResponseData(
//       sampleNullFilledTimeSeries,
//       sampleResponseJsons[0]
//     );
//     expect(result1.time_series).toStrictEqual([
//       { time_code: "2000000101", time_name: "2000年1月", value: "5468" },
//       { time_code: "2000000202", time_name: "2000年2月", value: "5372" },
//       { time_code: "2000000303", time_name: "2000年3月", value: "5469" },
//       { time_code: "2000000404", time_name: "2000年4月", value: "5477" },
//       { time_code: "2010000101", time_name: "2010年1月", value: null },
//       { time_code: "2010000202", time_name: "2010年2月", value: null },
//       { time_code: "2022001111", time_name: "2022年11月", value: null },
//       { time_code: "2022001212", time_name: "2022年12月", value: null },
//       { time_code: "2023000101", time_name: "2023年1月", value: null },
//       { time_code: "2023000202", time_name: "2023年2月", value: null },
//     ]);
//     const result2 = fillNullFilledTimeSeriesWithResponseData(
//       sampleNullFilledTimeSeries,
//       sampleResponseJsons[1]
//     );
//     expect(result2.time_series).toStrictEqual([
//       { time_code: "2000000101", time_name: "2000年1月", value: null },
//       { time_code: "2000000202", time_name: "2000年2月", value: null },
//       { time_code: "2000000303", time_name: "2000年3月", value: null },
//       { time_code: "2000000404", time_name: "2000年4月", value: null },
//       { time_code: "2010000101", time_name: "2010年1月", value: null },
//       { time_code: "2010000202", time_name: "2010年2月", value: null },
//       { time_code: "2022001111", time_name: "2022年11月", value: "2441" },
//       { time_code: "2022001212", time_name: "2022年12月", value: "2441" },
//       { time_code: "2023000101", time_name: "2023年1月", value: "2441" },
//       { time_code: "2023000202", time_name: "2023年2月", value: "2441" },
//     ]);
//   });
// });
