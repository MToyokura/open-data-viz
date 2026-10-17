import { beforeEach, describe, expect, test, vi } from "vitest";
import {
  fetchCombinationItems,
  getDisplayAssetCategories,
  validateAssetDistribution,
} from "../data.js";

const displayCodes = [
  "R01",
  "R02",
  "R03",
  ...Array.from({ length: 18 }, (_, index) =>
    String(index + 2).padStart(2, "0"),
  ),
];

/** @param {Record<string, unknown>} [overrides] */
/** @returns {any} */
function makeManifest(overrides = {}) {
  const categories = [
    { code: "01", label: "25万円未満", kind: "regular", order: 0 },
    { code: "M10", label: "平均", kind: "mean", order: 1 },
    ...displayCodes.map((code, order) => ({
      code,
      label: `${code}${order < 3 ? "（再掲）" : ""}`,
      kind: order < 3 ? "reprint" : "regular",
      order: order + 2,
    })),
  ];
  return {
    schema_version: 2,
    metadata: {},
    dimensions: [
      {
        id: "cat04",
        label: "資産額階級",
        role: "asset_class",
        order: 4,
        total_code: null,
        default_code: null,
        categories,
        display_category_codes: displayCodes,
      },
    ],
    combinations: [
      {
        id: "combination-a",
        dimensions: {},
        status: "published",
        reason: null,
        items_file: "test-items-unique.json",
      },
    ],
    chunks: [
      {
        path: "test-items-unique.json",
        combination_ids: ["combination-a"],
      },
    ],
    ...overrides,
  };
}

describe("asset distribution data contract", () => {
  test("uses manifest display codes in their canonical order", () => {
    const manifest = validateAssetDistribution(makeManifest());
    expect(getDisplayAssetCategories(manifest).map(({ code }) => code)).toEqual(
      displayCodes,
    );
  });

  test.each([1, 3])("rejects schema version %s", (schemaVersion) => {
    expect(() =>
      validateAssetDistribution({
        ...makeManifest(),
        schema_version: schemaVersion,
      }),
    ).toThrow("形式またはバージョン");
  });

  test("rejects a missing display list", () => {
    const manifest = makeManifest();
    delete manifest.dimensions[0].display_category_codes;
    expect(() => validateAssetDistribution(manifest)).toThrow("表示対象");
  });

  test("rejects duplicate, unknown, or overlapping display codes", () => {
    for (const codes of [
      [displayCodes[0], ...displayCodes.slice(1, -1), displayCodes[0]],
      ["unknown", ...displayCodes.slice(1)],
      ["01", ...displayCodes.slice(1)],
      [displayCodes[0], ...displayCodes.slice(2), "M10"],
    ]) {
      const manifest = makeManifest();
      manifest.dimensions[0].display_category_codes = codes;
      expect(() => validateAssetDistribution(manifest)).toThrow("表示対象");
    }
  });

  test("accepts only the 21 declared values for each published series", async () => {
    const manifest = validateAssetDistribution(makeManifest());
    const items = displayCodes.map((code) => ({
      combination_id: "combination-a",
      asset_class_code: code,
      count: 1,
      status: "observed",
      raw_value: "1",
      annotations: [],
    }));
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        text: async () => JSON.stringify({ schema_version: 2, items }),
      }),
    );

    await expect(
      fetchCombinationItems(manifest, "combination-a"),
    ).resolves.toHaveLength(21);
  });

  test("rejects an old chunk schema and missing or duplicate values", async () => {
    const makeItems = () =>
      displayCodes.map((code) => ({
        combination_id: "combination-a",
        asset_class_code: code,
        count: 1,
        status: "observed",
        raw_value: "1",
        annotations: [],
      }));

    for (const payload of [
      { schema_version: 1, items: makeItems() },
      { schema_version: 2, items: makeItems().slice(1) },
      {
        schema_version: 2,
        items: [...makeItems().slice(1), makeItems()[1]],
      },
      {
        schema_version: 2,
        items: [
          ...makeItems().slice(1),
          { ...makeItems()[1], asset_class_code: "01" },
        ],
      },
    ]) {
      const nextManifest = makeManifest();
      nextManifest.chunks[0].path = `items-${Math.random()}.json`;
      nextManifest.combinations[0].items_file = nextManifest.chunks[0].path;
      vi.stubGlobal(
        "fetch",
        vi.fn().mockResolvedValue({
          ok: true,
          text: async () => JSON.stringify(payload),
        }),
      );
      await expect(
        fetchCombinationItems(nextManifest, "combination-a"),
      ).rejects.toThrow();
    }
  });
});

beforeEach(() => {
  vi.unstubAllGlobals();
});
