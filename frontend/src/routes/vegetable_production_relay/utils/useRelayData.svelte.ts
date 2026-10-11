import { onMount } from "svelte";
import type { AggregationData, GeoCollection } from "../constants/types";
import { fetchDaikonAggregation, fetchPrefectureGeojson } from "./fetchRelayData";

function toError(reason: unknown, fallback: string): Error {
  return reason instanceof Error ? reason : new Error(fallback);
}

export function useRelayData() {
  let aggregation = $state.raw<AggregationData | undefined>(undefined);
  let geojson = $state.raw<GeoCollection | undefined>(undefined);
  let isLoading = $state(true);
  let error = $state<Error | undefined>(undefined);
  let mapError = $state<Error | undefined>(undefined);

  onMount(async () => {
    const [aggregationResult, geojsonResult] = await Promise.allSettled([
      fetchDaikonAggregation(),
      fetchPrefectureGeojson(),
    ]);

    if (aggregationResult.status === "fulfilled") {
      aggregation = aggregationResult.value;
    } else {
      error = toError(aggregationResult.reason, "データを読み込めませんでした");
    }

    if (geojsonResult.status === "fulfilled") {
      geojson = geojsonResult.value;
    } else {
      mapError = toError(geojsonResult.reason, "地図データを読み込めませんでした");
    }

    isLoading = false;
  });

  return {
    get aggregation() {
      return aggregation;
    },
    get geojson() {
      return geojson;
    },
    get isLoading() {
      return isLoading;
    },
    get error() {
      return error;
    },
    get mapError() {
      return mapError;
    },
  };
}
