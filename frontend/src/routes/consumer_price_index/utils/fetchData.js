import {
  CPI_DATA_PATH,
  CPI_ITEMS_PATH,
  CPI_REGIONS_PATH,
} from "../constants/constants.js";

const chunkRequests = new Map();

/** @param {string} url @param {{ missingIsEmpty?: boolean }} [options] */
async function fetchJson(url, { missingIsEmpty = false } = {}) {
  const response = await fetch(url);
  if (response.status === 404 && missingIsEmpty) return undefined;
  if (!response.ok) {
    throw new Error(`データを取得できませんでした (HTTP ${response.status})`);
  }
  return response.json();
}

export async function fetchLookups() {
  const [items, regions] = await Promise.all([
    fetchJson(CPI_ITEMS_PATH),
    fetchJson(CPI_REGIONS_PATH),
  ]);
  return { items, regions };
}

/** @param {{ itemCode: string, areaCode: string, itemIndex: number, itemCount: number }} params */
export async function fetchSeries({ itemCode, areaCode, itemIndex, itemCount }) {
  const firstItemIndex = Math.floor(itemIndex / 10) * 10;
  const lastItemIndex = Math.min(firstItemIndex + 9, itemCount - 1);
  const padded = (value) => String(value).padStart(4, "0");
  const url = `${CPI_DATA_PATH}/series/area_${areaCode}_items_${padded(firstItemIndex)}_${padded(lastItemIndex)}.json`;
  let request = chunkRequests.get(url);
  if (!request) {
    request = fetchJson(url, { missingIsEmpty: true });
    chunkRequests.set(url, request);
    request.catch(() => chunkRequests.delete(url));
  }
  const chunk = await request;
  return chunk?.find(
    (series) => series.itemCode === itemCode && series.areaCode === areaCode,
  );
}
