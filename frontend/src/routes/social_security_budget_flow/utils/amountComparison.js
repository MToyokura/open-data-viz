/** @type {Record<string, string>} */
const GROUP_LABELS = {
  income: "財源",
  transfer: "制度間移転",
  benefit: "給付",
  other: "給付以外の支出",
};

/** @typedef {{ id: string, label: string, group: string, value: number }} AmountItem */

export const INCOME_GROUPS = ["income", "transfer"];
export const EXPENDITURE_GROUPS = ["benefit", "other", "transfer"];

/** Values in the source catalog are in millions of yen. */
/** @param {AmountItem[]} items @returns {AmountItem[]} */
export function summarizeItems(items) {
  return [...items].sort((a, b) => a.group.localeCompare(b.group, "en") || b.value - a.value || a.id.localeCompare(b.id));
}

/** @param {AmountItem[]} items */
export function totalValue(items) {
  return items.reduce((total, item) => total + item.value, 0);
}

/** @param {number} value @param {number} total */
export function amountShare(value, total) {
  if (!Number.isFinite(total) || total <= 0) return "割合は算出できません";
  const percent = (value / total) * 100;
  return `${percent > 0 && percent < 0.05 ? "<0.1" : percent.toFixed(1)}%`;
}

/** @param {number} millionYen */
export function formatApproximateAmount(millionYen) {
  const yen = Math.round(millionYen * 1_000_000);
  if (yen >= 1_000_000_000_000) return `約${(yen / 1_000_000_000_000).toFixed(2).replace(/0+$/, "").replace(/\.$/, "")}兆円`;
  if (yen >= 100_000_000) return `約${Math.round(yen / 100_000_000).toLocaleString("ja-JP")}億円`;
  if (yen >= 10_000) return `約${Math.round(yen / 10_000).toLocaleString("ja-JP")}万円`;
  return `約${yen.toLocaleString("ja-JP")}円`;
}

/** @param {number} millionYen */
export function formatExactAmount(millionYen) {
  return `${millionYen.toLocaleString("ja-JP")}百万円`;
}

/** Format an exact source value as Japanese yen units without rounding. @param {number} millionYen */
export function formatExactYen(millionYen) {
  let yen = BigInt(Math.round(millionYen)) * 1_000_000n;
  let result = "";
  /** @type {[bigint, string][]} */
  const units = [[1_000_000_000_000n, "兆"], [100_000_000n, "億"], [10_000n, "万"]];
  for (const [unit, label] of units) {
    const amount = yen / unit;
    if (amount > 0n) result += `${amount.toLocaleString("ja-JP")}${label}`;
    yen %= unit;
  }
  if (yen > 0n || !result) result += yen.toLocaleString("ja-JP");
  return `${result}円`;
}

/** @param {string} group */
export function groupLabel(group) {
  return GROUP_LABELS[group] ?? group;
}
