import {
  DAIKON_AGGREGATION_DATA_PATH,
  PREFECTURE_GEOJSON_DATA_PATH,
} from "../constants/constants";
import type { AggregationData, GeoCollection } from "../constants/types";

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

export function fetchDaikonAggregation(): Promise<AggregationData> {
  return fetchJson<AggregationData>(DAIKON_AGGREGATION_DATA_PATH, "データ");
}

export function fetchPrefectureGeojson(): Promise<GeoCollection> {
  return fetchJson<GeoCollection>(PREFECTURE_GEOJSON_DATA_PATH, "地図データ");
}
