import { expect, test } from "@playwright/test";

const visualizationPages = [
  ["commodity-prices", "/commodity_prices"],
  ["workers", "/workers_by_age_employment_type_education_sex"],
  ["asset-distribution", "/asset_distribution"],
  ["vegetable-relay", "/vegetable_production_relay"],
  ["social-security", "/social_security_budget_flow"],
];

async function tabTo(page, locator, label) {
  for (let index = 0; index < 40; index += 1) {
    if (
      await locator.evaluate((element) => element === document.activeElement)
    ) {
      return;
    }
    await page.keyboard.press("Tab");
  }
  throw new Error(`Could not reach ${label} by keyboard`);
}

test("home, source, and license links show keyboard focus at 320px", async ({
  page,
}) => {
  await page.setViewportSize({ width: 320, height: 900 });
  await page.route("http://127.0.0.1:8000/**", (route) =>
    route.fulfill({ status: 503, body: "data unavailable" }),
  );

  for (const [slug, path] of visualizationPages) {
    await test.step(slug, async () => {
      await page.goto(path);
      const homeLink = page.getByRole("link", { name: "ホームに戻る" });
      await page.keyboard.press("Tab");
      await expect(homeLink).toBeFocused();
      await expect(homeLink).toHaveCSS("outline-style", "solid");

      const sourceLink = page.locator('a[href^="https://"]').first();
      await tabTo(page, sourceLink, "source link");
      await expect(sourceLink).toHaveCSS("outline-style", "solid");

      const licenseLink = page.getByRole("link", { name: "CC0" });
      await tabTo(page, licenseLink, "license link");
      await expect(licenseLink).toHaveCSS("outline-style", "solid");
    });
  }
});
