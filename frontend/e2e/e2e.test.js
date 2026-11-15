import { expect, test } from "@playwright/test";

test("home page has expected h1", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toBeVisible();
});

test("commodity_prices page has expected content and behavior", async ({
  page,
}) => {
  await page.route("**/commodity_prices/meta_data/cat02_lookup.json", (route) =>
    route.fulfill({
      json: [{ cat02_code: "0001", cat02_name: "食パン" }],
    }),
  );
  await page.route("**/commodity_prices/meta_data/area_lookup.json", (route) =>
    route.fulfill({
      json: [{ area_code: "01100", area_name: "札幌市" }],
    }),
  );
  await page.route(
    "**/commodity_prices/transformed_data/commodity_prices_0001_01100.json",
    (route) =>
      route.fulfill({
        json: {
          cat02_code: "0001",
          cat02_name: "食パン",
          area_code: "01100",
          area_name: "札幌市",
          time_value_pairs: [{ time: "2025-01-01", value: "200" }],
        },
      }),
  );

  await page.goto("/commodity_prices");
  await expect(page.locator("h1")).toHaveText("主要品目の都市別小売価格");
  const area = page.getByRole("checkbox", { name: "札幌市" });
  await expect(area).toBeDisabled();
  await expect(
    page.getByText("品目と地域を選ぶと、ここにグラフが表示されます。"),
  ).toBeVisible();

  await page.getByRole("radio", { name: "食パン" }).check();
  await expect(area).toBeEnabled();
  await area.check();

  await expect(page.getByText("食パン【札幌市】")).toBeVisible();
  await expect(page.locator(".chart-scroll-container")).toBeVisible();
});
