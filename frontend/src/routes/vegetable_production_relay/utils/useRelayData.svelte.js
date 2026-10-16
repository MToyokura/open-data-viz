import { onMount } from "svelte";
import { SvelteMap } from "svelte/reactivity";
import { PREFECTURE_CODES } from "./aggregateSankey.js";
import {
  fetchRelayAggregation,
  fetchRelayPrefectureDetail,
  fetchRelayProductIndex,
  fetchPrefectureGeojson,
} from "./fetchRelayData.js";

/** @typedef {import("../constants/types.js").AggregationData} AggregationData */
/** @typedef {import("../constants/types.js").GeoCollection} GeoCollection */
/** @typedef {import("../constants/types.js").PrefectureDetail} PrefectureDetail */
/** @typedef {import("../constants/types.js").RelayProduct} RelayProduct */

/** @param {unknown} reason @param {string} fallback @returns {Error} */
function toError(reason, fallback) {
  return reason instanceof Error ? reason : new Error(fallback);
}

export function useRelayData() {
  // here
  let aggregation = /** @type {AggregationData | undefined} */ (
    $state.raw(undefined)
  );
  let detail = /** @type {PrefectureDetail | undefined} */ (
    $state.raw(undefined)
  );
  let detailError = /** @type {Error | undefined} */ ($state(undefined));
  let isDetailLoading = $state(false);
  /** @type {SvelteMap<string, PrefectureDetail>} */
  const detailCache = new SvelteMap();
  let detailRequestId = 0;
  /** @type {AbortController | undefined} */
  let detailController;
  let products = /** @type {RelayProduct[]} */ ($state.raw([]));
  let selectedProduct = $state("");
  let geojson = /** @type {GeoCollection | undefined} */ (
    $state.raw(undefined)
  );
  let isLoading = $state(true);
  let error = /** @type {Error | undefined} */ ($state(undefined));
  let mapError = /** @type {Error | undefined} */ ($state(undefined));
  let requestId = 0;
  /** @type {AbortController | undefined} */
  let productController;
  let disposed = false;

  let sankeyOpen = $state(false);
  let isSankeyLoading = $state(false);
  let sankeyDetails = /** @type {PrefectureDetail[]} */ ($state.raw([]));
  let sankeyFailures = /** @type {string[]} */ ($state.raw([]));
  let sankeyCompleted = $state(0);
  /** @type {SvelteMap<string, PrefectureDetail>} */
  let sankeyCache = new SvelteMap();
  /** @type {AbortController | undefined} */
  let sankeyController;
  // Share pending detail requests, including map requests, to avoid duplicate fetches.
  /** @type {SvelteMap<string, Promise<PrefectureDetail>>} */
  const pendingDetails = new SvelteMap();
  let activeDetails = 0;
  /** @type {Array<() => void>} */
  const detailQueue = [];
  /** @param {string} key @param {string} code @param {AbortSignal} signal @returns {Promise<PrefectureDetail>} */
  async function fetchLimitedDetail(key, code, signal) {
    if (activeDetails >= 10)
      await /** @type {Promise<void>} */ (
        new Promise((resolve) => detailQueue.push(() => resolve(undefined)))
      );
    else activeDetails++;
    try {
      signal.throwIfAborted();
      return await fetchRelayPrefectureDetail(key, code, signal);
    } finally {
      const next = detailQueue.shift();
      if (next) next();
      else activeDetails--;
    }
  }
  /** @param {string} key @param {string} code @param {AbortSignal} signal @returns {Promise<PrefectureDetail>} */
  function getDetail(key, code, signal) {
    const cacheKey = `${key}:${code}`;
    const cached =
      detailCache.get(cacheKey) ??
      (key === selectedProduct ? sankeyCache.get(code) : undefined);
    if (cached) return Promise.resolve(cached);
    const pending = pendingDetails.get(cacheKey);
    if (pending)
      return pending
        .then((/** @type {PrefectureDetail} */ result) =>
          signal.aborted
            ? Promise.reject(new DOMException("Aborted", "AbortError"))
            : result,
        )
        .catch((/** @type {unknown} */ reason) => {
          if (signal.aborted) throw reason;
          return getDetail(key, code, signal);
        });
    const request = fetchLimitedDetail(key, code, signal).finally(() => {
      if (pendingDetails.get(cacheKey) === request)
        pendingDetails.delete(cacheKey);
    });
    pendingDetails.set(cacheKey, request);
    return request;
  }
  async function openSankey() {
    if (disposed || !aggregation) return;
    sankeyOpen = true;
    if (isSankeyLoading) return;
    if (sankeyDetails.length === 47) return;
    const key = selectedProduct;
    const controller = new AbortController();
    sankeyController = controller;
    isSankeyLoading = true;
    sankeyFailures = [];
    /** @type {string[]} */
    const failures = [];
    const current = () =>
      !disposed && !controller.signal.aborted && key === selectedProduct;
    const remaining = PREFECTURE_CODES.filter((code) => {
      const cached = detailCache.get(`${key}:${code}`) ?? sankeyCache.get(code);
      if (cached) sankeyCache.set(code, cached);
      return !cached;
    });
    sankeyCompleted = sankeyCache.size;
    for (let start = 0; start < remaining.length; start += 10) {
      if (!current()) return;
      await Promise.all(
        remaining.slice(start, start + 10).map(async (code) => {
          try {
            const result = await getDetail(key, code, controller.signal);
            if (current()) {
              sankeyCache.set(code, result);
              sankeyCompleted = sankeyCache.size;
            }
          } catch {
            if (current()) failures.push(code);
          }
        }),
      );
    }
    if (!current()) return;
    sankeyFailures = failures;
    sankeyDetails = sankeyCache.size === 47 ? [...sankeyCache.values()] : [];
    isSankeyLoading = false;
  }

  /** @param {string} key */
  async function selectProduct(key) {
    // here
    if (disposed || !products.some((product) => product.key === key)) return;

    sankeyController?.abort();
    sankeyCache = new SvelteMap();
    sankeyOpen = false;
    sankeyDetails = [];
    sankeyFailures = [];
    sankeyCompleted = 0;
    isSankeyLoading = false;
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
  /** @param {string} code */
  async function selectPrefectureDetail(code) {
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
      const result = await getDetail(key, code, controller.signal);
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
      sankeyController?.abort();
      productController?.abort();
      detailController?.abort();
      detailRequestId += 1;
      requestId += 1;
    };
  });

  return {
    openSankey,
    closeSankey() {
      sankeyOpen = false;
    },
    get sankeyOpen() {
      return sankeyOpen;
    },
    get sankeyDetails() {
      return sankeyDetails;
    },
    get sankeyFailures() {
      return sankeyFailures;
    },
    get sankeyCompleted() {
      return sankeyCompleted;
    },
    get isSankeyLoading() {
      return isSankeyLoading;
    },
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
