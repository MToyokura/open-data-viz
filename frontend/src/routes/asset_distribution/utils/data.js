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

const DISPLAY_CATEGORY_COUNT = 21;

/** @type {Map<string, Promise<DistributionItem[]>>} */
const chunkPromises = new Map();

/** @type {WeakMap<AssetDistribution, { chunks: Chunk[], byPath: Map<string, Chunk> }>} */
const combinationIndexes = new WeakMap();

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

  const seenDimensionIds = new Set();
  for (const dimension of data.dimensions) {
    if (
      !dimension ||
      typeof dimension.id !== "string" ||
      seenDimensionIds.has(dimension.id) ||
      typeof dimension.label !== "string" ||
      !["filter", "asset_class"].includes(dimension.role) ||
      !Number.isFinite(dimension.order) ||
      !Array.isArray(dimension.categories)
    )
      throw new Error("分類軸の形式が正しくありません");
    seenDimensionIds.add(dimension.id);
    const categoryCodes = new Set();
    for (const category of dimension.categories) {
      if (
        !category ||
        typeof category.code !== "string" ||
        categoryCodes.has(category.code) ||
        typeof category.label !== "string" ||
        typeof category.kind !== "string" ||
        !Number.isFinite(category.order)
      )
        throw new Error(`分類区分の形式が正しくありません: ${dimension.label}`);
      categoryCodes.add(category.code);
    }
    if (
      dimension.role === "filter" &&
      (!categoryCodes.has(dimension.total_code) ||
        !categoryCodes.has(dimension.default_code))
    )
      throw new Error(
        `総数または初期値が分類区分にありません: ${dimension.label}`,
      );
    if (
      dimension.role === "asset_class" &&
      (dimension.total_code !== null || dimension.default_code !== null)
    )
      throw new Error("資産階級軸の総数・初期値は null である必要があります");
    const setIds = new Set();
    for (const set of dimension.comparison_sets ?? []) {
      if (
        !set ||
        typeof set.id !== "string" ||
        setIds.has(set.id) ||
        typeof set.label !== "string" ||
        !Array.isArray(set.category_codes) ||
        new Set(set.category_codes).size !== set.category_codes.length ||
        set.category_codes.some((code) => !categoryCodes.has(code))
      )
        throw new Error(`比較集合の形式が正しくありません: ${dimension.label}`);
      setIds.add(set.id);
    }
  }
  if (
    data.dimensions.filter((dimension) => dimension.role === "asset_class")
      .length !== 1
  )
    throw new Error("資産階級軸を一つ特定できません");

  const assetDimension = data.dimensions.find(
    (dimension) => dimension.role === "asset_class",
  );
  if (!assetDimension) throw new Error("資産階級軸を一つ特定できません");
  const categoryCodes = new Set(
    assetDimension.categories.map((category) => category.code),
  );
  const displayCodes = assetDimension.display_category_codes;
  const displayCategories = Array.isArray(displayCodes)
    ? displayCodes.map((code) =>
        assetDimension.categories.find((category) => category.code === code),
      )
    : [];
  const displayKindCounts = displayCategories.reduce((counts, category) => {
    if (category)
      counts.set(category.kind, (counts.get(category.kind) ?? 0) + 1);
    return counts;
  }, new Map());
  if (
    !Array.isArray(displayCodes) ||
    displayCodes.length !== DISPLAY_CATEGORY_COUNT ||
    new Set(displayCodes).size !== displayCodes.length ||
    displayCodes.some(
      (code) => typeof code !== "string" || !categoryCodes.has(code),
    ) ||
    displayCodes.includes("01") ||
    displayKindCounts.size !== 2 ||
    displayKindCounts.get("reprint") !== 3 ||
    displayKindCounts.get("regular") !== 18
  )
    throw new Error("表示対象の資産階級一覧が正しくありません");

  const filterDimensions = data.dimensions.filter(
    (dimension) => dimension.role === "filter",
  );
  const dimensionIds = data.combination_index.dimension_ids;
  if (
    !Array.isArray(dimensionIds) ||
    dimensionIds.some((id) => typeof id !== "string") ||
    new Set(dimensionIds).size !== dimensionIds.length ||
    dimensionIds.length !== filterDimensions.length ||
    dimensionIds.some(
      (id) => !filterDimensions.some((dimension) => dimension.id === id),
    ) ||
    filterDimensions.some((dimension) => !dimensionIds.includes(dimension.id))
  )
    throw new Error("組み合わせインデックスの分類軸が正しくありません");
  const indexDimensions = dimensionIds.map(
    (id) =>
      /** @type {Dimension} */ (
        filterDimensions.find((dimension) => dimension.id === id)
      ),
  );
  const expectedCombinationCount = indexDimensions.reduce(
    (count, dimension) => count * dimension.categories.length,
    1,
  );
  if (
    !Number.isSafeInteger(data.combination_index.count) ||
    data.combination_index.count < 1 ||
    data.combination_index.count !== expectedCombinationCount
  )
    throw new Error("組み合わせインデックスの件数が正しくありません");

  /** @type {Map<string, Chunk>} */
  const chunkByPath = new Map();
  const orderedChunks = [];
  for (const chunk of data.chunks) {
    if (
      !chunk ||
      !validRelativePath(chunk.path) ||
      chunkByPath.has(chunk.path) ||
      !Number.isSafeInteger(chunk.start_index) ||
      !Number.isSafeInteger(chunk.count) ||
      chunk.start_index < 0 ||
      chunk.count < 1 ||
      (chunk.bytes !== undefined &&
        (!Number.isInteger(chunk.bytes) || chunk.bytes < 0))
    )
      throw new Error("値ファイル一覧の形式が正しくありません");
    chunkByPath.set(chunk.path, chunk);
    orderedChunks.push(chunk);
  }
  orderedChunks.sort((left, right) => left.start_index - right.start_index);
  let nextIndex = 0;
  for (const chunk of orderedChunks) {
    if (
      chunk.start_index !== nextIndex ||
      chunk.start_index + chunk.count > data.combination_index.count
    )
      throw new Error("値ファイル範囲に重複または欠落があります");
    nextIndex += chunk.count;
  }
  if (nextIndex !== data.combination_index.count)
    throw new Error("値ファイル範囲に欠落があります");
  combinationIndexes.set(data, { chunks: orderedChunks, byPath: chunkByPath });
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
  const indexed = combinationIndexes.get(manifest);
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

