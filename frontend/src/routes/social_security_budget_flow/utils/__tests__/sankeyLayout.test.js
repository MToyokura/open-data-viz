import { describe, expect, test } from "vitest";
import { buildSplitSankey } from "../sankeyLayout.js";

/** @param {string} id @param {string} group @param {number} value */
const item = (id, group, value) => ({ id, label: id, group, value });
/** @param {ReturnType<typeof item>[]} income @param {ReturnType<typeof item>[]} expenditure */
const program = (income, expenditure) => ({ income, expenditure });

describe("buildSplitSankey", () => {
  test("uses one band scale on both sides and leaves a center gap", () => {
    const layout = buildSplitSankey(program(
      [item("I1", "income", 60), item("I2", "transfer", 40)],
      [item("E1", "benefit", 50), item("E2", "other", 50)],
    ));

    expect(layout.scale).toBeGreaterThan(0);
    expect(layout.left.entries[0].thickness / 60).toBe(layout.right.entries[0].thickness / 50);
    expect(layout.left.entries.every((entry) => entry.path.includes("465"))).toBe(true);
    expect(layout.right.entries.every((entry) => entry.path.includes("535"))).toBe(true);
    expect(layout.totalIncome).toBe(100);
    expect(layout.totalExpenditure).toBe(100);
    expect(layout.left.stackYStart).toBe(layout.right.stackYStart);
    expect(layout.totalDifference).toBe(0);
  });

  test("handles surplus, deficit, and empty sides without non-finite geometry", () => {
    for (const data of [
      program([item("I", "income", 100)], [item("E", "benefit", 40)]),
      program([item("I", "income", 40)], [item("E", "benefit", 100)]),
      program([], [item("E", "benefit", 100)]),
      program([], []),
    ]) {
      const layout = buildSplitSankey(data);
      expect(Number.isFinite(layout.scale)).toBe(true);
      expect(layout.left.entries.concat(layout.right.entries).every((entry) => Number.isFinite(entry.thickness) && entry.path !== "")).toBe(true);
    }
  });

  test("marks narrow layouts for the stacked mobile presentation", () => {
    expect(buildSplitSankey(program([], []), 600).shortWidth).toBe(true);
    expect(buildSplitSankey(program([], []), 800).shortWidth).toBe(false);
  });
});
