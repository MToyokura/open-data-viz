/** @typedef {import("../constants/types.js").PrefectureDetail} PrefectureDetail */

/** @typedef {{
 * detailCache: Map<string, PrefectureDetail>,
 * getSankeyCache: () => Map<string, PrefectureDetail>,
 * getSelectedProduct: () => string,
 * fetchDetail: (product: string, prefectureCode: string, signal: AbortSignal) => Promise<PrefectureDetail>,
 * maxConcurrent?: number,
 * }} DetailRequestManagerOptions
 */

/**
 * Shares in-flight detail requests and limits how many fetches run at once.
 * @param {DetailRequestManagerOptions} options
 */
export function createDetailRequestManager({
  detailCache,
  getSankeyCache,
  getSelectedProduct,
  fetchDetail,
  maxConcurrent = 10,
}) {
  /** @type {Map<string, Promise<PrefectureDetail>>} */
  const pendingRequests = new Map();
  /** @type {Array<() => void>} */
  const waitingRequests = [];
  let activeRequests = 0;

  /** @param {string} product @param {string} prefectureCode @param {AbortSignal} signal */
  async function fetchWithLimit(product, prefectureCode, signal) {
    if (activeRequests >= maxConcurrent) {
      await /** @type {Promise<void>} */ (
        new Promise((resolve) => waitingRequests.push(() => resolve(undefined)))
      );
    } else {
      activeRequests += 1;
    }

    try {
      signal.throwIfAborted();
      return await fetchDetail(product, prefectureCode, signal);
    } finally {
      const nextRequest = waitingRequests.shift();
      if (nextRequest) nextRequest();
      else activeRequests -= 1;
    }
  }

  /**
   * @param {string} product
   * @param {string} prefectureCode
   * @param {AbortSignal} signal
   * @returns {Promise<PrefectureDetail>}
   */
  function getDetail(product, prefectureCode, signal) {
    const cacheKey = `${product}:${prefectureCode}`;
    const cached =
      detailCache.get(cacheKey) ??
      (product === getSelectedProduct()
        ? getSankeyCache().get(prefectureCode)
        : undefined);
    if (cached) return Promise.resolve(cached);

    const pendingRequest = pendingRequests.get(cacheKey);
    if (pendingRequest) {
      return pendingRequest
        .then((result) => {
          if (signal.aborted) {
            throw new DOMException("Aborted", "AbortError");
          }
          return result;
        })
        .catch((reason) => {
          if (signal.aborted) throw reason;
          return getDetail(product, prefectureCode, signal);
        });
    }

    const request = fetchWithLimit(product, prefectureCode, signal).finally(
      () => {
        if (pendingRequests.get(cacheKey) === request) {
          pendingRequests.delete(cacheKey);
        }
      },
    );
    pendingRequests.set(cacheKey, request);
    return request;
  }

  return { getDetail };
}
