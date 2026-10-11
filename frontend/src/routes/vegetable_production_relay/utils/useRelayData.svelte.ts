import { onMount } from "svelte";
import type {
  AggregationData,
  GeoCollection,
  PrefectureDetail,
  RelayProduct,
} from "../constants/types";
import {
  fetchRelayAggregation,
  fetchRelayPrefectureDetail,
  fetchRelayProductIndex,
  fetchPrefectureGeojson,
} from "./fetchRelayData";

function toError(reason: unknown, fallback: string): Error {
  return reason instanceof Error ? reason : new Error(fallback);
}

export function useRelayData() {
  // here
  let aggregation = $state.raw<AggregationData | undefined>(undefined);
  let detail = $state.raw<PrefectureDetail | undefined>(undefined);
  let detailError = $state<Error | undefined>(undefined);
  let isDetailLoading = $state(false);
  const detailCache = new Map<string, PrefectureDetail>();
  let detailRequestId = 0;
  let detailController: AbortController | undefined;
  let products = $state.raw<RelayProduct[]>([]);
  let selectedProduct = $state("");
  let geojson = $state.raw<GeoCollection | undefined>(undefined);
  let isLoading = $state(true);
  let error = $state<Error | undefined>(undefined);
  let mapError = $state<Error | undefined>(undefined);
  let requestId = 0;
  let productController: AbortController | undefined;
  let disposed = false;

  async function selectProduct(key: string) {
    // here
    if (disposed || !products.some((product) => product.key === key)) return;

    productController?.abort();
    detailController?.abort();
    detailRequestId += 1;
    detail = undefined;
    detailError = undefined;
    isDetailLoading = false;
    const controller = new AbortController();
    productController = controller;
    const currentRequest = ++requestId;
    selectedProduct = key;
    // Never show the newly selected label with the previous product's values.
    aggregation = undefined;
    isLoading = true;
    error = undefined;

    try {
      const result = await fetchRelayAggregation(key, controller.signal);
      if (!disposed && currentRequest === requestId) aggregation = result;
    } catch (reason) {
      if (
        !disposed &&
        !controller.signal.aborted &&
        currentRequest === requestId
      ) {
        error = toError(reason, "データを読み込めませんでした");
      }
    } finally {
      if (!disposed && currentRequest === requestId) isLoading = false;
    }
  }

  // Details are fetched only after the user selects a prefecture on the map.
  // Changing the month reuses the same prefecture's 36-month response.
  // here
  async function selectPrefectureDetail(code: string) {
    detailController?.abort();
    const currentRequest = ++detailRequestId;
    detail = undefined;
    detailError = undefined;
    isDetailLoading = false;
    const key = selectedProduct;
    if (disposed || !aggregation || aggregation.commodity !== key) return;
    const cacheKey = `${key}:${code}`;
    const cached = detailCache.get(cacheKey);
    if (cached) {
      detail = cached;
      return;
    }
    const controller = new AbortController();
    detailController = controller;
    isDetailLoading = true;
    try {
      const result = await fetchRelayPrefectureDetail(
        key,
        code,
        controller.signal,
      );
      if (
        !disposed &&
        currentRequest === detailRequestId &&
        selectedProduct === key
      ) {
        detailCache.set(cacheKey, result);
        detail = result;
      }
    } catch (reason) {
      if (
        !disposed &&
        !controller.signal.aborted &&
        currentRequest === detailRequestId
      ) {
        detailError = toError(reason, "産地内訳を読み込めませんでした");
      }
    } finally {
      if (!disposed && currentRequest === detailRequestId) {
        isDetailLoading = false;
      }
    }
  }

  onMount(() => {
    const initialController = new AbortController();

    async function initialize() {
      const [productIndexResult, geojsonResult] = await Promise.allSettled([
        fetchRelayProductIndex(initialController.signal),
        fetchPrefectureGeojson(initialController.signal),
      ]);
      if (disposed) return;

      if (geojsonResult.status === "fulfilled") {
        geojson = geojsonResult.value;
      } else {
        mapError = toError(
          geojsonResult.reason,
          "地図データを読み込めませんでした",
        );
      }

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
            products.find((product) => product.key === "veg-01") ?? products[0];
          if (initialProduct) {
            await selectProduct(initialProduct.key);
            return;
          }
          error = new Error("表示できる品目がありません");
        }
      } else {
        error = toError(
          productIndexResult.reason,
          "品目一覧を読み込めませんでした",
        );
      }
      isLoading = false;
    }

    void initialize();
    return () => {
      disposed = true;
      initialController.abort();
      productController?.abort();
      detailController?.abort();
      detailRequestId += 1;
      requestId += 1;
    };
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
    selectPrefectureDetail,
    get detail() {
      return detail;
    },
    get detailError() {
      return detailError;
    },
    get isDetailLoading() {
      return isDetailLoading;
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
