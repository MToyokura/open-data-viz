import { GeoPath } from "d3";
import { Feature, MultiPolygon } from "geojson";
import { JapanGeoJsonProperties } from "../types/JapanGeoJsonProperties";

/**
 * 円を見やすい位置に調整する。
 */

export function getPrefectureCentroid(
  geoPath: GeoPath,
  feature: Feature<MultiPolygon, JapanGeoJsonProperties>
) {
  if (feature.properties.ADM1_EN === "Ishikawa") {
    const centroid = geoPath.centroid(feature);
    return [centroid[0] * 0.98, centroid[1] * 1.02];
  }
  if (feature.properties.ADM1_EN === "Tottori") {
    const centroid = geoPath.centroid(feature);
    return [centroid[0], centroid[1] * 0.99];
  }
  if (feature.properties.ADM1_EN === "Nagasaki") {
    const centroid = geoPath.centroid(feature);
    return [centroid[0] * 0.97, centroid[1] * 1.02];
  }
  return geoPath.centroid(feature);
}
