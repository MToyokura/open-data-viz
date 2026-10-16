/** @typedef {import("../constants/types.js").GeoCollection} GeoCollection */
/** @typedef {import("../constants/types.js").PrefectureMonth} PrefectureMonth */

/** @typedef {[latitude: number, longitude: number]} LatLng */

/** 北海道の名称は維持し、それ以外の都道府県名から末尾の行政区分を取り除きます。 */
/** @param {string} name */
export function prefectureKey(name) {
  return name === "北海道" ? name : name.replace(/[都府県]$/, "");
}

/** 数値と単位を、単位が不明な場合の表示も含めて整形します。 */
/** @param {string} value @param {string | null} unit */
function formatQuantity(value, unit) {
  return `${value} ${unit ?? "単位未確認"}`;
}

/** 月次観測値の状態に応じた地図表示用ラベルを作成します。 */
/** @param {PrefectureMonth | undefined} observation @param {string | null} sourceUnit */
export function observationLabel(observation, sourceUnit) {
  if (!observation) return "値なし";
  if (observation.reported_cells > 0) {
    const quantity = formatQuantity(observation.reported_value_sum, sourceUnit);
    return observation.status === "incomplete"
      ? `${quantity}（一部未集計）`
      : quantity;
  }
  if (observation.below_display_unit_cells > 0) return "表示単位未満";
  if (observation.unknown_or_uncollected_cells > 0) return "不詳・調査なし";
  if (observation.no_fact_cells > 0) return "該当なし";
  return "値なし";
}

/** @param {unknown} ring @returns {{x: number, y: number, area: number} | null} */
function ringCentroid(ring) {
  if (!Array.isArray(ring) || ring.length < 4) return null;
  let twiceArea = 0;
  let x = 0;
  let y = 0;
  for (let index = 0; index < ring.length - 1; index += 1) {
    const first = ring[index];
    const second = ring[index + 1];
    if (!Array.isArray(first) || !Array.isArray(second)) continue;
    const cross =
      Number(first[0]) * Number(second[1]) -
      Number(second[0]) * Number(first[1]);
    twiceArea += cross;
    x += (Number(first[0]) + Number(second[0])) * cross;
    y += (Number(first[1]) + Number(second[1])) * cross;
  }
  if (!Number.isFinite(twiceArea) || Math.abs(twiceArea) < 1e-12) return null;
  return {
    x: x / (3 * twiceArea),
    y: y / (3 * twiceArea),
    area: Math.abs(twiceArea / 2),
  };
}

/** @param {unknown} polygon @returns {{x: number, y: number, area: number} | null} */
function polygonCentroid(polygon) {
  if (!Array.isArray(polygon)) return null;
  let weight = 0;
  let x = 0;
  let y = 0;
  polygon.forEach((ring, index) => {
    const centroid = ringCentroid(ring);
    if (!centroid) return;
    const area = index === 0 ? centroid.area : -centroid.area;
    weight += area;
    x += centroid.x * area;
    y += centroid.y * area;
  });
  return weight > 0 ? { x: x / weight, y: y / weight, area: weight } : null;
}

/** 地物を都道府県ごとにまとめ、面積で重み付けした重心を返します。 */
/** @param {GeoCollection["features"]} features @returns {Map<string, LatLng>} */
export function calculateCentroids(features) {
  const totals = new Map();

  for (const feature of features) {
    const name = prefectureKey(
      String(feature.properties?.N03_001 ?? "").trim(),
    );
    if (!name) continue;

    /** @type {{type: string, coordinates: unknown}} */
    const geometry = /** @type {any} */ (feature.geometry);
    const polygons =
      geometry.type === "Polygon"
        ? [geometry.coordinates]
        : geometry.type === "MultiPolygon" &&
            Array.isArray(geometry.coordinates)
          ? geometry.coordinates
          : [];
    const total = totals.get(name) ?? { area: 0, x: 0, y: 0 };

    for (const polygon of polygons) {
      const centroid = polygonCentroid(polygon);
      if (!centroid) continue;
      total.area += centroid.area;
      total.x += centroid.x * centroid.area;
      total.y += centroid.y * centroid.area;
    }

    totals.set(name, total);
  }

  /** @type {Map<string, LatLng>} */
  const locations = new Map();
  for (const [name, { area, x, y }] of totals) {
    if (area <= 0) continue;
    locations.set(name, [y / area, x / area]);
  }
  return locations;
}

/** 入れ子になった座標列の経度・緯度に指定したオフセットを加えます。 */
/** @param {unknown} coordinates @param {number} longitudeOffset @param {number} latitudeOffset @returns {unknown} */
export function translateCoordinates(
  coordinates,
  longitudeOffset,
  latitudeOffset,
) {
  if (!Array.isArray(coordinates)) return coordinates;
  if (
    typeof coordinates[0] === "number" &&
    typeof coordinates[1] === "number"
  ) {
    return [
      Number(coordinates[0]) + longitudeOffset,
      Number(coordinates[1]) + latitudeOffset,
      ...coordinates.slice(2),
    ];
  }
  return coordinates.map((coordinate) =>
    translateCoordinates(coordinate, longitudeOffset, latitudeOffset),
  );
}
