import { FeatureCollection, MultiPolygon } from "geojson";
import useSWRImmutable from "swr/immutable";
import { staticAssetsBaseUrl } from "../../../pages/_app";
import { fetcher } from "../../fetcher";
import { JapanGeoJsonProperties } from "../types/JapanGeoJsonProperties";

export function usePrefectureGeoJson() {
  const {
    data: responseGeoJson,
    isLoading,
    error,
  } = useSWRImmutable<FeatureCollection<MultiPolygon, JapanGeoJsonProperties>>(
    `${staticAssetsBaseUrl}/simplified_geojson_island_omitted.geojson`,
    fetcher
  );
  if (responseGeoJson) {
    const clonedResponseGeoJson = structuredClone(responseGeoJson);
    for (const outerArray of clonedResponseGeoJson["features"][31]["geometry"][
      "coordinates"
    ]) {
      for (const innerArray of outerArray) {
        for (const latLongPairs of innerArray) {
          latLongPairs[0] = latLongPairs[0] + 17;
          latLongPairs[1] = latLongPairs[1] + 6.5;
        }
      }
    }
    return {
      responseGeoJson: clonedResponseGeoJson,
      isLoading: isLoading,
      error: error,
    };
  }
  return {
    responseGeoJson: responseGeoJson,
    isLoading: isLoading,
    error: error,
  };
}
