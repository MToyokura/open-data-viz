import { usePrefectureGeoJson } from "../hooks/usePrefectureGeoJson";
import { useVegetableJson } from "../hooks/useVegetableJson";
import { QueryParams } from "../types/QueryParams";
import { EmptyVegetableProductionRelay } from "./EmptyVegetableProductionRelay";
import { VegetableProductionRelay } from "./VegetableProductionRelay";

/**
 * クエリパラメータを作って、必要なデータを取得して地図本体とかのページを呼び出す。
 */
export const VegetableProductionRelayPrep = (props: {
  queryParams: QueryParams;
}) => {
  const {
    responseGeoJson: prefectureGeoJson,
    isLoading: responseGeoJsonIsLoading,
    error: responseGeoJsonError,
  } = usePrefectureGeoJson();
  const {
    responseVegetableJson: vegetableJson,
    isLoading: vegetableJsonIsLoading,
    error: vegetableJsonError,
  } = useVegetableJson(props.queryParams.vegetable);

  if (props.queryParams && prefectureGeoJson && vegetableJson) {
    return (
      <VegetableProductionRelay
        queryParams={props.queryParams}
        prefectureGeoJson={prefectureGeoJson}
        vegetableJson={vegetableJson}
      />
    );
  }
  if (prefectureGeoJson && !vegetableJson) {
    return (
      <EmptyVegetableProductionRelay prefectureGeoJson={prefectureGeoJson} />
    );
  }
  if (responseGeoJsonError || vegetableJsonError) {
    return (
      <div style={{ display: "flex", justifyContent: "center" }}>
        <p>Error!!</p>
      </div>
    );
  }
  return (
    <div style={{ display: "flex", justifyContent: "center" }}>loading...</div>
  );
};
