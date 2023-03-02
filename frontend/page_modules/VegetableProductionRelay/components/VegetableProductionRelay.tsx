import { geoMercator, geoPath } from "d3-geo";
import { FeatureCollection, MultiPolygon } from "geojson";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { controlPanelSize, prefectures, svgSize } from "../constants";
import {
  AmountPerPrefectureOfMonth,
  VegetableJson,
} from "../types/AmountPerPrefectureOfMonth";
import { JapanGeoJsonProperties } from "../types/JapanGeoJsonProperties";
import { MousePositionType } from "../types/MousePosition";
import { QueryParams } from "../types/QueryParams";
import { createMonthlyData } from "../utils/createMonthlyData";
import { ControlPanel } from "./ControlPanel";
import { HoverLabel } from "./HoverLabel";
import { PrefectureMap } from "./PrefectureMap";

export const VegetableProductionRelay = (props: {
  queryParams: QueryParams;
  prefectureGeoJson: FeatureCollection<MultiPolygon, JapanGeoJsonProperties>;
  vegetableJson: VegetableJson;
}) => {
  const router = useRouter();
  const [queryParams, setQueryParams] = useState<QueryParams>(
    props.queryParams
  );
  const [mouseOverPrefecture, setMouseOverPrefecture] = useState<
    (typeof prefectures)[number] | null
  >(null);
  const [mousePosition, setMousePosition] = useState<MousePositionType | null>(
    null
  );
  const [prefectureAmount, setPrefectureAmount] = useState<number | null>(null);

  useEffect(() => {
    router.push(
      {
        pathname: "/vegetable-production-relay",
        query: queryParams,
      },
      // router.push でページ先頭までスクロールするのを防ぐ。
      // https://nextjs.org/docs/api-reference/next/router#routerpush
      undefined,
      { scroll: false }
    );
  }, [queryParams]);

  const monthlyData: AmountPerPrefectureOfMonth[] = createMonthlyData(
    props.vegetableJson
  );
  let monthData = monthlyData[queryParams.yearMonthIndex];

  const myGeoPath = geoPath(
    geoMercator().fitSize([svgSize, svgSize], props.prefectureGeoJson)
  );

  return (
    <>
      <div className="hide_on_large_screens">
        <div
          style={{
            display: "flex",
            justifyContent: "center",
            margin: "1rem",
          }}
        >
          <ControlPanel
            queryParams={queryParams}
            setQueryParams={setQueryParams}
            yearMonth={monthlyData}
          />
        </div>
      </div>
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
          <div
            // 操作盤の grid
            className="hide_on_small_screens"
            style={{
              gridArea: "1 / 1 / 2 / 2",
              zIndex: "999",
            }}
          >
            <div
              style={{
                marginTop: "35%",
                marginLeft: "5%",
                padding: "1rem",
                border: "solid rgba(0, 0, 0, 0.3) 1px",
                borderRadius: "10px",
                backgroundColor: "white",
              }}
            >
              <ControlPanel
                queryParams={queryParams}
                setQueryParams={setQueryParams}
                yearMonth={monthlyData}
              />
            </div>
          </div>
          <div style={{ gridArea: "1 / 1 / 3 / 3" }}>
            {mousePosition &&
              prefectureAmount !== null &&
              mouseOverPrefecture !== null && (
                <HoverLabel
                  mousePosition={mousePosition}
                  mouseOverPrefecture={mouseOverPrefecture}
                  prefectureAmount={prefectureAmount}
                />
              )}
            <PrefectureMap
              svgSize={svgSize}
              prefectureGeoJson={props.prefectureGeoJson}
              myGeoPath={myGeoPath}
              monthData={monthData}
              circleSize={queryParams.circleSize}
              mouseOverPrefecture={mouseOverPrefecture}
              onMouseMove={(
                event: React.MouseEvent<SVGElement>,
                prefecture: (typeof prefectures)[number]
              ) => {
                setMouseOverPrefecture(prefecture);
                setPrefectureAmount(monthData[prefecture]);
                setMousePosition({
                  pageX: event.pageX,
                  pageY: event.pageY,
                });
              }}
              onMouseLeave={() => {
                setMouseOverPrefecture(null);
                setPrefectureAmount(null);
                setMousePosition(null);
              }}
            />
          </div>
        </div>
      </div>
    </>
  );
};
