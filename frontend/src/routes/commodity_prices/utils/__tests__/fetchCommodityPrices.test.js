import { afterEach, describe, expect, test, vi } from "vitest";
import { fetchCommodityPrices } from "../fetchCommodityPrices.js";

const lookup = Array.from({ length: 11 }, (_, index) => ({
  cat02_code: `item-${index}`,
  cat02_name: `Item ${index}`,
}));
const series = {
  cat02_code: "item-0",
  cat02_name: "Item 0",
  area_code: "area-1",
  area_name: "Area 1",
  time_value_pairs: [{ time: "2025010101", value: "100" }],
};

afterEach(() => vi.unstubAllGlobals());

describe("fetchCommodityPrices", () => {
  test("shares lookup and chunk requests for series in the same chunk", async () => {
    const fetchMock = vi.fn(async (url) => ({
      ok: true,
      status: 200,
      json: async () =>
        String(url).endsWith("cat02_lookup.json") ? lookup : [series],
    }));
    vi.stubGlobal("fetch", fetchMock);

    const [first, second] = await Promise.all([
      fetchCommodityPrices({ cat02Code: "item-0", areaCode: "area-1" }),
      fetchCommodityPrices({ cat02Code: "item-0", areaCode: "area-1" }),
    ]);

    expect(first).toEqual(series);
    expect(second).toEqual(series);
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(String(fetchMock.mock.calls[1][0])).toContain(
      "area_area-1_items_item-0_item-9.json",
    );
  });

  test("returns undefined for an absent series or item", async () => {
    const fetchMock = vi.fn(async (url) => ({
      ok: true,
      status: 200,
      json: async () =>
        String(url).endsWith("cat02_lookup.json") ? lookup : [series],
    }));
    vi.stubGlobal("fetch", fetchMock);

    await expect(
      fetchCommodityPrices({ cat02Code: "item-1", areaCode: "area-2" }),
    ).resolves.toBeUndefined();
    await expect(
      fetchCommodityPrices({ cat02Code: "missing", areaCode: "area-1" }),
    ).resolves.toBeUndefined();
  });

  test("retries a failed chunk request", async () => {
    const fetchMock = vi.fn(async (url) => {
      if (String(url).endsWith("cat02_lookup.json")) {
        return { ok: true, status: 200, json: async () => lookup };
      }
      if (
        fetchMock.mock.calls.filter(([calledUrl]) => calledUrl === url)
          .length === 1
      ) {
        return { ok: false, status: 503, json: async () => [] };
      }
      return {
        ok: true,
        status: 200,
        json: async () => [{ ...series, area_code: "area-retry" }],
      };
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(
      fetchCommodityPrices({ cat02Code: "item-0", areaCode: "area-retry" }),
    ).rejects.toThrow("HTTP 503");
    await expect(
      fetchCommodityPrices({ cat02Code: "item-0", areaCode: "area-retry" }),
    ).resolves.toEqual({ ...series, area_code: "area-retry" });
  });
});
