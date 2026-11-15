import { SOCIAL_SECURITY_BUDGET_FLOW_CATALOG_URL } from "../constants.js";
import { ITEM_METADATA } from "./amountComparison.js";

/** @typedef {{id:string,label:string,group:string,value:number}} DiagramNode */
/** @typedef {{id:string,value:number}} CatalogAmount */
/** @typedef {{id:string,name:string,income:CatalogAmount[],expenditure:CatalogAmount[]}} CatalogProgramData */
/** @typedef {{programs:CatalogProgramData[]}} CatalogData */
/** @typedef {{id:string,name:string,income:DiagramNode[],expenditure:DiagramNode[]}} Program */
/** @typedef {{programs:Program[]}} Catalog */

/** @param {unknown[]} nodes @param {"income"|"expenditure"} side */
function hasValidItems(nodes, side) {
  return nodes.every((node) => {
    if (!node || typeof node !== "object") return false;
    const item = /** @type {Record<string, unknown>} */ (node);
    return typeof item.id === "string" && ITEM_METADATA[item.id]?.side === side &&
      typeof item.value === "number" && Number.isFinite(item.value) && item.value > 0;
  });
}

/** @param {unknown} value @returns {value is CatalogData} */
function isCatalog(value) {
  if (!value || typeof value !== "object") return false;
  const programs = /** @type {{programs?: unknown}} */ (value).programs;
  if (!Array.isArray(programs) || programs.length === 0) return false;
  return programs.every((program) => {
    if (!program || typeof program !== "object") return false;
    const candidate = /** @type {Record<string, unknown>} */ (program);
    if (typeof candidate.id !== "string" || typeof candidate.name !== "string" ||
        !Array.isArray(candidate.income) || !Array.isArray(candidate.expenditure)) return false;
    return hasValidItems(candidate.income, "income") && hasValidItems(candidate.expenditure, "expenditure");
  });
}

/** @param {CatalogAmount[]} items @returns {DiagramNode[]} */
function addItemMetadata(items) {
  return items.map(({ id, value }) => {
    const { label, group } = ITEM_METADATA[id];
    return { id, label, group, value };
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
  return {
    programs: data.programs.map((program) => ({
      ...program,
      income: addItemMetadata(program.income),
      expenditure: addItemMetadata(program.expenditure),
    })),
  };
}
