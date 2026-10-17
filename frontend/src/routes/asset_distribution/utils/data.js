import { VITE_STORAGE_BASE_URL } from "#lib/paths.js";

export const ASSET_DISTRIBUTION_PATH = `${VITE_STORAGE_BASE_URL}/asset_distribution/asset_distribution.json`;

/** @typedef {{ code: string, label: string, kind: string, order: number }} Category */
/** @typedef {{ id: string, label: string, category_codes: string[] }} ComparisonSet */
/** @typedef {{ id: string, label: string, role: "filter"|"asset_class", order: number, total_code: string|null, default_code: string|null, categories: Category[], comparison_sets?: { id: string, label: string, category_codes: string[] }[] }} Dimension */
/** @typedef {{ id: string, dimensions: Record<string, string>, status: "published"|"not_published", reason: string|null }} Combination */
/** @typedef {{ combination_id: string, asset_class_code: string, count: number|null, status: "observed"|"suppressed"|"missing", raw_value: string, annotations: string[] }} DistributionItem */
/** @typedef {{ metadata: Record<string, unknown>, dimensions: Dimension[], combinations: Combination[], items: DistributionItem[] }} AssetDistribution */

/** @param {unknown} input */
export function validateAssetDistribution(input) {
  if (!input || typeof input !== "object")
    throw new Error("データの形式が正しくありません");
  const data = /** @type {AssetDistribution} */ (input);
  if (
    !data.metadata ||
    !Array.isArray(data.dimensions) ||
    !Array.isArray(data.combinations) ||
    !Array.isArray(data.items)
  ) {
    throw new Error(
      "データの形式が正しくありません（metadata / dimensions / combinations / items が必要です）",
    );
  }
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
    ) {
      throw new Error("分類軸の形式が正しくありません");
    }
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
    for (const set of dimension.comparison_sets ?? []) {
      if (
        !set ||
        typeof set.id !== "string" ||
        typeof set.label !== "string" ||
        !Array.isArray(set.category_codes) ||
        new Set(set.category_codes).size !== set.category_codes.length ||
        set.category_codes.some((code) => !categoryCodes.has(code))
      )
        throw new Error(`比較集合の形式が正しくありません: ${dimension.label}`);
    }
  }
  if (
    data.dimensions.filter((dimension) => dimension.role === "asset_class")
      .length !== 1
  )
    throw new Error("資産階級軸を一つ特定できません");
  const combinationIds = new Set();
  for (const combination of data.combinations) {
    if (
      !combination ||
      typeof combination.id !== "string" ||
      combinationIds.has(combination.id) ||
      !["published", "not_published"].includes(combination.status) ||
      (combination.status === "not_published" &&
        (typeof combination.reason !== "string" ||
          !combination.reason.trim())) ||
      !combination.dimensions ||
      typeof combination.dimensions !== "object"
    )
      throw new Error("組み合わせ一覧の形式が正しくありません");
    combinationIds.add(combination.id);
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
  const assetDimension = /** @type {Dimension} */ (
    data.dimensions.find((dimension) => dimension.role === "asset_class")
  );
  const assetCodes = new Set(
    assetDimension.categories
      .filter((category) => category.kind === "regular")
      .map((category) => category.code),
  );
  const itemKeys = new Set();
  for (const item of data.items) {
    if (
      !item ||
      typeof item.combination_id !== "string" ||
      !combinationIds.has(item.combination_id) ||
      typeof item.asset_class_code !== "string" ||
      !assetCodes.has(item.asset_class_code) ||
      !["observed", "suppressed", "missing"].includes(item.status) ||
      !(
        item.count === null ||
        (typeof item.count === "number" &&
          Number.isFinite(item.count) &&
          item.count >= 0)
      ) ||
      typeof item.raw_value !== "string" ||
      !Array.isArray(item.annotations) ||
      item.annotations.some((note) => typeof note !== "string")
    )
      throw new Error("階級別データの形式が正しくありません");
    if ((item.status === "observed") !== (item.count !== null))
      throw new Error("値の状態と数値が一致しません");
    const key = `${item.combination_id}\u0000${item.asset_class_code}`;
    if (itemKeys.has(key)) throw new Error("階級別データに重複があります");
    itemKeys.add(key);
  }
  for (const combination of data.combinations) {
    const expected = combination.status === "published" ? assetCodes.size : 0;
    const actual = data.items.filter(
      (item) => item.combination_id === combination.id,
    ).length;
    if (actual !== expected)
      throw new Error(
        `組み合わせ ${combination.id} の階級別データが不足または余分です`,
      );
  }
  return /** @type {AssetDistribution} */ (data);
}

export async function fetchAssetDistribution() {
  const response = await fetch(ASSET_DISTRIBUTION_PATH);
  if (!response.ok)
    throw new Error(`データを取得できませんでした (HTTP ${response.status})`);
  return validateAssetDistribution(await response.json());
}