/** @param {unknown} path */
function validRelativePath(path) {
  return (
    typeof path === "string" &&
    path.length > 0 &&
    !path.startsWith("/") &&
    !path.includes("\\") &&
    path.split("/").every((part) => part && part !== "." && part !== "..")
  );
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
  const indexed = combinationIndexes.get(manifest);
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

  const request = loadAndValidateChunk(manifest, chunk);
  chunkPromises.set(file, request);
  request.catch(() => {
    if (chunkPromises.get(file) === request) chunkPromises.delete(file);
  });
  return request.then((items) =>
    items.filter((item) => item.combination_id === combinationId),
  );
}

/** @param {number} index @param {Dimension[]} dimensions */
function decodeCombinationIndex(index, dimensions) {
  const codes = Array(dimensions.length);
  for (let cursor = dimensions.length - 1; cursor >= 0; cursor--) {
    const categories = dimensions[cursor].categories;
    codes[cursor] = categories[index % categories.length].code;
    index = Math.floor(index / categories.length);
  }
  return codes;
}

/** @param {AssetDistribution} manifest @param {Chunk} chunk @param {number} index */
function combinationAtIndex(manifest, chunk, index) {
  const resolvedDimensions = /** @type {Dimension[]} */ (
    manifest.combination_index.dimension_ids.map((dimensionId) =>
      manifest.dimensions.find((dimension) => dimension.id === dimensionId),
    )
  );
  const codes = decodeCombinationIndex(index, resolvedDimensions);
  return /** @type {Combination} */ ({
    id: resolvedDimensions
      .map(
        (dimension, dimensionIndex) =>
          `${dimension.id}=${codes[dimensionIndex]}`,
      )
      .join("|"),
    dimensions: Object.fromEntries(
      resolvedDimensions.map((dimension, dimensionIndex) => [
        dimension.id,
        codes[dimensionIndex],
      ]),
    ),
    status: "published",
    reason: null,
    items_file: chunk.path,
  });
}

/** @param {AssetDistribution} manifest @param {Chunk} chunk */
async function loadAndValidateChunk(manifest, chunk) {
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
  if (
    chunk.bytes !== undefined &&
    new TextEncoder().encode(text).byteLength !== chunk.bytes
  )
    throw new Error(
      `値ファイルのサイズがmanifestと一致しません: ${chunk.path}`,
    );
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
  const items = /** @type {DistributionItem[]} */ (payload.items);

  const assetCodes = new Set(
    getDisplayAssetCategories(manifest).map((category) => category.code),
  );
  const expectedCombinations = new Set();
  const combinations = [];
  for (
    let index = chunk.start_index;
    index < chunk.start_index + chunk.count;
    index++
  ) {
    const combination = combinationAtIndex(manifest, chunk, index);
    expectedCombinations.add(combination.id);
    combinations.push(combination);
  }
  const itemCounts = new Map();
  const itemKeys = new Set();
  const combinationById = new Map(
    combinations.map((combination) => [combination.id, combination]),
  );
  for (const item of items) {
    const combination = combinationById.get(item?.combination_id);
    if (
      !item ||
      !expectedCombinations.has(item.combination_id) ||
      !combination ||
      combination.items_file !== chunk.path ||
      typeof item.asset_class_code !== "string" ||
      !assetCodes.has(item.asset_class_code) ||
      !["observed", "suppressed", "missing"].includes(item.status) ||
      !(
        item.count === null ||
        (typeof item.count === "number" &&
          Number.isFinite(item.count) &&
          item.count >= 0)
      ) ||
      (item.status === "observed") !== (item.count !== null) ||
      typeof item.raw_value !== "string" ||
      !Array.isArray(item.annotations) ||
      item.annotations.some((note) => typeof note !== "string")
    )
      throw new Error(
        `値ファイル内の階級別データが正しくありません: ${chunk.path}`,
      );
    const key = `${item.combination_id}\u0000${item.asset_class_code}`;
    if (itemKeys.has(key))
      throw new Error(
        `値ファイル内に階級別データの重複があります: ${chunk.path}`,
      );
    itemKeys.add(key);
    itemCounts.set(
      item.combination_id,
      (itemCounts.get(item.combination_id) ?? 0) + 1,
    );
  }
  for (const id of expectedCombinations) {
    const expected = assetCodes.size;
    if ((itemCounts.get(id) ?? 0) !== expected)
      throw new Error(`組み合わせ ${id} の階級別データが不足または余分です`);
  }
  return items;
}
