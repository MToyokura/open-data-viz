import dynamic from "next/dynamic";
import { useState } from "react";
import { staticAssetsBaseUrl } from "../../../pages/_app";
import { fetcher } from "../../fetcher";
import styles from "../css/CommodityPriceChart.module.css";
import { CommodityPriceResponseJson } from "../types/CommodityPriceData";
import {
  createMonthCodeNamePairs,
  createNullFilledTimeSeries,
  fillNullFilledTimeSeriesWithResponseData,
} from "../utils/dataShaping";
import { Selectors } from "./Selectors";

// https://github.com/apexcharts/vue-apexcharts/issues/307#issuecomment-863501543
const MainChart = dynamic(() => import("react-apexcharts"), { ssr: false });
const MiniChart = dynamic(() => import("react-apexcharts"), { ssr: false });

// HACK: 同じ Chart コンポーネントの場合、グラフ上の legend で非表示にした後に
// データの state をリセットするとエラーが起きるので別チャートを置く。
const PlaceholderChart = dynamic(() => import("react-apexcharts"), {
  ssr: false,
});

export const CommodityPriceChart = () => {
  const [data, setData] = useState<CommodityPriceResponseJson[]>([]);
  const [xaxisCategories, setXaxisCategories] = useState<string[]>([]);
  const [dataNotFound, setDataNotFound] = useState<boolean>(false);

  function fetchData(fileName: string) {
    async function innerAsyncFunction() {
      try {
        const response: CommodityPriceResponseJson = await fetcher(
          `${staticAssetsBaseUrl}/commodity_prices/commodity_price_json_files/${fileName}.json`
        );

        // 最初のデータの場合
        if (!data || data.length === 0) {
          setData([response]);
          setXaxisCategories(
            response.time_series.map((timeSeries) => timeSeries.time_name)
          );
          return;
        }

        // 1つ以上のデータが存在する場合
        const allResponseData = [...data, response];
        const monthCodeNamePairs = createMonthCodeNamePairs(allResponseData);
        const nullFilledTimeSeries =
          createNullFilledTimeSeries(monthCodeNamePairs);
        // もともとあるデータについても増えた年月の分 null 埋めを行う必要がある
        const allCompleteResponseData = allResponseData.map((responseJson) => {
          // 時期の長さと、時期の最初と最後の値が同じであれば null 埋めの作業は不要
          if (
            responseJson.time_series.length === monthCodeNamePairs.size &&
            responseJson.time_series[0].time_code ===
              nullFilledTimeSeries[0].time_code &&
            responseJson.time_series[responseJson.time_series.length - 1]
              .time_code ===
              nullFilledTimeSeries[nullFilledTimeSeries.length - 1].time_code
          ) {
            return responseJson;
          }
          return fillNullFilledTimeSeriesWithResponseData(
            nullFilledTimeSeries,
            responseJson
          );
        });
        setData([...allCompleteResponseData]);
        setXaxisCategories(Array.from(monthCodeNamePairs.values()));
      } catch (err) {
        if (err instanceof Error && err.message === "404 Not Found") {
          setDataNotFound(true);
        }
      }
    }
    innerAsyncFunction();
  }

  function resetData() {
    setData([]);
    setXaxisCategories([]);
  }

  return (
    <div
      style={{
        margin: "5rem 0rem 0rem",
      }}
    >
      <Selectors
        onClickFetch={fetchData}
        onClickReset={resetData}
        dataNotFound={dataNotFound}
        onCloseAlert={() => {
          setDataNotFound(false);
        }}
      />
      <div className={styles.chart_wrapper}>
        {data.length > 0 ? (
          <div>
            <MainChart
              options={{
                chart: {
                  id: "mainChart",
                  zoom: {
                    enabled: false,
                  },
                  animations: {
                    enabled: false,
                  },
                },
                xaxis: {
                  categories: xaxisCategories,
                },
              }}
              type="line"
              width="100%"
              height="500px"
              series={data.map((responseJson) => {
                return {
                  name: `${responseJson.cat02_name} ${responseJson.area_name}`,
                  data: responseJson.time_series.map((timeSeries) => {
                    if (timeSeries.value) {
                      return Number(timeSeries.value);
                    }
                    return null;
                  }),
                };
              })}
            />
            <MiniChart
              options={{
                chart: {
                  brush: {
                    target: "mainChart",
                    enabled: true,
                    autoScaleYaxis: true,
                    // multi-series の場合は機能しないらしい
                    // https://github.com/apexcharts/apexcharts.js/issues/1294
                  },
                  selection: {
                    enabled: true,
                    xaxis: {
                      min: xaxisCategories.length / 3,
                      max: xaxisCategories.length,
                    },
                  },
                  animations: {
                    enabled: false,
                  },
                },
                legend: {
                  show: false,
                },
                xaxis: {
                  categories: xaxisCategories,
                },
              }}
              type="line"
              width="100%"
              height="200"
              series={data.map((responseJson) => {
                return {
                  name: `${responseJson.cat02_name} ${responseJson.area_name}`,
                  data: responseJson.time_series.map((timeSeries) => {
                    if (timeSeries.value) {
                      return Number(timeSeries.value);
                    }
                    return null;
                  }),
                };
              })}
            />
          </div>
        ) : (
          <PlaceholderChart
            options={{
              xaxis: {
                categories: [],
              },
            }}
            type="line"
            width="100%"
            height="500"
            series={[]}
          />
        )}
      </div>
    </div>
  );
};
