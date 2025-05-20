import { expect, test } from "@playwright/test";

test("home page has expected h1", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("h1")).toBeVisible();
});

test("commodity_prices page has expected content and behavior", async ({
  page,
}) => {
  await page.goto("/commodity_prices");
  await expect(page.locator("h1")).toHaveText("主要品目の都市別小売価格");
  await expect(page.locator("text=自動で追加する")).toBeVisible();

  const selectElements = await page.locator("select").all();

  // Scenario 1: 未選択
  // Select the first option in each select and check canvas is hidden
  for (const selectElement of selectElements) {
    const options = await selectElement.locator("option").all();
    expect(options.length).toBeGreaterThan(1);

    const firstOptionValue = await options[0].getAttribute("value");
    await selectElement.selectOption({ value: firstOptionValue });
  }
  const canvasAfterFirstOption = page.locator("canvas");
  await expect(canvasAfterFirstOption).toBeHidden();

  // Scenario 2: Select the second option in each select and check canvas is visible
  for (const selectElement of selectElements) {
    const options = await selectElement.locator("option").all();
    expect(options.length).toBeGreaterThan(1);

    const secondOptionValue = await options[1].getAttribute("value");
    await selectElement.selectOption({ value: secondOptionValue });
  }

  const canvasAfterSecondOption = page.locator("canvas");
  await expect(canvasAfterSecondOption).toBeVisible();
});
