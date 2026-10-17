import { VITE_STORAGE_BASE_URL } from "#lib/paths.js";

export const ASSET_DISTRIBUTION_PATH = `${VITE_STORAGE_BASE_URL}/asset_distribution/manifest.json`;

/** @typedef {{ code: string, label: string, kind: string, order: number }} Category */
/** @typedef {{ id: string, label: string, category_codes: string[] }} ComparisonSet */
/** @typedef {{ id: string, label: string, role: "filter"|"asset_class", order: number, total_code: string|null, default_code: string|null, categories: Category[], comparison_sets?: ComparisonSet[] }} Dimension */
/** @typedef {{ id: string, dimensions: Record<string, string>, status: "published"|"not_published", reason: string|null, items_file: string|null }} Combination */
/** @typedef {{ combination_id: string, asset_class_code: string, count: number|null, status: "observed"|"suppressed"|"missing", raw_value: string, annotations: string[] }} DistributionItem */
/** @typedef {{ path: string, combination_ids: string[], bytes?: number }} Chunk */
/** @typedef {{ schema_version: 1, metadata: Record<string, unknown>, dimensions: Dimension[], combinations: Combination[], chunks: Chunk[] }} AssetDistribution */

/** @type {Map<string, Promise<DistributionItem[]>>} */
const chunkPromises = new Map();

/** @param {unknown} input */
export function validateAssetDistribution(input) {
  if (!input || typeof input !== "object")
    throw new Error("manifestの形式が正しくありません");
  const data = /** @type {AssetDistribution} */ (input);
  if (
    data.schema_version !== 1 ||
    !data.metadata ||
    typeof data.metadata !== "object" ||
    !Array.isArray(data.dimensions) ||
    !Array.isArray(data.combinations) ||
    !Array.isArray(data.chunks)
  )
    throw new Error("manifestの形式またはバージョンが正しくありません");

  const dimensionIds = new Set();
  for (const dimension of data.dimensions) {
    if (
      !dimension ||
      typeof dimension.id !== "string" ||
      dimensionIds.has(dimension.id) ||
      typeof dimension.label !== "string" ||
      !["filter", "asset_class"].includes(dimension.role) ||
      !Number.isFinite(dimension.order) ||
      !Array.isArray(dimension.categories)
    )
      throw new Error("分類軸の形式が正しくありません");
    dimensionIds.add(dimension.id);
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

  /** @type {Map<string, Combination>} */
  const combinationById = new Map();
  for (const combination of data.combinations) {
    if (
      !combination ||
      typeof combination.id !== "string" ||
      combinationById.has(combination.id) ||
      !["published", "not_published"].includes(combination.status) ||
      (combination.status === "not_published" &&
        (typeof combination.reason !== "string" ||
          !combination.reason.trim())) ||
      !combination.dimensions ||
      typeof combination.dimensions !== "object" ||
      (combination.status === "published" &&
        !validRelativePath(combination.items_file)) ||
      (combination.status === "not_published" &&
        combination.items_file !== null)
    )
      throw new Error("組み合わせ一覧の形式が正しくありません");
    combinationById.set(combination.id, combination);
    for (const dimension of data.dimensions.filter(
      (item) => item.role === "filter",
    )) {
      const code = combination.dimensions[dimension.id];
      if (
        typeof code !== "string" ||
        !dimension.categories.some((category) => category.code === code)
      )
        throw new Error(`組み合わせの分類コードが不正です: ${dimension.id}`);
    }
  }

  /** @type {Map<string, Chunk>} */
  const chunkByPath = new Map();
  const chunkCombinationIds = new Set();
  for (const chunk of data.chunks) {
    if (
      !chunk ||
      !validRelativePath(chunk.path) ||
      chunkByPath.has(chunk.path) ||
      !Array.isArray(chunk.combination_ids) ||
      chunk.combination_ids.length === 0 ||
      new Set(chunk.combination_ids).size !== chunk.combination_ids.length ||
      (chunk.bytes !== undefined &&
        (!Number.isInteger(chunk.bytes) || chunk.bytes < 0))
    )
      throw new Error("値ファイル一覧の形式が正しくありません");
    chunkByPath.set(chunk.path, chunk);
    for (const id of chunk.combination_ids) {
      const combination = combinationById.get(id);
      if (
        !combination ||
        combination.status !== "published" ||
        combination.items_file !== chunk.path ||
        chunkCombinationIds.has(id)
      )
        throw new Error("manifestの値ファイル対応が正しくありません");
      chunkCombinationIds.add(id);
    }
  }
  for (const combination of data.combinations) {
    if (
      combination.status === "published" &&
      (combination.items_file === null ||
        !chunkByPath.has(combination.items_file) ||
        !chunkCombinationIds.has(combination.id))
    )
      throw new Error(`組み合わせ ${combination.id} の値ファイルがありません`);
  }
  return data;
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
  const combination = manifest.combinations.find(
    (item) => item.id === combinationId,
  );
  if (
    !combination ||
    combination.status !== "published" ||
    !combination.items_file
  )
    return Promise.reject(
      new Error(`公表系列の値ファイルが見つかりません: ${combinationId}`),
    );
  const file = combination.items_file;
  const cached = chunkPromises.get(file);
  if (cached)
    return cached.then((items) =>
      items.filter((item) => item.combination_id === combinationId),
    );

  const chunk = manifest.chunks.find((item) => item.path === file);
  if (!chunk || !chunk.combination_ids.includes(combinationId))
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
  if (!payload || payload.schema_version !== 1 || !Array.isArray(payload.items))
    throw new Error(
      `値ファイルの形式またはバージョンが正しくありません: ${chunk.path}`,
    );
  const items = /** @type {DistributionItem[]} */ (payload.items);

  const assetDimension = manifest.dimensions.find(
    (dimension) => dimension.role === "asset_class",
  );
  if (!assetDimension) throw new Error("資産階級軸を一つ特定できません");
  const assetCodes = new Set(
    assetDimension.categories
      .filter((category) => category.kind === "regular")
      .map((category) => category.code),
  );
  const expectedCombinations = new Set(chunk.combination_ids);
  const itemCounts = new Map();
  const itemKeys = new Set();
  for (const item of items) {
    const combination = manifest.combinations.find(
      (entry) => entry.id === item?.combination_id,
    );
    if (
      !item ||
      !expectedCombinations.has(item.combination_id) ||
      !combination ||
      combination.status !== "published" ||
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
