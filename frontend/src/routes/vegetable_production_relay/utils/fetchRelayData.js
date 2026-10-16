import { CONSUMER_REGIONS } from "./aggregateSankey.js";
import {
  PREFECTURE_GEOJSON_DATA_PATH,
  VEGETABLE_PRODUCT_INDEX_PATH,
  VEGETABLE_PRODUCTION_RELAY_DATA_PATH,
} from "../constants/constants.js";

/** @typedef {import("../constants/types.js").AggregationData} AggregationData */
/** @typedef {import("../constants/types.js").GeoCollection} GeoCollection */
/** @typedef {import("../constants/types.js").PrefectureDetail} PrefectureDetail */
/** @typedef {import("../constants/types.js").RelayProductIndex} RelayProductIndex */

const DETAIL_MONTH_COUNT = 36;
const CONTRIBUTION_COUNT = 9;
const VALID_VALUE_KINDS = new Set([
  "reported",
  "below_display_unit",
  "no_fact",
  "unknown_or_uncollected",
]);

/** @param {PrefectureDetail["months"][number]["contributions"][number]} cell */
function isValidContribution(cell) {
  if (
    !CONSUMER_REGIONS.some(
      ([regionCode]) => regionCode === cell.consumer_region_code,
    ) ||
    typeof cell.consumer_region_name !== "string" ||
    typeof cell.raw_value !== "string" ||
    !VALID_VALUE_KINDS.has(cell.value_kind)
  ) {
    return false;
  }

  if (cell.value_kind !== "reported") return true;
  const value = Number(cell.raw_value);
  return cell.raw_value.trim() !== "" && Number.isFinite(value) && value >= 0;
}

/** @param {PrefectureDetail["months"][number]} month */
function isValidDetailMonth(month) {
  return (
    /^\d{4}-\d{2}$/.test(month.month) &&
    Array.isArray(month.contributions) &&
    month.contributions.length === CONTRIBUTION_COUNT &&
    new Set(month.contributions.map((cell) => cell.consumer_region_code))
      .size === CONTRIBUTION_COUNT &&
    month.contributions.every(isValidContribution)
  );
}

/** @param {PrefectureDetail} result @param {string} key @param {string} code */
function isValidPrefectureDetail(result, key, code) {
  return (
    result.schema_version === 2 &&
    result.commodity === key &&
    result.consignor_prefecture_code === code &&
    Array.isArray(result.months) &&
    result.months.length === DETAIL_MONTH_COUNT &&
    new Set(result.months.map((month) => month.month)).size ===
      DETAIL_MONTH_COUNT &&
    result.months.every(isValidDetailMonth)
  );
}

/** @template T @param {string} path @param {string} label @param {AbortSignal} [signal] @returns {Promise<T>} */
async function fetchJson(path, label, signal) {
  const response = await fetch(path, { signal });

  if (!response.ok) {
    throw new Error(`${label} HTTP ${response.status}`);
  }

  return response.json();
}

/** @param {AbortSignal} [signal] @returns {Promise<RelayProductIndex>} */
export function fetchRelayProductIndex(signal) {
  return /** @type {Promise<RelayProductIndex>} */ (
    fetchJson(VEGETABLE_PRODUCT_INDEX_PATH, "品目一覧", signal)
  );
}

/** @param {string} key @param {AbortSignal} [signal] @returns {Promise<AggregationData>} */
export function fetchRelayAggregation(key, signal) {
  if (!/^[a-z0-9_-]+$/.test(key)) {
    return Promise.reject(new Error("品目の指定が正しくありません"));
  }
  const request = /** @type {Promise<AggregationData>} */ (
    fetchJson(
      `${VEGETABLE_PRODUCTION_RELAY_DATA_PATH}/${key}.summary.json`,
      "データ",
      signal,
    )
  );
  return request.then((result) => {
    if (result.schema_version !== 2 || result.commodity !== key) {
      throw new Error("品目データの形式が正しくありません");
    }
    return result;
  });
}

/** @param {string} key @param {string} code @param {AbortSignal} [signal] @returns {Promise<PrefectureDetail>} */
export function fetchRelayPrefectureDetail(key, code, signal) {
  if (
    !/^[a-z0-9_-]+$/.test(key) ||
    !/^10(?:0[2-9]|[1-3][0-9]|4[0-8])$/.test(code)
  ) {
    return Promise.reject(
      new Error("品目または都道府県の指定が正しくありません"),
    );
  }
  const request = /** @type {Promise<PrefectureDetail>} */ (
    fetchJson(
      `${VEGETABLE_PRODUCTION_RELAY_DATA_PATH}/${key}/details/${code}.json`,
      "産地内訳",
      signal,
    )
  );
  return request.then((result) => {
    if (!isValidPrefectureDetail(result, key, code)) {
      throw new Error("産地内訳データの形式が正しくありません");
    }
    return result;
  });
}

/** @param {AbortSignal} [signal] @returns {Promise<GeoCollection>} */
export function fetchPrefectureGeojson(signal) {
  return /** @type {Promise<GeoCollection>} */ (
    fetchJson(PREFECTURE_GEOJSON_DATA_PATH, "地図データ", signal)
  );
}
