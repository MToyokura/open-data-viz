import { beforeEach, describe, expect, test, vi } from "vitest";
import {
  fetchCombinationItems,
  findCombination,
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
  const filters = ["cat01", "cat02", "cat03", "cat05"].map((id, index) => ({
    id,
    label: id,
    role: "filter",
    order: index,
    total_code: "A",
    default_code: "A",
    categories: ["A", "B"].map((code, order) => ({
      code,
      label: code,
      kind: "regular",
      order,
    })),
  }));
  return {
    schema_version: 3,
    metadata: {},
    dimensions: [
      ...filters,
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
    combination_index: {
      dimension_ids: ["cat01", "cat02", "cat03", "cat05"],
      count: 16,
    },
    chunks: [
      {
        path: "test-items-unique.json",
        start_index: 0,
        count: 16,
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

  test.each([2, 4])("rejects schema version %s", (schemaVersion) => {
    expect(() =>
      validateAssetDistribution({
        ...makeManifest(),
        schema_version: schemaVersion,
      }),
    ).toThrow("形式またはバージョン");
  });

  test("loads values for a searched combination and preserves missing values", async () => {
    const manifest = validateAssetDistribution(makeManifest());
    const id = "cat01=A|cat02=A|cat03=A|cat05=A";
    const ids = Array.from({ length: 16 }, (_, index) => {
      const codes = Array(4);
      let remainder = index;
      for (let axis = 3; axis >= 0; axis--) {
        codes[axis] = remainder % 2 === 0 ? "A" : "B";
        remainder = Math.floor(remainder / 2);
      }
      return `cat01=${codes[0]}|cat02=${codes[1]}|cat03=${codes[2]}|cat05=${codes[3]}`;
    });
    const items = ids.flatMap((combination_id) =>
      displayCodes.map((code, index) => ({
        combination_id,
        asset_class_code: code,
        count: index === 1 ? null : index === 2 ? 0 : 1,
        status: index === 1 ? "missing" : "observed",
        raw_value: index === 1 ? "-" : String(index === 2 ? 0 : 1),
        annotations: [],
      })),
    );
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        text: async () => JSON.stringify({ schema_version: 3, items }),
      }),
    );

    await expect(fetchCombinationItems(manifest, id)).resolves.toEqual(
      items.slice(0, 21),
    );
  });

  test("rejects an unsupported chunk schema version", async () => {
    const manifest = makeManifest();
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        text: async () => JSON.stringify({ schema_version: 2, items: [] }),
      }),
    );
    await expect(
      fetchCombinationItems(manifest, "cat01=A|cat02=A|cat03=A|cat05=A"),
    ).rejects.toThrow("形式またはバージョン");
  });

  test("derives IDs in Cartesian order with the final axis changing fastest", () => {
    const manifest = validateAssetDistribution(makeManifest());
    expect(
      findCombination(manifest, {
        cat01: "A",
        cat02: "A",
        cat03: "B",
        cat05: "B",
      }),
    ).toMatchObject({
      id: "cat01=A|cat02=A|cat03=B|cat05=B",
      items_file: "test-items-unique.json",
      status: "published",
    });
    expect(
      findCombination(manifest, {
        cat01: "B",
        cat02: "A",
        cat03: "A",
        cat05: "A",
      }),
    ).toMatchObject({ id: "cat01=B|cat02=A|cat03=A|cat05=A" });
  });
});

beforeEach(() => {
  vi.unstubAllGlobals();
});
