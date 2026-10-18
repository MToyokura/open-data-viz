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
  const expectedEducation =
    /** @type {[string, string, string, string|null][]} */ ([
      ["00", "総数", "1", null],
      ["01", "うち在学中", "2", null],
      ["02", "小学・中学・高校", "3", null],
      ["03", "短大・高専", "3", "02"],
      ["04", "大学・大学院", "3", "02"],
      ["05", "うち卒業", "2", null],
      ["06", "小学・中学・高校・旧中", "3", "05"],
      ["07", "短大・高専", "3", "05"],
      ["08", "大学・大学院", "3", "05"],
      ["09", "大学", "3", "05"],
      ["10", "大学院", "3", "05"],
    ]);
  const education = options.dimensions.education;
  if (
    education.length !== expectedEducation.length ||
    education.some(
      (
        /** @type {import("../constants/types.js").DimensionOption} */ item,
        /** @type {number} */ index,
      ) => {
        const [code, name, level, parent] = expectedEducation[index];
        return (
          item.code !== code ||
          item.name !== name ||
          item.order !== index ||
          item.api_level !== level ||
          (item.api_parent_code ?? null) !== parent
        );
      },
    )
  )
    throw new Error("教育区分の公式メタデータが想定と異なります");
  const flattened = /** @type {string[]} */ ([]);
  /** @param {import("../constants/types.js").DisplayNode[]} nodes */
  const collect = (nodes) => {
    for (const node of nodes) {
      flattened.push(node.code);
      collect(node.children ?? []);
    }
  };
  collect(options.education_display_hierarchy ?? []);
  if (
    flattened.length !== expectedEducation.length ||
    new Set(flattened).size !== flattened.length ||
    expectedEducation.some(([code]) => !flattened.includes(code))
  )
    throw new Error("教育区分の画面表示階層が不完全です");
  if (
    !Array.isArray(options.education_comparison_sets) ||
    JSON.stringify(
      options.education_comparison_sets.map(
        (
          /** @type {import("../constants/types.js").EducationComparisonSet} */ set,
        ) => [set.id, set.category_codes],
      ),
    ) !==
      JSON.stringify([
        ["education-status", ["01", "05"]],
        ["enrolled-categories", ["02", "03", "04"]],
        ["graduated-categories", ["06", "07", "08", "09", "10"]],
      ])
  )
    throw new Error("教育区分の比較集合が想定と異なります");
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
