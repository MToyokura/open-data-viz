import { EXPENDITURE_GROUPS, INCOME_GROUPS, groupLabel, summarizeItems, totalValue } from "./amountComparison.js";

const ROW_HEIGHT = 38;
const GROUP_GAP = 16;
const TOP = 48;
const GROUP_HEADER_HEIGHT = 24;
const ITEM_VERTICAL_INSET = 5;
const CENTER_X = 500;
const CENTER_GAP = 40;
const LEFT_NODE_X = 190;
const RIGHT_NODE_X = 810;

/** @typedef {{ id: string, label: string, group: string, value: number }} Item */
/** @typedef {{ id: string, label: string, items: Item[] }} ItemGroup */

/** @param {Item[]} items @param {string[]} order @returns {ItemGroup[]} */
function groupedItems(items, order) {
  return order.flatMap((group) => {
    const members = items.filter((item) => item.group === group);
    return members.length ? [{ id: group, label: groupLabel(group), items: members }] : [];
  });
}

/** @param {ItemGroup[]} groups @param {number} scale */
function assignRows(groups, scale) {
  let y = TOP;
  const entries = [];
  const headers = [];
  const groupFrames = [];
  for (const group of groups) {
    const frameTop = y;
    headers.push({ label: group.label, y: y + 15 });
    y += GROUP_HEADER_HEIGHT;
    let lastBottom = y;
    for (const item of group.items) {
      const thickness = item.value * scale;
      const slotHeight = Math.max(ROW_HEIGHT, thickness + ITEM_VERTICAL_INSET * 2);
      entries.push({ ...item, groupLabel: group.label, rowY: y + slotHeight / 2, thickness });
      y += slotHeight;
      lastBottom = y;
    }
    groupFrames.push({ label: group.label, top: frameTop, bottom: lastBottom + 6 });
    y += GROUP_GAP;
  }
  return { entries, headers, groupFrames, height: y + TOP };
}

/** @param {number} x1 @param {number} x2 @param {number} y1 @param {number} y2 @param {number} thickness */
function ribbonPath(x1, x2, y1, y2, thickness) {
  const direction = x2 > x1 ? 1 : -1;
  const start = x1 + direction * (CENTER_GAP / 2);
  const end = x2 - direction * 8;
  const curve = Math.abs(end - start) * 0.48;
  const c1 = start + direction * curve;
  const c2 = end - direction * curve;
  const half = thickness / 2;
  return `M ${start} ${y1 - half} C ${c1} ${y1 - half}, ${c2} ${y2 - half}, ${end} ${y2 - half} L ${end} ${y2 + half} C ${c2} ${y2 + half}, ${c1} ${y1 + half}, ${start} ${y1 + half} Z`;
}

/** @param {Item[]} items @param {string[]} order @param {"left"|"right"} direction @param {number} scale */
function sideLayout(items, order, direction, scale) {
  const { entries, headers, groupFrames } = assignRows(groupedItems(items, order), scale);
  const total = totalValue(items);
  let stackY = TOP + GROUP_HEADER_HEIGHT + ITEM_VERTICAL_INSET;
  const nodeX = direction === "left" ? LEFT_NODE_X : RIGHT_NODE_X;
  const ribbons = entries.map((item) => {
    const thickness = item.thickness;
    const centerY = stackY + thickness / 2;
    stackY += thickness;
    return {
      ...item,
      thickness,
      centerY,
      path: ribbonPath(direction === "left" ? CENTER_X : CENTER_X, nodeX, centerY, item.rowY, thickness),
    };
  });
  return { entries: ribbons, headers, groupFrames, total, stackYStart: TOP + GROUP_HEADER_HEIGHT + ITEM_VERTICAL_INSET, nodeX, direction };
}

/** Calculate two visually separated half-Sankey diagrams on one common money scale. */
/** @param {{ income: Item[], expenditure: Item[] }} program @param {number} [width] */
export function buildSplitSankey(program, width = 1000) {
  const income = summarizeItems(program.income);
  const expenditure = summarizeItems(program.expenditure);
  const maxTotal = Math.max(totalValue(income), totalValue(expenditure));
  const scale = maxTotal > 0 ? 300 / maxTotal : 0;
  const incomeRows = assignRows(groupedItems(income, INCOME_GROUPS), scale);
  const expenditureRows = assignRows(groupedItems(expenditure, EXPENDITURE_GROUPS), scale);
  const height = Math.max(incomeRows.height, expenditureRows.height, 360);
  const left = sideLayout(income, INCOME_GROUPS, "left", scale);
  const right = sideLayout(expenditure, EXPENDITURE_GROUPS, "right", scale);
  const shortWidth = width < 700;

  return {
    width,
    height: shortWidth ? height * 2 + 72 : height,
    scale,
    left,
    right,
    shortWidth,
    totalIncome: totalValue(income),
    totalExpenditure: totalValue(expenditure),
    totalDifference: totalValue(income) - totalValue(expenditure),
    centerX: CENTER_X,
    centerGap: CENTER_GAP,
  };
}
