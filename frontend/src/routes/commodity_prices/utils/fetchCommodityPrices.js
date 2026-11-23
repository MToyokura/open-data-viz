import {
  CAT02_LOOKUP_DATA_PATH,
  COMMODITY_PRICES_TRANSFORMED_DATA_PATH,
} from "../constants/constants.js";

/** @typedef {import("../constants/types.js").CommodityPrices} CommodityPrices */
/** @typedef {import("../constants/types.js").Cat02Lookup} Cat02Lookup */

// Cache both pending and fulfilled requests. Rejected requests are removed so
// callers can retry after a transient network or server failure.
const requestsByUrl = new Map();

/** @param {string} url */
function fetchJson(url) {
  const cached = requestsByUrl.get(url);
  if (cached) return cached;

  const request = fetch(url, {
    method: "GET",
    headers: { "Content-Type": "application/json" },
  }).then(async (response) => {
    if (response.status === 404) return undefined;
    if (!response.ok) {
      throw new Error(`データを取得できませんでした (HTTP ${response.status})`);
    }
    return response.json();
  });
  requestsByUrl.set(url, request);
  request.catch(() => {
    if (requestsByUrl.get(url) === request) requestsByUrl.delete(url);
  });
  return request;
}

/** @type {Promise<Cat02Lookup> | undefined} */
let cat02LookupRequest;

/** @returns {Promise<Cat02Lookup>} */
function fetchCat02Lookup() {
  cat02LookupRequest ??= /** @type {Promise<Cat02Lookup>} */ (
    fetchJson(CAT02_LOOKUP_DATA_PATH).catch(
      /** @param {unknown} error */ (error) => {
        cat02LookupRequest = undefined;
        throw error;
      },
    )
  );
  return cat02LookupRequest;
}

/**
 * Fetches one series from its area chunk.
 * @param {{ cat02Code: string, areaCode: string }} params
 * @returns {Promise<CommodityPrices | undefined>}
 */
export async function fetchCommodityPrices({ cat02Code, areaCode }) {
  try {
    const lookup = await fetchCat02Lookup();
    const itemIndex = lookup.findIndex((item) => item.cat02_code === cat02Code);
    // Missing item codes retain the old missing-series result.
    if (itemIndex < 0) return undefined;
    const firstIndex = Math.floor(itemIndex / 10) * 10;
    const first = lookup[firstIndex].cat02_code;
    const last = lookup[Math.min(firstIndex + 9, lookup.length - 1)].cat02_code;
    const url = `${COMMODITY_PRICES_TRANSFORMED_DATA_PATH}/area_${areaCode}_items_${first}_${last}.json`;
    /** @type {CommodityPrices[] | undefined} */
    const chunk = await fetchJson(url);
    if (!chunk) return undefined;
    return chunk.find(
      (series) =>
        series.cat02_code === cat02Code && series.area_code === areaCode,
    );
  } catch (error) {
    console.error("Error fetching data:", error);
    throw error;
  }
}
