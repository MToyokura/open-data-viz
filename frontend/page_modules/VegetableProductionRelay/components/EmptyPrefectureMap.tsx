import { GeoPath, GeoPermissibleObjects } from "d3";
import { FeatureCollection, MultiPolygon } from "geojson";
import styles from "../css/VegetableProductionRelay.module.css";
import { JapanGeoJsonProperties } from "../types/JapanGeoJsonProperties";

export const EmptyPrefectureMap = (props: {
  svgSize: number;
  prefectureGeoJson: FeatureCollection<MultiPolygon, JapanGeoJsonProperties>;
  myGeoPath: GeoPath<any, GeoPermissibleObjects>;
}) => {
  return (
    <svg viewBox={`0 0 ${props.svgSize} ${props.svgSize}`}>
      <path
        fill="none"
        strokeWidth={"2"}
        stroke="grey"
        d={`
          M ${props.svgSize * 0.43},${props.svgSize * 0.98}
          l ${props.svgSize * 0.25}, ${props.svgSize * -0.25}
          l ${props.svgSize * 0.3} 0
        `}
      />
      {
        // 都道府県を描く
        // https://wattenberger.com/blog/react-and-d3
        props.prefectureGeoJson.features.map((feature) => {
          const prefPath = props.myGeoPath(feature);
          if (prefPath) {
            return (
              <g key={feature.properties["ADM1_PCODE"]}>
                <path className={styles.prefecture_path} d={prefPath} />
              </g>
            );
          }
          return null;
        })
      }
    </svg>
  );
};
