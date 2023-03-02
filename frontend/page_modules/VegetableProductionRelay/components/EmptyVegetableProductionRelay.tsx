import { geoMercator, geoPath } from "d3-geo";
import { FeatureCollection, MultiPolygon } from "geojson";
import { controlPanelSize, svgSize } from "../constants";
import { JapanGeoJsonProperties } from "../types/JapanGeoJsonProperties";

import { EmptyPrefectureMap } from "./EmptyPrefectureMap";

export const EmptyVegetableProductionRelay = (props: {
  prefectureGeoJson: FeatureCollection<MultiPolygon, JapanGeoJsonProperties>;
}) => {
  const myGeoPath = geoPath(
    geoMercator().fitSize([svgSize, svgSize], props.prefectureGeoJson)
  );

  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
      }}
    >
      <div
        // 図の grid
        style={{
          display: "grid",
          gridTemplateColumns: `
              minmax(0px, ${controlPanelSize}px) 
              minmax(0px, ${svgSize - controlPanelSize}px)
            `,
        }}
      >
        <div style={{ gridArea: "1 / 1 / 3 / 3" }}>
          <EmptyPrefectureMap
            svgSize={svgSize}
            prefectureGeoJson={props.prefectureGeoJson}
            myGeoPath={myGeoPath}
          />
        </div>
      </div>
    </div>
  );
};
