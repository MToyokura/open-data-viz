import {
  PREFECTURE_GEOJSON_DATA_PATH,
  VEGETABLE_PRODUCT_INDEX_PATH,
  VEGETABLE_PRODUCTION_RELAY_DATA_PATH,
} from "../constants/constants";
import type {
  AggregationData,
  GeoCollection,
  PrefectureDetail,
  RelayProductIndex,
} from "../constants/types";

async function fetchJson<T>(
  path: string,
  label: string,
  signal?: AbortSignal,
): Promise<T> {
  const response = await fetch(path, { signal });

  if (!response.ok) {
    throw new Error(`${label} HTTP ${response.status}`);
  }

  return response.json();
}

export function fetchRelayProductIndex(
  signal?: AbortSignal,
): Promise<RelayProductIndex> {
  return fetchJson<RelayProductIndex>(
    VEGETABLE_PRODUCT_INDEX_PATH,
    "品目一覧",
    signal,
  );
}

export function fetchRelayAggregation(
  key: string,
  signal?: AbortSignal,
): Promise<AggregationData> {
  // here
  if (!/^[a-z0-9_-]+$/.test(key)) {
    return Promise.reject(new Error("品目の指定が正しくありません"));
  }
  return fetchJson<AggregationData>(
    `${VEGETABLE_PRODUCTION_RELAY_DATA_PATH}/${key}.summary.json`,
    "データ",
    signal,
  ).then((result) => {
    if (result.schema_version !== 2 || result.commodity !== key) {
      throw new Error("品目データの形式が正しくありません");
    }
    return result;
  });
}

// here
export function fetchRelayPrefectureDetail(
  key: string,
  code: string,
  signal?: AbortSignal,
): Promise<PrefectureDetail> {
  if (
    !/^[a-z0-9_-]+$/.test(key) ||
    !/^10(?:0[2-9]|[1-3][0-9]|4[0-8])$/.test(code)
  ) {
    return Promise.reject(
      new Error("品目または都道府県の指定が正しくありません"),
    );
  }
  return fetchJson<PrefectureDetail>(
    `${VEGETABLE_PRODUCTION_RELAY_DATA_PATH}/${key}/details/${code}.json`,
    "産地内訳",
    signal,
  ).then((result) => {
    if (
      result.schema_version !== 2 ||
      result.commodity !== key ||
      result.consignor_prefecture_code !== code ||
      !Array.isArray(result.months) ||
      result.months.length !== 36 ||
      new Set(result.months.map((row) => row.month)).size !== 36 ||
      result.months.some(
        (row) =>
          !Array.isArray(row.contributions) || row.contributions.length !== 9,
      )
    ) {
      throw new Error("産地内訳データの形式が正しくありません");
    }
    return result;
  });
}

export function fetchPrefectureGeojson(
  signal?: AbortSignal,
): Promise<GeoCollection> {
  return fetchJson<GeoCollection>(
    PREFECTURE_GEOJSON_DATA_PATH,
    "地図データ",
    signal,
  );
}
