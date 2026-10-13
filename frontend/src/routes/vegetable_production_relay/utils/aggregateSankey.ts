import type {
  PrefectureDetail,
  PrefectureMonth,
  RelaySankeyData,
} from "../constants/types";

export const CONSUMER_REGIONS = [
  ["1001", "北海道"],
  ["1002", "東北"],
  ["1004", "関東"],
  ["1006", "北陸"],
  ["1007", "東海"],
  ["1008", "近畿"],
  ["1009", "中国"],
  ["1010", "四国"],
  ["1011", "九州"],
] as const;
export const PREFECTURE_CODES = Array.from({ length: 47 }, (_, i) =>
  String(1002 + i),
);

function prefectureColor(code: string) {
  const number = Number(code) - 1001;
  const limits = [1, 7, 14, 23, 30, 35, 39, 47];
  const colors = [
    "#2563a0",
    "#647a26",
    "#9c5427",
    "#8a528e",
    "#187c76",
    "#ab4767",
    "#766128",
    "#526a96",
  ];
  return colors[limits.findIndex((limit) => number <= limit)] ?? colors[7];
}

export function aggregateSankey(
  details: PrefectureDetail[],
  month: string,
  prefectures: PrefectureMonth[],
): RelaySankeyData {
  const result: RelaySankeyData = {
    nodes: prefectures
      .slice()
      .sort((a, b) =>
        a.consignor_prefecture_code.localeCompare(b.consignor_prefecture_code),
      )
      .map((row, order) => ({
        id: `p:${row.consignor_prefecture_code}`,
        name: row.consignor_prefecture_name,
        side: "source",
        color: prefectureColor(row.consignor_prefecture_code),
        order,
      })),
    links: [],
    total: 0,
    missing: { below_display_unit: 0, no_fact: 0, unknown_or_uncollected: 0 },
  };
  result.nodes.push(
    ...CONSUMER_REGIONS.map(([code, name], order) => ({
      id: `r:${code}`,
      name,
      side: "target" as const,
      color: "#64748b",
      order,
    })),
  );
  for (const detail of details) {
    const cells =
      detail.months.find((row) => row.month === month)?.contributions ?? [];
    for (const [code] of CONSUMER_REGIONS) {
      const cell = cells.find((cell) => cell.consumer_region_code === code);
      if (!cell) {
        result.missing.unknown_or_uncollected++;
        continue;
      }
      if (cell.value_kind !== "reported") {
        result.missing[cell.value_kind]++;
        continue;
      }
      const value = Number(cell.raw_value);
      if (!cell.raw_value.trim() || !Number.isFinite(value) || value < 0) {
        result.missing.unknown_or_uncollected++;
        continue;
      }
      if (value === 0) continue;
      result.links.push({
        source: `p:${detail.consignor_prefecture_code}`,
        target: `r:${code}`,
        value,
      });
      result.total += value;
    }
  }
  return result;
}
