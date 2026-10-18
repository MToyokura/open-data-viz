import { expect, test } from "@playwright/test";

const displayCodes = [
  "R01",
  "R02",
  "R03",
  ...Array.from({ length: 18 }, (_, index) =>
    String(index + 2).padStart(2, "0"),
  ),
];

const assetCategories = [
  { code: "01", label: "25万円未満", kind: "regular", order: 0 },
  ...displayCodes.map((code, order) => ({
    code,
    label: `${
      code === "R01"
        ? "0万円未満"
        : code === "R02"
          ? "0万円"
          : code === "R03"
            ? "0万円超25万円未満"
            : `${Number(code) * 25}万円以上`
    }（再掲）`,
    kind: order < 3 ? "reprint" : "regular",
    order: order + 1,
  })),
];

const manifest = {
  schema_version: 3,
  metadata: {
    year: 2024,
    published_at: "2025-12-19",
    area: "全国",
    measure: "世帯数",
    unit: "世帯",
    asset_basis: "純資産総額",
  },
  dimensions: [
    {
      id: "cat01",
      label: "世帯主の性別",
      role: "filter",
      order: 1,
      total_code: "00",
      default_code: "00",
      categories: [
        { code: "00", label: "総数", kind: "total", order: 0 },
        { code: "01", label: "男性", kind: "regular", order: 1 },
        { code: "02", label: "女性", kind: "regular", order: 2 },
      ],
      comparison_sets: [
        { id: "sex", label: "性別", category_codes: ["01", "02"] },
      ],
    },
    {
      id: "cat04",
      label: "資産額階級",
      role: "asset_class",
      order: 4,
      total_code: null,
      default_code: null,
      categories: assetCategories,
      display_category_codes: displayCodes,
    },
  ],
  combination_index: {
    dimension_ids: ["cat01"],
    count: 3,
  },
  chunks: [
    {
      path: "items/sex-series.json",
      start_index: 0,
      count: 3,
    },
  ],
};

const chunk = {
  schema_version: 3,
  items: ["00", "01", "02"].flatMap((sexCode) =>
    displayCodes.map((code, index) => ({
      combination_id: `cat01=${sexCode}`,
      asset_class_code: code,
      count: 1000 + index,
      status: "observed",
      raw_value: String(1000 + index),
      annotations: [],
    })),
  ),
};

test("asset distribution shows negative assets, selection, and comparison on a narrow screen", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.route("**/asset_distribution/manifest.json", (route) =>
    route.fulfill({ json: manifest }),
  );
  await page.route("**/asset_distribution/items/sex-series.json", (route) =>
    route.fulfill({ json: chunk }),
  );

  await page.goto("/asset_distribution");
  await expect(
    page.getByRole("heading", { name: "日本の純資産分布" }),
  ).toBeVisible();
  await expect(
    page.getByText(/「0万円未満」は純資産総額がマイナスの世帯です/),
  ).toBeVisible();

  const chart = page.getByRole("img", {
    name: "純資産額階級別世帯数の横棒グラフ",
  });
  await expect(chart).toBeVisible();
  await expect(chart).toHaveCSS("height", "982px");

  await page
    .getByRole("region", { name: "世帯属性の選択" })
    .getByLabel("世帯主の性別")
    .selectOption("02");
  await expect(chart).toHaveCSS("height", "1192px");

  await page.getByLabel("比較する属性").selectOption("cat01");
  const compareOptions = page.getByRole("group", { name: "比較する区分" });
  await expect(compareOptions.getByRole("checkbox")).toHaveCount(2);
  await expect(compareOptions.getByLabel("男性")).toBeChecked();
  await expect(compareOptions.getByLabel("女性")).toBeChecked();
  await expect(chart).toHaveCSS("height", "1192px");

  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(375);
});
