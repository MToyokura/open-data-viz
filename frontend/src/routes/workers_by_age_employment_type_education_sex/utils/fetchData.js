import {
  WORKERS_DATA_PATH,
  WORKERS_OPTIONS_PATH,
  WORKERS_SERIES_PATH,
} from "../constants/constants.js";

/** @param {unknown} data */
export function validateOptions(data) {
  if (!data || typeof data !== "object")
    throw new Error("選択肢データの形式が正しくありません");
  const options = /** @type {any} */ (data);
  for (const key of ["age", "employment_type", "education", "sex"]) {
    if (
      !Array.isArray(options.dimensions?.[key]) ||
      options.dimensions[key].some(
        (item) =>
          typeof item.code !== "string" || typeof item.name !== "string",
      )
    )
      throw new Error("選択肢データの形式が正しくありません");
  }
  if (!Array.isArray(options.series))
    throw new Error("系列一覧の形式が正しくありません");
  if (!options.metadata || typeof options.metadata.stats_data_id !== "string")
    throw new Error("メタデータの形式が正しくありません");
  for (const item of options.series) {
    if (
      ![
        item.age_code,
        item.employment_type_code,
        item.education_code,
        item.sex_code,
        item.path,
      ].every((v) => typeof v === "string")
    ) {
      throw new Error("系列一覧の形式が正しくありません");
    }
  }
  return /** @type {import("../constants/types.js").WorkersOptions} */ (
    options
  );
}

export async function fetchOptions() {
  const response = await fetch(WORKERS_OPTIONS_PATH);
  if (!response.ok)
    throw new Error(`選択肢を取得できませんでした (HTTP ${response.status})`);
  return validateOptions(await response.json());
}

/** @param {import("../constants/types.js").SeriesIndexItem} item */
export async function fetchSeries(item) {
  const path = item.path.replace(/^\//, "");
  const url = path.startsWith("transformed_data/")
    ? `${WORKERS_DATA_PATH}/${path}`
    : `${WORKERS_SERIES_PATH}/${path}`;
  const response = await fetch(url);
  if (!response.ok)
    throw new Error(
      `系列データを取得できませんでした (HTTP ${response.status})`,
    );
  const data = await response.json();
  if (!data || !Array.isArray(data.time_value_pairs))
    throw new Error("系列データの形式が正しくありません");
  for (const row of data.time_value_pairs) {
    if (
      typeof row.year !== "string" ||
      !(row.value === null || typeof row.value === "number")
    )
      throw new Error("系列データの形式が正しくありません");
  }
  return /** @type {import("../constants/types.js").WorkersSeries} */ (data);
}
