/** @typedef {{ code: string, name: string, parentCode?: string }} Item */
/** @typedef {Item & { displayParentCode?: string, displayDepth: number }} DisplayItem */

// Relationships shown in the Statistics Bureau's 2025-base aggregation tables
// or national release (Table 4) that are absent from the e-Stat parent codes.
/** @type {Map<string, string>} */
const displayParents = new Map([
  ["0158", "0157"],
  ["0159", "0157"],
  ["0160", "0157"],
  ["0202", "0201"],
  ["0220", "0201"],
  ["0206", "0205"],
  ["0245", "0230"],
  ["0246", "0230"],
  ["0247", "0231"],
  ["0248", "0231"],
]);

/** @param {Item[]} items @returns {DisplayItem[]} */
export function buildItemDisplayHierarchy(items) {
  const itemCodes = new Set(items.map((item) => item.code));
  /** @type {Map<string | undefined, Item[]>} */
  const childrenByParent = new Map();
  for (const item of items) {
    const parentCode = item.parentCode || displayParents.get(item.code);
    const displayParentCode =
      parentCode && itemCodes.has(parentCode) ? parentCode : undefined;
    const siblings = childrenByParent.get(displayParentCode) ?? [];
    siblings.push(item);
    childrenByParent.set(displayParentCode, siblings);
  }

  /** @type {DisplayItem[]} */
  const displayItems = [];
  /** @type {Set<string>} */
  const displayed = new Set();

  /** @param {Item} item @param {number} displayDepth @param {string} [displayParentCode] */
  function append(item, displayDepth, displayParentCode) {
    if (displayed.has(item.code)) return;
    displayed.add(item.code);
    displayItems.push({ ...item, displayParentCode, displayDepth });
    for (const child of childrenByParent.get(item.code) ?? []) {
      append(child, displayDepth + 1, item.code);
    }
  }

  for (const root of childrenByParent.get(undefined) ?? []) append(root, 0);
  for (const item of items) append(item, 0);
  return displayItems;
}
