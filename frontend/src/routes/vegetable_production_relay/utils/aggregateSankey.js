/** @typedef {import("../constants/types.js").PrefectureDetail} PrefectureDetail */
/** @typedef {import("../constants/types.js").PrefectureMonth} PrefectureMonth */
/** @typedef {import("../constants/types.js").RelaySankeyData} RelaySankeyData */

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
];

// 定義順が地域・都道府県の表示順です。沖縄は九州に含めます。
const AGRICULTURAL_REGIONS = [
  { name: "北海道", color: "#2563a0", prefectures: [["1002", "北海道"]] },
  {
    name: "東北",
    color: "#647a26",
    prefectures: [
      ["1003", "青森"],
      ["1004", "岩手"],
      ["1005", "宮城"],
      ["1006", "秋田"],
      ["1007", "山形"],
      ["1008", "福島"],
    ],
  },
  {
    name: "北陸",
    color: "#9c5427",
    prefectures: [
      ["1016", "新潟"],
      ["1017", "富山"],
      ["1018", "石川"],
      ["1019", "福井"],
    ],
  },
  {
    name: "関東・東山",
    color: "#8a528e",
    prefectures: [
      ["1009", "茨城"],
      ["1010", "栃木"],
      ["1011", "群馬"],
      ["1012", "埼玉"],
      ["1013", "千葉"],
      ["1014", "東京"],
      ["1015", "神奈川"],
      ["1020", "山梨"],
      ["1021", "長野"],
    ],
  },
  {
    name: "東海",
    color: "#187c76",
    prefectures: [
      ["1022", "岐阜"],
      ["1023", "静岡"],
      ["1024", "愛知"],
      ["1025", "三重"],
    ],
  },
  {
    name: "近畿",
    color: "#ab4767",
    prefectures: [
      ["1026", "滋賀"],
      ["1027", "京都"],
      ["1028", "大阪"],
      ["1029", "兵庫"],
      ["1030", "奈良"],
      ["1031", "和歌山"],
    ],
  },
  {
    name: "中国",
    color: "#766128",
    prefectures: [
      ["1032", "鳥取"],
      ["1033", "島根"],
      ["1034", "岡山"],
      ["1035", "広島"],
      ["1036", "山口"],
    ],
  },
  {
    name: "四国",
    color: "#526a96",
    prefectures: [
      ["1037", "徳島"],
      ["1038", "香川"],
      ["1039", "愛媛"],
      ["1040", "高知"],
    ],
  },
  {
    name: "九州",
    color: "#9b5d78",
    prefectures: [
      ["1041", "福岡"],
      ["1042", "佐賀"],
      ["1043", "長崎"],
      ["1044", "熊本"],
      ["1045", "大分"],
      ["1046", "宮崎"],
      ["1047", "鹿児島"],
      ["1048", "沖縄"],
    ],
  },
];

const PREFECTURE_STYLES = new Map(
  AGRICULTURAL_REGIONS.flatMap((region) =>
    region.prefectures.map(([code]) => ({ code, color: region.color })),
  ).map(({ code, color }, order) => [code, { color, order }]),
);

export const PREFECTURE_CODES = [...PREFECTURE_STYLES.keys()];

/** @param {PrefectureDetail[]} details @param {string} month @param {PrefectureMonth[]} prefectures @returns {RelaySankeyData} */
export function aggregateSankey(details, month, prefectures) {
  /** @type {RelaySankeyData} */
  const result = {
    nodes: prefectures
      .slice()
      .sort(
        (a, b) =>
          (PREFECTURE_STYLES.get(a.consignor_prefecture_code)?.order ??
            PREFECTURE_STYLES.size) -
            (PREFECTURE_STYLES.get(b.consignor_prefecture_code)?.order ??
              PREFECTURE_STYLES.size) ||
          a.consignor_prefecture_code.localeCompare(
            b.consignor_prefecture_code,
          ),
      )
      .map((row, order) => ({
        id: `p:${row.consignor_prefecture_code}`,
        name: row.consignor_prefecture_name,
        side: "source",
        color:
          PREFECTURE_STYLES.get(row.consignor_prefecture_code)?.color ??
          "#64748b",
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
      side: /** @type {"target"} */ ("target"),
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
