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
  await page.route(
    "**/commodity_prices/v2/meta_data/cat02_lookup.json",
    (route) =>
      route.fulfill({
        json: [{ cat02_code: "0001", cat02_name: "食パン" }],
      }),
  );
  await page.route(
    "**/commodity_prices/v2/meta_data/area_lookup.json",
    (route) =>
      route.fulfill({
        json: [{ area_code: "01100", area_name: "札幌市" }],
      }),
  );
  await page.route(
    "**/commodity_prices/v2/transformed_data/area_01100_items_0001_0001.json",
    (route) =>
      route.fulfill({
        json: [
          {
            cat02_code: "0001",
            cat02_name: "食パン",
            area_code: "01100",
            area_name: "札幌市",
            time_value_pairs: [
              { time: "2025000101", value: "200" },
              { time: "2025000201", value: "210" },
            ],
          },
        ],
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
  await expect(page.locator(".selector-panel")).toHaveAttribute("open", "");
  const area = page.getByRole("checkbox", { name: "札幌市" });
  await expect(area).toBeEnabled();
  await expect(
    page.getByText("品目と地域を選ぶと、ここにグラフが表示されます。"),
  ).toBeVisible();

  await page.getByRole("checkbox", { name: "食パン" }).check();
  await area.check();

  await expect(page.getByText("食パン【札幌市】")).toBeVisible();
  await expect(page.locator(".chart-scroll-container")).toBeVisible();
  const fitToggle = page.getByRole("button", { name: "画面幅に収める" });
  await expect(fitToggle).toHaveAttribute("aria-pressed", "true");
  await fitToggle.click();
  await expect(fitToggle).toHaveAttribute("aria-pressed", "false");
  const scrollRegion = page.getByRole("region", {
    name: "価格推移グラフ。左右にスクロールできます",
  });
  await expect(scrollRegion).toBeVisible();
  const graphDimensions = await scrollRegion.evaluate((element) => ({
    client: element.clientWidth,
    scroll: element.scrollWidth,
  }));
  expect(graphDimensions.scroll).toBeGreaterThan(graphDimensions.client);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth),
  ).toBeLessThanOrEqual(375);
  await fitToggle.click();
  await expect(fitToggle).toHaveAttribute("aria-pressed", "true");
  await expect(scrollRegion).toHaveCount(0);
});

test("social security chart uses the shared tooltip style on hover", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 768, height: 900 });
  await page.route("**/social_security_budget_flow/catalog.json", (route) =>
    route.fulfill({
      json: {
        programs: [
          {
            id: "test-program",
            name: "テスト制度",
            income: [{ id: "I11", value: 100 }],
            expenditure: [{ id: "E27", value: 80 }],
          },
        ],
      },
    }),
  );

  await page.goto("/social_security_budget_flow");
  const item = page.getByRole("button", { name: /拠出／被保険者/ });
  await item.hover();
  await expect(item).toHaveClass(/highlighted/);
  const tooltip = page.getByRole("tooltip", { name: "項目詳細" });
  await expect(tooltip).toBeVisible();
  await expect(tooltip).toContainText("拠出／被保険者");
  await page.screenshot({
    path: testInfo.outputPath("sankey-hover-tooltip.png"),
  });

  const tooltipStyle = await tooltip.evaluate((element) => {
    const style = getComputedStyle(element);
    const root = getComputedStyle(document.documentElement);
    return {
      background: style.backgroundColor,
      expectedBackground: root.getPropertyValue("--tooltip-background").trim(),
      border: style.borderTopColor,
      expectedBorder: root.getPropertyValue("--tooltip-border-color").trim(),
      fontSize: style.fontSize,
      expectedFontSize: root.getPropertyValue("--tooltip-font-size").trim(),
      shadow: style.boxShadow,
    };
  });
  expect(tooltipStyle.background).toBe("rgb(255, 255, 255)");
  expect(tooltipStyle.border).toBe("rgb(201, 213, 233)");
  expect(tooltipStyle.fontSize).toBe("14px");
  expect(tooltipStyle.shadow).not.toBe("none");
  expect(tooltipStyle.expectedBackground).toBe("#fff");
  expect(tooltipStyle.expectedBorder).toBe("#c9d5e9");
  expect(tooltipStyle.expectedFontSize).toBe("0.875rem");
});
