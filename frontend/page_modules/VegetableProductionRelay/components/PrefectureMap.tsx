import { animated, useSpring } from "@react-spring/web";
import { GeoPath, GeoPermissibleObjects } from "d3";
import { FeatureCollection, MultiPolygon } from "geojson";
import { useEffect, useRef } from "react";
import { prefectures } from "../constants";
import styles from "../css/VegetableProductionRelay.module.css";
import { AmountPerPrefectureOfMonth } from "../types/AmountPerPrefectureOfMonth";
import { JapanGeoJsonProperties } from "../types/JapanGeoJsonProperties";
import { getPrefectureCentroid } from "../utils/getPrefectureCentroid";

export const PrefectureMap = (props: {
  svgSize: number;
  prefectureGeoJson: FeatureCollection<MultiPolygon, JapanGeoJsonProperties>;
  myGeoPath: GeoPath<any, GeoPermissibleObjects>;
  monthData: AmountPerPrefectureOfMonth;
  circleSize: number;
  mouseOverPrefecture: (typeof prefectures)[number] | null;
  onMouseMove: (
    event: React.MouseEvent<SVGElement>,
    prefecture: (typeof prefectures)[number]
  ) => void;
  onMouseLeave: () => void;
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
          const prefecture: (typeof prefectures)[number] =
            feature.properties["ADM1_JA"];
          const isMouseOver = props.mouseOverPrefecture === prefecture;
          if (prefPath) {
            return (
              <g key={feature.properties["ADM1_PCODE"]}>
                <path
                  className={
                    isMouseOver
                      ? styles.prefecture_path_hover
                      : styles.prefecture_path
                  }
                  d={prefPath}
                  onMouseMove={(event) => {
                    props.onMouseMove(event, prefecture);
                  }}
                  onMouseLeave={props.onMouseLeave}
                />
              </g>
            );
          }
          return null;
        })
      }
      {
        // 円を描く
        // https://d3-graph-gallery.com/graph/hexbinmap_geo_label.html
        props.prefectureGeoJson.features.map((feature) => {
          const prefecture: (typeof prefectures)[number] =
            feature.properties["ADM1_JA"];
          const prefPath = props.myGeoPath(feature);
          const isMouseOver = props.mouseOverPrefecture === prefecture;

          // Stuff for transition effect
          const previousValue = useRef(0);
          useEffect(() => {
            previousValue.current = props.monthData[prefecture];
          }, [props.monthData[prefecture]]);
          const springs = useSpring({
            from: {
              r: calculateRadius(previousValue.current, props.circleSize),
            },
            to: {
              r: calculateRadius(props.monthData[prefecture], props.circleSize),
            },
          });

          if (prefPath) {
            const prefectureCentroid = getPrefectureCentroid(
              props.myGeoPath,
              feature
            );
            return (
              <g key={feature.properties["ADM1_PCODE"]}>
                <animated.circle
                  {...springs}
                  className={
                    isMouseOver
                      ? styles.prefecture_circle_hover
                      : styles.prefecture_circle
                  }
                  cx={prefectureCentroid[0]}
                  cy={prefectureCentroid[1]}
                  onMouseMove={(event) => {
                    props.onMouseMove(event, prefecture);
                  }}
                  onMouseLeave={props.onMouseLeave}
                />
              </g>
            );
          }
          return null;
        })
      }
    </svg>
  );
};

function calculateRadius(amount: number, baseCircleSize: number): number {
  return Math.sqrt(amount) / (15 / baseCircleSize);
}
