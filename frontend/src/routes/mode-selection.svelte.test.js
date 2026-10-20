import { expect, test, vi } from "vitest";
import { flushSync } from "svelte";
import { fireEvent, render, screen, within } from "@testing-library/svelte";
import AssetPage from "./asset_distribution/+page.svelte";
import WorkersPage from "./workers_by_age_employment_type_education_sex/+page.svelte";

vi.mock(
  "./asset_distribution/DistributionChart.svelte",
  () => import("#lib/Spinner.svelte"),
);
vi.mock(
  "./workers_by_age_employment_type_education_sex/BarChart.svelte",
  () => import("#lib/Spinner.svelte"),
);

vi.mock("./asset_distribution/utils/data.js", async (importOriginal) => {
  const original =
    /** @type {typeof import("./asset_distribution/utils/data.js")} */ (
      await importOriginal()
    );
  return {
    ...original,
    fetchAssetDistribution: async () => ({
      metadata: {},
      dimensions: [
        ...["cat01", "cat05"].map((id) => ({
          id,
          label: id === "cat01" ? "年齢階級" : "世帯人員",
          role: "filter",
          default_code: "0",
          categories: ["0", "1", "2"].map((code, order) => ({
            code,
            label: code,
            order,
          })),
          comparison_sets: [
            { id: "groups", label: "区分", category_codes: ["1", "2"] },
          ],
        })),
        {
          id: "cat04",
          role: "asset_class",
          categories: [],
          display_category_codes: [],
        },
      ],
      combination_index: { dimension_ids: ["cat01", "cat05"], count: 9 },
      chunks: [{ path: "items.json", start_index: 0, count: 9 }],
    }),
    fetchCombinationItems: async () => [],
  };
});

vi.mock(
  "./workers_by_age_employment_type_education_sex/utils/fetchData.js",
  () => ({
    fetchOptions: async () => ({
      metadata: {},
      education_display_hierarchy: [],
      education_comparison_sets: [],
      dimensions: Object.fromEntries(
        ["age", "employment_type", "education", "sex"].map((id) => [
          id,
          ["0", "1", "2"].map((code) => ({ code, name: code })),
        ]),
      ),
      series: ["0", "1", "2"].flatMap((age_code) =>
        ["0", "1", "2"].map((sex_code) => ({
          age_code,
          sex_code,
          employment_type_code: "0",
          education_code: "0",
          path: `${age_code}-${sex_code}.json`,
        })),
      ),
    }),
    fetchSeries: async () => ({ time_value_pairs: [] }),
  }),
);

for (const config of [
  {
    name: "assets",
    component: AssetPage,
    fixedLabel: "年齢階級",
    compareId: "cat05",
    group: "比較する区分",
  },
  {
    name: "workers",
    component: WorkersPage,
    fixedLabel: "年齢階級",
    compareId: "sex",
    group: "性別の区分",
  },
]) {
  test(`${config.name}: responsive panels preserve manual choices and clean up viewport listeners`, async () => {
    const query = window.matchMedia("(max-width: 50rem)");
    Object.defineProperty(query, "matches", { value: true, writable: true });
    vi.mocked(window.matchMedia).mockReturnValueOnce(query);
    const { container, unmount } = render(config.component);
    await screen.findByLabelText(config.fixedLabel);
    const selector = container.querySelector("details.selector-panel");
    const comparison = container.querySelector("details.added-items");
    expect(selector).not.toHaveAttribute("open");
    expect(comparison).toHaveAttribute("open");

    const updateViewport = vi.mocked(query.addEventListener).mock.calls[0][1];
    const resize = (/** @type {boolean} */ matches) => {
      Object.defineProperty(query, "matches", { value: matches });
      flushSync(() => {
        if (typeof updateViewport === "function") {
          updateViewport.call(query, new Event("change"));
        }
      });
    };
    resize(false);
    expect(selector).toHaveAttribute("open");
    expect(comparison).toHaveAttribute("open");
    await fireEvent.change(screen.getByLabelText(config.fixedLabel), {
      target: { value: "1" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "系列に追加" }));
    resize(true);
    expect(selector).not.toHaveAttribute("open");
    expect(comparison).not.toHaveAttribute("open");

    await fireEvent.click(
      within(/** @type {HTMLElement} */ (selector)).getByText(/系列を追加：/),
    );
    expect(selector).toHaveAttribute("open");
    await fireEvent.click(screen.getByRole("button", { name: "すべて消去" }));
    expect(selector).toHaveAttribute("open");
    expect(comparison).not.toHaveAttribute("open");
    unmount();
    expect(query.removeEventListener).toHaveBeenCalledWith(
      "change",
      updateViewport,
    );
  });

  test(`${config.name}: each mode retains its own conditions and comparison choices`, async () => {
    render(config.component);
    const fixed = await screen.findByLabelText(config.fixedLabel);
    const modes = within(screen.getByRole("group", { name: "表示モード" }));
    const compare = () =>
      fireEvent.click(modes.getByRole("button", { name: "属性を比較" }));
    const add = () =>
      fireEvent.click(modes.getByRole("button", { name: "系列を追加" }));
    await fireEvent.change(fixed, { target: { value: "1" } });
    await compare();
    expect(fixed).toHaveValue("0");
    await fireEvent.change(fixed, { target: { value: "2" } });
    await fireEvent.change(screen.getByLabelText("比較する属性"), {
      target: { value: config.compareId },
    });
    const choices = within(screen.getByRole("group", { name: config.group }));
    const unchecked = choices.getAllByRole("checkbox")[0];
    await fireEvent.click(unchecked);
    expect(unchecked).not.toBeChecked();
    await add();
    expect(fixed).toHaveValue("1");
    await fireEvent.change(fixed, { target: { value: "0" } });
    await compare();
    expect(fixed).toHaveValue("2");
    expect(screen.getByLabelText("比較する属性")).toHaveValue(config.compareId);
    expect(
      within(screen.getByRole("group", { name: config.group })).getAllByRole(
        "checkbox",
      )[0],
    ).not.toBeChecked();
    await add();
    expect(fixed).toHaveValue("0");
  });
}
