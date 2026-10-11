import { onMount } from "svelte";
import type {
  AggregationData,
  GeoCollection,
  RelayProduct,
} from "../constants/types";
import {
  fetchRelayAggregation,
  fetchRelayProductIndex,
  fetchPrefectureGeojson,
} from "./fetchRelayData";

function toError(reason: unknown, fallback: string): Error {
  return reason instanceof Error ? reason : new Error(fallback);
}

export function useRelayData() {
  let aggregation = $state.raw<AggregationData | undefined>(undefined);
  let products = $state.raw<RelayProduct[]>([]);
  let selectedProduct = $state("");
  let geojson = $state.raw<GeoCollection | undefined>(undefined);
  let isLoading = $state(true);
  let error = $state<Error | undefined>(undefined);
  let mapError = $state<Error | undefined>(undefined);
  let requestId = 0;

  async function selectProduct(key: string) {
    if (!products.some((product) => product.key === key)) return;
    selectedProduct = key;
    const currentRequest = ++requestId;
    isLoading = true;
    error = undefined;
    try {
      const result = await fetchRelayAggregation(key);
      if (currentRequest === requestId) aggregation = result;
    } catch (reason) {
      if (currentRequest === requestId) {
        aggregation = undefined;
        error = toError(reason, "データを読み込めませんでした");
      }
    } finally {
      if (currentRequest === requestId) isLoading = false;
    }
  }

  onMount(async () => {
    const [productIndexResult, geojsonResult] = await Promise.allSettled([
      fetchRelayProductIndex(),
      fetchPrefectureGeojson(),
    ]);

    if (productIndexResult.status === "fulfilled") {
      const index = productIndexResult.value;
      if (index.schema_version !== 1 || !Array.isArray(index.products)) {
        error = new Error("品目一覧の形式が正しくありません");
      } else {
        products = index.products.filter(
          (product) =>
            typeof product.key === "string" &&
            /^[a-z0-9_-]+$/.test(product.key) &&
            typeof product.label === "string" &&
            product.label.trim().length > 0,
        );
        const initialProduct =
          products.find((product) => product.key === "daikon") ?? products[0];
        if (initialProduct) await selectProduct(initialProduct.key);
        else error = new Error("表示できる品目がありません");
      }
    } else {
      error = toError(
        productIndexResult.reason,
        "品目一覧を読み込めませんでした",
      );
      isLoading = false;
    }

    if (geojsonResult.status === "fulfilled") {
      geojson = geojsonResult.value;
    } else {
      mapError = toError(
        geojsonResult.reason,
        "地図データを読み込めませんでした",
      );
    }

    isLoading = false;
  });

  return {
    get aggregation() {
      return aggregation;
    },
    get products() {
      return products;
    },
    get selectedProduct() {
      return selectedProduct;
    },
    selectProduct,
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
