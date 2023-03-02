import { Alert, Button, SelectChangeEvent } from "@mui/material";
import dynamic from "next/dynamic";
import { useState } from "react";
import { staticAssetsBaseUrl } from "../../../pages/_app";
import { fetcher } from "../../fetcher";
import { ageClass, education, employmentType, sex, years } from "../constants";
import { PlotData } from "../types";
import { createNullFilledPlotData } from "../utils/fillPlotDataWithNull";
import { createOptions, createPlotDataSeries } from "../utils/plotUtils";
import { FilterSelector } from "./FilterSelector";
import { ListOfPlotData } from "./ListOfPlotData";

// https://github.com/apexcharts/vue-apexcharts/issues/307#issuecomment-863501543
const Chart = dynamic(() => import("react-apexcharts"), { ssr: false });

export const EmploymentChart = () => {
  const [plotData, setPlotData] = useState<PlotData[]>([]);
  const [dataIndex, setDataIndex] = useState<number>(0); // データを削除するときに使う

  const [ageSelect, setAgeSelect] = useState<keyof typeof ageClass>("総数");
  const [employmentTypeSelect, setEmploymentTypeSelect] =
    useState<keyof typeof employmentType>("雇用者");
  const [educationSelect, setEducationSelect] =
    useState<keyof typeof education>("総数");
  const [sexSelect, setSexSelect] = useState<keyof typeof sex>("総数");

  const plotDataId =
    ageClass[ageSelect] +
    employmentType[employmentTypeSelect] +
    education[educationSelect] +
    sex[sexSelect];

  const [dataNotFound, setDataNotFound] = useState<boolean>(false);

  const [isHorizontal, setIsHorizontal] = useState<boolean>(false);
  const [canResize, setCanResize] = useState<boolean>(false);

  const options = createOptions(years, isHorizontal);
  const series = createPlotDataSeries(plotData);

  function onClickFetchData() {
    const fetchNewPlotData = async () => {
      try {
        // 代入時は PlotData 型ではない。
        // もっと良い書き方がありそう。
        let newPlotData: PlotData = await fetcher(
          `${staticAssetsBaseUrl}/workers_by_age_employment_type_education_sex_files/${plotDataId}.json`
        );
        newPlotData["index"] = dataIndex;
        setDataIndex(dataIndex + 1);
        setDataNotFound(false);
        setPlotData([...plotData, createNullFilledPlotData(newPlotData)]);
      } catch (err) {
        if (err instanceof Error) {
          setDataNotFound(true);
        }
      }
    };
    fetchNewPlotData();
  }

  function onClickDeletePlotData(shownRowdata: PlotData) {
    setPlotData([]);
    setTimeout(() => {
      // HACK: 時間差がないとノーモーションで色が更新されるので時間をおいている
      setPlotData(
        plotData.filter((individualStatePlotData) => {
          return individualStatePlotData.index !== shownRowdata.index;
        })
      );
    }, 1);
  }

  return (
    <div
      style={{
        margin: "5rem 0rem",
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          gap: "1rem",
          flexWrap: "wrap",
        }}
      >
        <FilterSelector
          label="年齢階級"
          currentValue={ageSelect}
          selectOptionValues={Object.keys(ageClass)}
          onChange={(event: SelectChangeEvent) => {
            setAgeSelect(event.target.value as keyof typeof ageClass);
          }}
        />
        <FilterSelector
          label="就業状態"
          currentValue={employmentTypeSelect}
          selectOptionValues={Object.keys(employmentType)}
          onChange={(event: SelectChangeEvent) => {
            setEmploymentTypeSelect(
              event.target.value as keyof typeof employmentType
            );
          }}
        />
        <FilterSelector
          label="教育"
          currentValue={educationSelect}
          selectOptionValues={Object.keys(education)}
          onChange={(event: SelectChangeEvent) => {
            setEducationSelect(event.target.value as keyof typeof education);
          }}
        />
        <FilterSelector
          label="性別"
          currentValue={sexSelect}
          selectOptionValues={Object.keys(sex)}
          onChange={(event: SelectChangeEvent) => {
            setSexSelect(event.target.value as keyof typeof sex);
          }}
        />
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          gap: "1rem",
          flexWrap: "wrap",
          margin: "1rem",
        }}
      >
        <Button
          style={{ width: "20em" }}
          variant="contained"
          disableElevation
          onClick={onClickFetchData}
        >
          追加
        </Button>
        <Button
          variant="outlined"
          disableElevation
          onClick={() => {
            setPlotData([]);
          }}
        >
          リセット
        </Button>
      </div>
      <div style={{ display: "flex", justifyContent: "center" }}>
        {dataNotFound && (
          // 例えば 65歳以上、パート・アルバイト、大学・大学院（在学中）、総数 で起こる
          <Alert
            severity="error"
            onClose={() => {
              setDataNotFound(false);
            }}
          >
            該当データが見つかりませんでした。
          </Alert>
        )}
      </div>
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div
          style={{
            width: "1000px",
            height: `${500 + plotData.length * 10}px`,
            padding: canResize ? "1em" : undefined,
            resize: canResize ? "both" : undefined,
            overflow: canResize ? "auto" : undefined,
            border: canResize ? "solid rgba(0, 0, 0, 0.2) 2px" : undefined,
          }}
        >
          <Chart
            options={options}
            series={series}
            type="bar"
            width="100%"
            height="100%"
          />
        </div>
      </div>
      <div
        style={{
          display: "flex",
          justifyContent: "center",
          gap: "1rem",
          margin: "1rem",
        }}
      >
        <Button
          variant="outlined"
          onClick={() => {
            setIsHorizontal(!isHorizontal);
          }}
        >
          向きを変える
        </Button>
        <Button
          variant="outlined"
          onClick={() => {
            setCanResize(!canResize);
          }}
        >
          サイズを変える（PCのみ）
        </Button>
      </div>
      <ListOfPlotData
        plotData={plotData}
        onClickDeletePlotData={onClickDeletePlotData}
      />
    </div>
  );
};
