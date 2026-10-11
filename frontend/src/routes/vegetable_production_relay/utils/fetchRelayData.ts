import {
  PREFECTURE_GEOJSON_DATA_PATH,
  VEGETABLE_PRODUCT_INDEX_PATH,
  VEGETABLE_PRODUCTION_RELAY_DATA_PATH,
} from "../constants/constants";
import type {
  AggregationData,
  GeoCollection,
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
  if (!/^[a-z0-9_-]+$/.test(key)) {
    return Promise.reject(new Error("品目の指定が正しくありません"));
  }
  return fetchJson<AggregationData>(
    `${VEGETABLE_PRODUCTION_RELAY_DATA_PATH}/${key}.json`,
    "データ",
    signal,
  );
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
