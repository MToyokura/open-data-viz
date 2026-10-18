import { describe, expect, test } from "vitest";
import { validateOptions } from "../fetchData.js";

const education = [
  ["00", "総数", "1", null],
  ["01", "うち在学中", "2", null],
  ["02", "小学・中学・高校", "3", null],
  ["03", "短大・高専", "3", "02"],
  ["04", "大学・大学院", "3", "02"],
  ["05", "うち卒業", "2", null],
  ["06", "小学・中学・高校・旧中", "3", "05"],
  ["07", "短大・高専", "3", "05"],
  ["08", "大学・大学院", "3", "05"],
  ["09", "大学", "3", "05"],
  ["10", "大学院", "3", "05"],
].map(([code, name, api_level, api_parent_code], order) => ({
  code,
  name,
  order,
  api_level,
  api_parent_code,
}));

const expectedHierarchy = [
  {
    code: "00",
    children: [
      {
        code: "01",
        children: [{ code: "02" }, { code: "03" }, { code: "04" }],
      },
      {
        code: "05",
        children: [
          { code: "06" },
          { code: "07" },
          { code: "08" },
          { code: "09" },
          { code: "10" },
        ],
      },
    ],
  },
];

/** @param {Record<string, unknown>} [overrides] */
function makeOptions(overrides = {}) {
  return {
    dimensions: {
      age: [],
      employment_type: [],
      education,
      sex: [],
    },
    education_display_hierarchy: expectedHierarchy,
    education_comparison_sets: [
      { id: "education-status", category_codes: ["01", "05"] },
      { id: "enrolled-categories", category_codes: ["02", "03", "04"] },
      {
        id: "graduated-categories",
        category_codes: ["06", "07", "08", "09", "10"],
      },
    ],
    metadata: { stats_data_id: "test" },
    series: [],
    ...overrides,
  };
}

describe("workers options education display hierarchy", () => {
  test("accepts the expected tree", () => {
    expect(validateOptions(makeOptions()).education_display_hierarchy).toEqual(
      expectedHierarchy,
    );
  });

  test.each([
    [
      "swapped parents",
      [
        {
          code: "00",
          children: [
            {
              code: "01",
              children: [
                { code: "06" },
                { code: "07" },
                { code: "08" },
                { code: "09" },
                { code: "10" },
              ],
            },
            {
              code: "05",
              children: [{ code: "02" }, { code: "03" }, { code: "04" }],
            },
          ],
        },
      ],
    ],
    ["unknown code", [{ code: "00", children: [{ code: "99" }] }]],
    [
      "duplicate code",
      [
        {
          code: "00",
          children: [
            {
              code: "01",
              children: [{ code: "02" }, { code: "03" }, { code: "03" }],
            },
            {
              code: "05",
              children: [
                { code: "06" },
                { code: "07" },
                { code: "08" },
                { code: "09" },
                { code: "10" },
              ],
            },
          ],
        },
      ],
    ],
    [
      "missing code",
      [
        {
          code: "00",
          children: [
            { code: "01", children: [{ code: "02" }, { code: "03" }] },
            {
              code: "05",
              children: [
                { code: "06" },
                { code: "07" },
                { code: "08" },
                { code: "09" },
                { code: "10" },
              ],
            },
          ],
        },
      ],
    ],
  ])("rejects %s", (_case, hierarchy) => {
    expect(() =>
      validateOptions(makeOptions({ education_display_hierarchy: hierarchy })),
    ).toThrow("教育区分の画面表示階層が想定と異なります");
  });
});
