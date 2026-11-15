import { expect, test } from "@playwright/test";

const visualizationPages = [
  ["commodity-prices", "/commodity_prices"],
  ["workers", "/workers_by_age_employment_type_education_sex"],
  ["asset-distribution", "/asset_distribution"],
  ["vegetable-relay", "/vegetable_production_relay"],
  ["social-security", "/social_security_budget_flow"],
];

const viewportWidths = [320, 390, 768, 799, 801, 1024, 1279, 1281, 1440];

test("all visualization pages retain source access and avoid page overflow", async ({
  page,
}, testInfo) => {
  await page.route("http://127.0.0.1:8000/**", (route) =>
    route.fulfill({ status: 503, body: "data unavailable" }),
  );

  for (const [slug, path] of visualizationPages) {
    await test.step(slug, async () => {
      for (const width of viewportWidths) {
        await page.setViewportSize({ width, height: 900 });
        await page.goto(path);
        await expect(
          page.getByRole("heading", { name: "データについて" }),
        ).toBeVisible();
        const homeLink = page.getByRole("link", { name: "ホームに戻る" });
        await expect(homeLink).toBeVisible();

        const documentWidth = await page.evaluate(
          () => document.documentElement.scrollWidth,
        );
        expect(documentWidth, `${slug} at ${width}px`).toBeLessThanOrEqual(
          width,
        );

        if ([320, 768, 1440].includes(width)) {
          await page.screenshot({
            path: testInfo.outputPath(`${slug}-${width}px.png`),
            fullPage: true,
          });
        }
      }
    });
  }
});
