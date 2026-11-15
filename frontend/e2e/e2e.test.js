import { expect, test } from "@playwright/test";

test("home page has expected h1", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toBeVisible();
});

test("not found pages explain the error and offer a home link", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 320, height: 800 });
  await page.goto("/not-a-real-page");
  await expect(
    page.getByRole("heading", { name: "ページが見つかりません" }),
  ).toBeVisible();
  await expect(
    page.getByText("お探しのページは見つかりませんでした。"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "ホームへ戻る" }),
  ).toHaveAttribute("href", "/");
  await page.screenshot({ path: testInfo.outputPath("404-home-320px.png") });

  await page.goto("/commodity_prices/not-a-real-page");
  await expect(
    page.getByRole("heading", { name: "ページが見つかりません" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "ホームへ戻る" }),
  ).toHaveAttribute("href", "/");
  await page.screenshot({
    path: testInfo.outputPath("404-workspace-320px.png"),
  });
});

test("commodity_prices page has expected content and behavior", async ({
  page,
}) => {
  await page.setViewportSize({ width: 375, height: 812 });
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
  const panelPositions = await page.evaluate(() => ({
    chart: document.querySelector(".chart-panel").getBoundingClientRect().top,
    controls: document.querySelector(".details-column").getBoundingClientRect()
      .top,
  }));
  expect(panelPositions.chart).toBeLessThan(panelPositions.controls);
  await page.locator(".selector-panel > summary").click();
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
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(375);
});
