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
    if (
      result.schema_version !== 2 ||
      result.commodity !== key ||
      result.consignor_prefecture_code !== code ||
      !Array.isArray(result.months) ||
      result.months.length !== 36 ||
      new Set(result.months.map((row) => row.month)).size !== 36 ||
      result.months.some(
        (row) =>
          !/^\d{4}-\d{2}$/.test(row.month) ||
          !Array.isArray(row.contributions) ||
          row.contributions.length !== 9 ||
          new Set(row.contributions.map((cell) => cell.consumer_region_code))
            .size !== 9 ||
          row.contributions.some(
            (cell) =>
              !CONSUMER_REGIONS.some(
                ([regionCode]) => regionCode === cell.consumer_region_code,
              ) ||
              typeof cell.consumer_region_name !== "string" ||
              typeof cell.raw_value !== "string" ||
              ![
                "reported",
                "below_display_unit",
                "no_fact",
                "unknown_or_uncollected",
              ].includes(cell.value_kind) ||
              (cell.value_kind === "reported" &&
                (!cell.raw_value.trim() ||
                  !Number.isFinite(Number(cell.raw_value)) ||
                  Number(cell.raw_value) < 0)),
          ),
      )
    ) {
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
