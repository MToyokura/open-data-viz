import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { expect, test, vi } from "vitest";
import Page from "./+page.svelte";

vi.mock("echarts", () => ({
  init: () => ({ setOption: vi.fn(), dispose: vi.fn() }),
}));

test("shows each relevant source note once while any related series is selected", async () => {
  const series = [
    {
      id: "col-b",
      label: "漁獲量計",
      groupPath: [],
      type: "total",
      values: [100],
    },
    {
      id: "col-bh",
      label: "ちだい･きだい",
      groupPath: [],
      type: "detail",
      values: [null],
    },
    {
      id: "col-bi",
      label: "ちだい",
      groupPath: [],
      type: "detail",
      values: [10],
    },
  ];
  const notes = [
    {
      number: 9,
      text: "9.令和元年からは「ちだい」、「きだい」に分離した。",
      seriesIds: ["col-bh", "col-bi", "col-bj"],
      yearRanges: [[2019, 2024]],
    },
  ];
  vi.stubGlobal("fetch", async () =>
    Response.json({ years: [2019], series, notes }),
  );
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      disconnect() {}
    },
  );

  const view = render(Page);
  try {
    const combined = await screen.findByRole("checkbox", {
      name: "ちだい･きだい",
    });
    const separate = screen.getByRole("checkbox", { name: "ちだい" });
    const emptyChart = "漁獲量の推移を比較";
    const heading = "選択中の系列に関する注記";
    expect(
      screen.getByRole("checkbox", { name: "漁獲量計" }),
    ).not.toBeChecked();
    expect(
      screen.getByRole("heading", { name: emptyChart }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: heading })).toBeNull();

    await fireEvent.click(combined);
    expect(screen.queryByRole("heading", { name: emptyChart })).toBeNull();
    const list = screen.getByRole("heading", {
      name: heading,
    }).nextElementSibling;
    if (!(list instanceof HTMLOListElement))
      throw new Error("Missing selected notes list");
    expect(within(list).getByText("2019〜2024年")).toBeInTheDocument();

    await fireEvent.click(separate);
    expect(within(list).getAllByRole("listitem")).toHaveLength(1);

    await fireEvent.click(combined);
    expect(within(list).getAllByRole("listitem")).toHaveLength(1);

    await fireEvent.click(separate);
    expect(screen.queryByRole("heading", { name: heading })).toBeNull();
    expect(
      screen.getByRole("heading", { name: emptyChart }),
    ).toBeInTheDocument();
  } finally {
    view.unmount();
    vi.unstubAllGlobals();
  }
});
