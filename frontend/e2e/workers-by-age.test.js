import { expect, test } from "@playwright/test";

const options = {
  metadata: {
    stats_data_id: "0003006608",
    title: "年齢階級，教育，雇用形態別雇用者数(2002年～)",
    statistics_name: "労働力調査",
    unit: "万人",
    period: "2002～2025年",
    fetched_at: "2026-10-17T00:00:00Z",
  },
  dimensions: {
    age: [{ code: "00", name: "総数" }],
    employment_type: [{ code: "00", name: "雇用者" }],
    education: [
      { code: "00", name: "総数" },
      { code: "03", name: "短大・高専" },
      { code: "07", name: "短大・高専" },
    ],
    sex: [{ code: "0", name: "総数" }],
  },
  series: [
    {
      age_code: "00",
      employment_type_code: "00",
      education_code: "00",
      sex_code: "0",
      path: "age-00_employment-00_education-00_sex-0.json",
    },
    { age_code: "00", employment_type_code: "00", education_code: "03", sex_code: "0", path: "series-03.json" },
    { age_code: "00", employment_type_code: "00", education_code: "07", sex_code: "0", path: "series-07.json" },
  ],
};

test("workers by age page loads options, adds a series, and removes it", async ({ page }) => {
  await page.route("**/workers_by_age_employment_type_education_sex/meta_data/options.json", (route) =>
    route.fulfill({ json: options }),
  );
  await page.route("**/workers_by_age_employment_type_education_sex/transformed_data/**", (route) =>
    route.fulfill({
      json: {
        age_code: "00",
        age_name: "総数",
        employment_type_code: "00",
        employment_type_name: "雇用者",
        education_code: "00",
        education_name: "総数",
        sex_code: "0",
        sex_name: "総数",
        unit: "万人",
        time_value_pairs: [
          { year: "2002", value: 5337 },
          { year: "2003", value: null, reason: "表章単位未満" },
        ],
      },
    }),
  );

  await page.goto("/workers_by_age_employment_type_education_sex");
  await expect(page.locator("h1")).toHaveText("年齢階級，教育，雇用形態別雇用者数");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect(page.getByText("総数・雇用者・総数・総数")).toBeVisible();
  const education = page.getByLabel("教育");
  await education.selectOption("03");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect(page.getByText("総数・雇用者・在学中 > 短大・高専・総数")).toBeVisible();
  await education.selectOption("07");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect(page.getByText("総数・雇用者・卒業 > 短大・高専・総数")).toBeVisible();
  await expect(page.getByText("比較する系列").locator("..").getByRole("listitem")).toHaveCount(3);
  await expect(page.locator('[role="img"][aria-label="雇用者数の年次推移"]')).toBeVisible();
  await page.getByRole("button", { name: /を削除/ }).click();
  await expect(page.getByText("比較する系列")).toHaveCount(0);
});

test("series requests can finish out of order and a removed request cannot replace its retry", async ({ page }) => {
  const held = [];
  await page.route("**/workers_by_age_employment_type_education_sex/meta_data/options.json", (route) =>
    route.fulfill({ json: options }),
  );
  await page.route("**/workers_by_age_employment_type_education_sex/transformed_data/**", async (route) => {
    held.push(route);
  });

  await page.goto("/workers_by_age_employment_type_education_sex");
  const education = page.getByLabel("教育");
  await education.selectOption("03");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect.poll(() => held.length).toBe(1);
  await education.selectOption("07");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect.poll(() => held.length).toBe(2);

  await held[1].fulfill({ json: { time_value_pairs: [{ year: "2003", value: 7 }] } });
  await held[0].fulfill({ json: { time_value_pairs: [{ year: "2002", value: 3 }] } });
  await expect(page.getByText("取得中…")).toHaveCount(0);
  await expect(page.getByText("総数・雇用者・在学中 > 短大・高専・総数")).toBeVisible();
  await expect(page.getByText("総数・雇用者・卒業 > 短大・高専・総数")).toBeVisible();

  await page.getByRole("button", { name: /在学中 > 短大・高専.*を削除/ }).click();
  await education.selectOption("03");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect.poll(() => held.length).toBe(3);
  await page.getByRole("button", { name: /在学中 > 短大・高専.*を削除/ }).click();
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect.poll(() => held.length).toBe(4);

  await held[3].fulfill({ json: { time_value_pairs: [{ year: "2004", value: 4 }] } });
  await expect(page.getByText("取得中…")).toHaveCount(0);
  await held[2].fulfill({ json: { invalid: true } });
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(page.getByText("総数・雇用者・在学中 > 短大・高専・総数")).toBeVisible();
});

test("series 404s are distinct from missing options and all-null series remain visible", async ({ page }) => {
  await page.route("**/workers_by_age_employment_type_education_sex/meta_data/options.json", (route) =>
    route.fulfill({ json: options }),
  );
  await page.route("**/workers_by_age_employment_type_education_sex/transformed_data/**", (route) => {
    const url = route.request().url();
    if (url.endsWith("age-00_employment-00_education-00_sex-0.json")) {
      return route.fulfill({ status: 404, body: "not found" });
    }
    if (url.endsWith("series-03.json")) {
      return route.fulfill({ json: { time_value_pairs: [{ year: "2002", value: null, reason: "該当数値なし" }] } });
    }
    return route.fulfill({ json: { time_value_pairs: [{ year: "2003", value: 8 }] } });
  });

  await page.goto("/workers_by_age_employment_type_education_sex");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect(page.getByRole("alert")).toContainText("系列データを取得できませんでした (HTTP 404)");
  await page.getByRole("button", { name: /総数・雇用者・総数・総数を削除/ }).click();

  const education = page.getByLabel("教育");
  await education.selectOption("03");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect(page.getByText("数値を表示できる年がありません（すべて欠損）")).toBeVisible();
  await education.selectOption("07");
  await page.getByRole("button", { name: "系列を追加" }).click();
  await expect(page.getByText("総数・雇用者・卒業 > 短大・高専・総数")).toBeVisible();
  await page.getByLabel("グラフの向き").selectOption("horizontal");
  await expect(page.locator('[role="img"][aria-label="雇用者数の年次推移"]')).toBeVisible();
});
