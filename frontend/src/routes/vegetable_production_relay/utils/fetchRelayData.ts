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

async function fetchJson<T>(path: string, label: string): Promise<T> {
  const response = await fetch(path, {
    method: "GET",
    headers: { "Content-Type": "application/json" },
  });

  if (!response.ok) {
    throw new Error(`${label} HTTP ${response.status}`);
  }

  return response.json();
}

export function fetchRelayProductIndex(): Promise<RelayProductIndex> {
  return fetchJson<RelayProductIndex>(VEGETABLE_PRODUCT_INDEX_PATH, "品目一覧");
}

export function fetchRelayAggregation(key: string): Promise<AggregationData> {
  if (!/^[a-z0-9_-]+$/.test(key)) {
    return Promise.reject(new Error("品目の指定が正しくありません"));
  }
  return fetchJson<AggregationData>(
    `${VEGETABLE_PRODUCTION_RELAY_DATA_PATH}/${key}.json`,
    "データ",
  );
}

export function fetchPrefectureGeojson(): Promise<GeoCollection> {
  return fetchJson<GeoCollection>(PREFECTURE_GEOJSON_DATA_PATH, "地図データ");
}
