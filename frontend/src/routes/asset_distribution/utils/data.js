import { VITE_STORAGE_BASE_URL } from "#lib/paths.js";

export const ASSET_DISTRIBUTION_PATH = `${VITE_STORAGE_BASE_URL}/asset_distribution/manifest.json`;

/** @typedef {{ code: string, label: string, kind: string, order: number }} Category */
/** @typedef {{ id: string, label: string, category_codes: string[] }} ComparisonSet */
/** @typedef {{ id: string, label: string, role: "filter"|"asset_class", order: number, total_code: string|null, default_code: string|null, categories: Category[], display_category_codes?: string[], comparison_sets?: ComparisonSet[] }} Dimension */
/** @typedef {{ id: string, dimensions: Record<string, string>, status: "published", reason: null, items_file: string }} Combination */
/** @typedef {{ combination_id: string, asset_class_code: string, count: number|null, status: "observed"|"suppressed"|"missing", raw_value: string, annotations: string[] }} DistributionItem */
/** @typedef {{ path: string, start_index: number, count: number, bytes?: number }} Chunk */
/** @typedef {{ dimension_ids: string[], count: number }} CombinationIndex */
/** @typedef {{ schema_version: 3, metadata: Record<string, unknown>, dimensions: Dimension[], combination_index: CombinationIndex, chunks: Chunk[] }} AssetDistribution */

/** @type {Map<string, Promise<DistributionItem[]>>} */
const chunkPromises = new Map();

/** @type {WeakMap<AssetDistribution, { chunks: Chunk[], byPath: Map<string, Chunk> }>} */
const manifestIndexes = new WeakMap();

/** @param {unknown} input */
export function validateAssetDistribution(input) {
  if (!input || typeof input !== "object")
    throw new Error("manifestの形式が正しくありません");
  const data = /** @type {AssetDistribution} */ (input);
  if (
    data.schema_version !== 3 ||
    !data.metadata ||
    typeof data.metadata !== "object" ||
    !Array.isArray(data.dimensions) ||
    !data.combination_index ||
    typeof data.combination_index !== "object" ||
    !Array.isArray(data.chunks)
  )
    throw new Error("manifestの形式またはバージョンが正しくありません");

  if (
    data.dimensions.some(
      (dimension) =>
        !dimension ||
        typeof dimension.id !== "string" ||
        typeof dimension.label !== "string" ||
        !Array.isArray(dimension.categories),
    )
  )
    throw new Error("manifestに表示または検索に必要な情報がありません");
  const assetDimension = data.dimensions.find(
    (dimension) => dimension.role === "asset_class",
  );
  if (
    !assetDimension ||
    !Array.isArray(assetDimension.display_category_codes) ||
    !Array.isArray(data.combination_index.dimension_ids) ||
    !data.combination_index.dimension_ids.length ||
    !data.combination_index.dimension_ids.every((id) =>
      data.dimensions.some((dimension) => dimension.id === id),
    )
  )
    throw new Error("manifestに表示または検索に必要な情報がありません");

  // The generator validates category identities and chunk ranges. Keep the
  // lookup index as part of normal manifest loading for combination search.
  const chunks = [...data.chunks].sort(
    (left, right) => left.start_index - right.start_index,
  );
  manifestIndexes.set(data, {
    chunks,
    byPath: new Map(data.chunks.map((chunk) => [chunk.path, chunk])),
  });
  return data;
}

/** @param {AssetDistribution} manifest */
export function getDisplayAssetCategories(manifest) {
  const dimension = manifest.dimensions.find(
    (item) => item.role === "asset_class",
  );
  if (!dimension?.display_category_codes)
    throw new Error("表示対象の資産階級一覧がありません");
  const categoriesByCode = new Map(
    dimension.categories.map((category) => [category.code, category]),
  );
  return dimension.display_category_codes.map((code) => {
    const category = categoriesByCode.get(code);
    if (!category) throw new Error(`表示対象の資産階級がありません: ${code}`);
    return category;
  });
}

