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
      id: "cat02",
      label: "世帯主の学歴",
      role: "filter",
      order: 2,
      total_code: "0",
      default_code: "0",
      categories: [
        { code: "0", label: "平均", kind: "regular", order: 0, api_level: "1" },
        {
          code: "1",
          label: "在学中",
          kind: "regular",
          order: 1,
          api_level: "1",
        },
        { code: "2", label: "卒業", kind: "regular", order: 2, api_level: "1" },
        {
          code: "21",
          label: "小学校・中学校",
          kind: "regular",
          order: 3,
          api_level: "2",
          api_parent_code: "2",
        },
        {
          code: "22",
          label: "高校程度",
          kind: "regular",
          order: 4,
          api_level: "2",
          api_parent_code: "2",
        },
        {
          code: "23",
          label: "短大程度以上",
          kind: "regular",
          order: 5,
          api_level: "2",
          api_parent_code: "2",
        },
      ],
      display_hierarchy: {
        standalone_codes: ["0", "1"],
        groups: [{ code: "2", children: ["21", "22", "23"] }],
      },
      comparison_sets: [
        {
          id: "education-status",
          label: "在学中・卒業者全体",
          category_codes: ["1", "2"],
        },
        {
          id: "graduated-subgroups",
          label: "卒業者の学歴内訳",
          category_codes: ["21", "22", "23"],
        },
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
    dimension_ids: ["cat01", "cat02"],
    count: 18,
  },
  chunks: [
    {
      path: "items/sex-series.json",
      start_index: 0,
      count: 18,
    },
  ],
};

const chunk = {
  schema_version: 3,
  items: ["00", "01", "02"].flatMap((sexCode) =>
    displayCodes.map((code, index) => ({
      combination_id: `cat01=${sexCode}|cat02=0`,
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
  const chartOrder = await page.evaluate(() => ({
    chart: document.querySelector(".chart-panel").getBoundingClientRect().top,
    controls: document.querySelector(".details-column").getBoundingClientRect()
      .top,
  }));
  expect(chartOrder.chart).toBeLessThan(chartOrder.controls);

  await page.locator(".selector-panel > summary").click();
  await page
    .getByRole("region", { name: "世帯属性の選択" })
    .getByLabel("世帯主の性別")
    .selectOption("02");
  await page.getByRole("button", { name: "系列に追加" }).click();
  await expect(chart).toHaveCSS("height", "1192px");

  await page.getByRole("button", { name: "属性を比較" }).click();
  await page.getByLabel("比較する属性").selectOption("cat01");
  const compareOptions = page.getByRole("group", { name: "比較する区分" });
  await expect(compareOptions.getByRole("checkbox")).toHaveCount(2);
  await expect(compareOptions.getByLabel("男性")).toBeChecked();
  await expect(compareOptions.getByLabel("女性")).toBeChecked();
  await expect(chart).toHaveCSS("height", "1192px");

  await page.setViewportSize({ width: 1024, height: 900 });
  await expect(page.locator(".selector-panel")).toHaveJSProperty("open", true);
  await expect(page.locator(".comparison-panel .selected-list li")).toHaveCount(
    2,
  );
  await page.setViewportSize({ width: 375, height: 812 });
  await expect(page.locator(".selector-panel")).toHaveJSProperty("open", false);
  await page.locator(".selector-panel > summary").click();
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect(
    page.locator(".selector-panel").getByLabel("世帯主の性別"),
  ).toHaveValue("02");
  await expect(page.locator(".comparison-panel .selected-list li")).toHaveCount(
    2,
  );

  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(375);
});
