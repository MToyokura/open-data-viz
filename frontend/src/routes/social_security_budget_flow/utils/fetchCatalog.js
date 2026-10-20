import { SOCIAL_SECURITY_BUDGET_FLOW_CATALOG_URL } from "../constants.js";

/** @typedef {{id:string,label:string,group:string,value:number}} DiagramNode */
/** @typedef {{id:string,name:string,income:DiagramNode[],expenditure:DiagramNode[],note:string}} Program */
/** @typedef {{programs:Program[]}} Catalog */

/** @param {unknown} value @returns {value is Catalog} */
function isCatalog(value) {
  if (!value || typeof value !== "object") return false;
  const programs = /** @type {{programs?: unknown}} */ (value).programs;
  if (!Array.isArray(programs) || programs.length === 0) return false;
  return programs.every((program) => {
    if (!program || typeof program !== "object") return false;
    const candidate = /** @type {Record<string, unknown>} */ (program);
    if (typeof candidate.id !== "string" || typeof candidate.name !== "string" ||
        typeof candidate.note !== "string" || !Array.isArray(candidate.income) || !Array.isArray(candidate.expenditure)) return false;
    return [...candidate.income, ...candidate.expenditure].every((node) => {
      if (!node || typeof node !== "object") return false;
      const item = /** @type {Record<string, unknown>} */ (node);
      return typeof item.id === "string" && typeof item.label === "string" && typeof item.group === "string" &&
        typeof item.value === "number" && Number.isFinite(item.value) && item.value > 0;
    });
  });
}

/** @returns {Promise<Catalog>} */
export async function fetchCatalog() {
  const response = await fetch(SOCIAL_SECURITY_BUDGET_FLOW_CATALOG_URL, {
    cache: "no-store",
    headers: { Accept: "application/json" },
  });
  if (!response.ok) throw new Error(`カタログの取得に失敗しました（HTTP ${response.status}）。`);
  const data = await response.json();
  if (!isCatalog(data)) throw new Error("カタログの形式が正しくありません。");
  return data;
}