/** @param {AssetDistribution} manifest @param {Record<string, string>} codes @returns {Combination|undefined} */
export function findCombination(manifest, codes) {
  const dimensions = /** @type {Dimension[]} */ (
    manifest.combination_index.dimension_ids.map((id) =>
      manifest.dimensions.find((dimension) => dimension.id === id),
    )
  );
  let index = 0;
  for (const dimension of dimensions) {
    const categoryIndex = dimension.categories.findIndex(
      (category) => category.code === codes[dimension.id],
    );
    if (categoryIndex < 0) return undefined;
    index = index * dimension.categories.length + categoryIndex;
  }
  const indexed = manifestIndexes.get(manifest);
  const chunks =
    indexed?.chunks ??
    [...manifest.chunks].sort(
      (left, right) => left.start_index - right.start_index,
    );
  const chunk = chunks.find(
    (item) =>
      index >= item.start_index && index < item.start_index + item.count,
  );
  if (!chunk) return undefined;
  const combinationDimensions = Object.fromEntries(
    dimensions.map((dimension) => [dimension.id, codes[dimension.id]]),
  );
  return /** @type {Combination} */ ({
    id: dimensions
      .map((dimension) => `${dimension.id}=${codes[dimension.id]}`)
      .join("|"),
    dimensions: combinationDimensions,
    status: "published",
    reason: null,
    items_file: chunk.path,
  });
}

/** @param {string} path */
function chunkUrl(path) {
  return `${VITE_STORAGE_BASE_URL}/asset_distribution/${path
    .split("/")
    .map(encodeURIComponent)
    .join("/")}`;
}

export async function fetchAssetDistribution() {
  let response;
  try {
    response = await fetch(ASSET_DISTRIBUTION_PATH);
  } catch {
    throw new Error(
      "manifestを取得できませんでした。通信状態を確認してください",
    );
  }
  if (!response.ok)
    throw new Error(`manifestを取得できませんでした (HTTP ${response.status})`);
  let input;
  try {
    input = await response.json();
  } catch {
    throw new Error("manifestのJSONを解析できませんでした");
  }
  return validateAssetDistribution(input);
}

/** @param {AssetDistribution} manifest @param {string} combinationId */
export function fetchCombinationItems(manifest, combinationId) {
  const indexed = manifestIndexes.get(manifest);
  const parts = combinationId.split("|");
  const codes = /** @type {Record<string, string>} */ ({});
  for (const part of parts) {
    const separator = part.indexOf("=");
    if (separator < 1) break;
    codes[part.slice(0, separator)] = part.slice(separator + 1);
  }
  const combination = findCombination(manifest, codes);
  if (!combination || combination.id !== combinationId)
    return Promise.reject(
      new Error(`公表系列の値ファイルが見つかりません: ${combinationId}`),
    );
  const file = combination.items_file;
  const cached = chunkPromises.get(file);
  if (cached)
    return cached.then((items) =>
      items.filter((item) => item.combination_id === combinationId),
    );

  const chunk =
    indexed?.byPath.get(file) ??
    manifest.chunks.find((item) => item.path === file);
  if (!chunk)
    return Promise.reject(
      new Error(`manifestに値ファイルの登録がありません: ${combinationId}`),
    );

  const request = loadChunk(chunk);
  chunkPromises.set(file, request);
  request.catch(() => {
    if (chunkPromises.get(file) === request) chunkPromises.delete(file);
  });
  return request.then((items) =>
    items.filter((item) => item.combination_id === combinationId),
  );
}

/** @param {Chunk} chunk */
async function loadChunk(chunk) {
  let response;
  try {
    response = await fetch(chunkUrl(chunk.path));
  } catch {
    throw new Error(`値ファイルを取得できませんでした: ${chunk.path}`);
  }
  if (!response.ok)
    throw new Error(
      `値ファイルを取得できませんでした (HTTP ${response.status}): ${chunk.path}`,
    );
  let text;
  try {
    text = await response.text();
  } catch {
    throw new Error(`値ファイルのJSONを解析できませんでした: ${chunk.path}`);
  }
  let payload;
  try {
    payload = JSON.parse(text);
  } catch {
    throw new Error(`値ファイルのJSONを解析できませんでした: ${chunk.path}`);
  }
  if (!payload || payload.schema_version !== 3 || !Array.isArray(payload.items))
    throw new Error(
      `値ファイルの形式またはバージョンが正しくありません: ${chunk.path}`,
    );
  return /** @type {DistributionItem[]} */ (payload.items);
}
