import { expect, test } from "@playwright/test";

const cpiItems = [
  { code: "0001", name: "総合" },
  { code: "0178", name: "生鮮食品及びエネルギーを除く総合" },
  ...[
    "0002",
    "1461",
    "1761",
    "1914",
    "3000",
    "3500",
    "4003",
    "4021",
    "4042",
    "4063",
    "4413",
    "0120",
    "0137",
    "9372",
    "0162",
    "9621",
    "9626",
    "9622",
  ].map((code) => ({ code, name: `品目 ${code}` })),
];

async function mockCpi(page) {
  const base = "**/consumer_price_index/transformed_data";
  await page.route(`${base}/items.json`, (route) =>
    route.fulfill({ json: cpiItems }),
  );
  await page.route(`${base}/regions.json`, (route) =>
    route.fulfill({ json: [{ code: "00000", name: "全国" }] }),
  );

  for (const start of [0, 10]) {
    const end = start + 9;
    const first = String(start).padStart(4, "0");
    const last = String(end).padStart(4, "0");
    await page.route(
      `${base}/series/area_00000_items_${first}_${last}.json`,
      (route) =>
        route.fulfill({
          json: cpiItems.slice(start, end + 1).map(({ code }) => ({
            itemCode: code,
            areaCode: "00000",
            timeValuePairs: [
              ["2000-01", 90],
              ["2005-01", 95],
              ["2025-01", 100],
            ],
          })),
        }),
    );
  }
}

test("consumer price index workspace plots a selected series", async ({
  page,
}) => {
  await mockCpi(page);
  await page.goto("/consumer_price_index");
  await expect(
    page.getByRole("heading", { name: "消費者物価指数", exact: true }),
  ).toBeVisible();
  const selectors = page.getByRole("region", { name: "品目と地域" });
  await selectors.getByRole("checkbox", { name: "総合", exact: true }).check();
  await selectors.getByRole("checkbox", { name: "全国", exact: true }).check();
  await expect(
    page.getByRole("img", { name: /総合【全国】/ }).locator("canvas"),
  ).toBeVisible();
});

test("vegetable production relay renders a map for the loaded month", async ({
  page,
}) => {
  await page.route("**/vegetable_production_relay/products.json", (route) =>
    route.fulfill({
      json: {
        schema_version: 1,
        products: [{ key: "veg-01", label: "だいこん" }],
      },
    }),
  );
  await page.route(
    "**/vegetable_production_relay/veg-01.summary.json",
    (route) =>
      route.fulfill({
        json: {
          schema_version: 2,
          status: "valid",
          commodity: "veg-01",
          commodity_label: "だいこん",
          mapping_warnings: [],
          years: [
            {
              year: 2021,
              table_id: "0000000001",
              source_unit: "t",
              months: [
                {
                  month: "2021-01",
                  consignor_prefecture_code: "1002",
                  consignor_prefecture_name: "北海道",
                  reported_value_sum: "120",
                  source_unit: "t",
                  status: "valid",
                  reported_cells: 1,
                  below_display_unit_cells: 0,
                  no_fact_cells: 0,
                  unknown_or_uncollected_cells: 0,
                },
              ],
            },
          ],
        },
      }),
  );
  await page.route(
    "**/prefectures/N03-20260101_prefecture_simplified.geojson",
    (route) =>
      route.fulfill({
        json: {
          type: "FeatureCollection",
          features: [
            {
              type: "Feature",
              properties: { N03_001: "北海道" },
              geometry: {
                type: "Polygon",
                coordinates: [
                  [
                    [141, 43],
                    [142, 43],
                    [142, 44],
                    [141, 44],
                    [141, 43],
                  ],
                ],
              },
            },
          ],
        },
      }),
  );

  await page.goto("/vegetable_production_relay");
  await expect(
    page.getByRole("heading", { name: "野菜の産地リレー" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "だいこん · 2021年 1月の出荷数量" }),
  ).toBeVisible();
  const map = page.locator('[aria-label="都道府県別の野菜卸売数量地図"]');
  await expect(map.locator(".leaflet-overlay-pane path").first()).toBeVisible();
});

test("marine fishery catch plots a selected species", async ({ page }) => {
  await page.route(
    "**/marine_fishery_catch/transformed_data/catches.json",
    (route) =>
      route.fulfill({
        json: {
          years: [2023, 2024],
          series: [
            {
              id: "col-b",
              label: "漁獲量計",
              groupPath: [],
              type: "total",
              values: [100, 120],
            },
          ],
          notes: [],
        },
      }),
  );
  await page.goto("/marine_fishery_catch");
  await expect(
    page.getByRole("heading", { name: "海面漁業魚種別漁獲量" }),
  ).toBeVisible();
  await page.getByRole("checkbox", { name: "漁獲量計" }).check();
  await expect(
    page
      .getByRole("img", { name: "選択した魚種の年別漁獲量を示す折れ線グラフ" })
      .locator("canvas"),
  ).toBeVisible();
});

test("commodity prices article renders its price and catch charts", async ({
  page,
}) => {
  await page.route(
    "**/commodity_prices/v2/meta_data/cat02_lookup.json",
    (route) =>
      route.fulfill({
        json: [{ cat02_code: "01001", cat02_name: "コシヒカリ" }],
      }),
  );
  await page.route(
    "**/commodity_prices/v2/transformed_data/area_13100_items_01001_01001.json",
    (route) =>
      route.fulfill({
        json: [
          {
            cat02_code: "01001",
            cat02_name: "コシヒカリ",
            area_code: "13100",
            area_name: "特別区部",
            time_value_pairs: [
              { time: "2025000101", value: "200" },
              { time: "2025000201", value: "210" },
            ],
          },
        ],
      }),
  );
  await page.goto("/articles/commodity_prices");
  await expect(
    page.getByRole("heading", { name: "モノの価格の推移を見る" }),
  ).toBeVisible();
  await expect(
    page.getByRole("img", { name: /コシヒカリ【特別区部】/ }).locator("canvas"),
  ).toBeVisible();
  await expect(
    page.getByRole("img", { name: /さんま漁獲量/ }).locator("canvas"),
  ).toBeVisible();
});

test("consumer price index article renders its overall index chart", async ({
  page,
}) => {
  await mockCpi(page);
  await page.goto("/articles/consumer_price_index");
  await expect(
    page.getByRole("heading", { name: "消費者物価指数の推移を見る" }),
  ).toBeVisible();
  await expect(
    page.getByRole("img", { name: /総合【全国】/ }).locator("canvas"),
  ).toBeVisible();
});

test("understanding CPI article renders the national index chart", async ({
  page,
}) => {
  await mockCpi(page);
  await page.goto("/articles/understanding_consumer_price_index");
  await expect(
    page.getByRole("heading", { name: "消費者物価指数を理解する" }),
  ).toBeVisible();
  await expect(
    page
      .getByRole("img", {
        name: /生鮮食品及びエネルギーを除く総合【全国】/,
      })
      .locator("canvas"),
  ).toBeVisible();
});
