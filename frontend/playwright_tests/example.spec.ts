// @ts-check
import { expect, test } from "@playwright/test";

test("top page and about page", async ({ page }) => {
  await page.goto("http://localhost:3000/");
  const topPageHeading = page.getByText("オープンデータを可視化していく");
  await expect(topPageHeading).toBeVisible();
  await page.getByRole("link", { name: "このサイトについて" }).click();
  const aboutPageHeading = page.getByText("このサイトについて");
  await expect(aboutPageHeading).toBeVisible();
  await page.getByRole("link", { name: "Website Logo Open Data Viz" }).click();
});

test("vegetable production relay page", async ({ page }) => {
  test.slow();
  await page.goto("/");
  await page.getByRole("link", { name: "野菜の産地リレー" }).click();
  const vegetableRelayHeader = page.getByText("野菜の産地リレー");
  await expect(vegetableRelayHeader).toBeVisible();
  await page
    .locator(
      "g:nth-child(51) > .VegetableProductionRelay_prefecture_circle__V1_Ph"
    )
    .click();
  await page
    .locator(
      "g:nth-child(9) > .VegetableProductionRelay_prefecture_path__icOqL"
    )
    .click();
  await page.getByRole("link", { name: "Website Logo Open Data Viz" }).click();
});

test("employment page", async ({ page }) => {
  await page.goto("http://localhost:3000/");
  await page
    .getByRole("link", { name: "年齢階級，教育，雇用形態別雇用者数(2002年～)" })
    .click();
  const workersByGroupsHeader = page.getByText(
    "年齢階級，教育，雇用形態別雇用者数(2002年～)"
  );
  await expect(workersByGroupsHeader).toBeVisible();
  await page.getByRole("button", { name: "追加" }).click();
  await page.locator("#SvgjsPath1365").click();
  await page.getByRole("link", { name: "Website Logo Open Data Viz" }).click();
});

test("commodity prices page", async ({ page }) => {
  await page.goto("http://localhost:3000/");
  await page.getByRole("link", { name: "小売物価統計調査（動向編）" }).click();
  const commodityPricesHeader = page.getByText("小売物価統計調査（動向編）");
  await expect(commodityPricesHeader).toBeVisible();
  await page.getByLabel("品目").click();
  await page
    .getByRole("option", { name: "1001 うるち米(単一原料米,「コシヒカリ」)" })
    .click();
  await page.getByLabel("地域").click();
  await page.getByRole("option", { name: "札幌市" }).click();
  await page.getByRole("button", { name: "追加" }).click();
  await page.locator("#SvgjsSvg7938").click();
  await page.getByRole("link", { name: "Website Logo Open Data Viz" }).click();
});
