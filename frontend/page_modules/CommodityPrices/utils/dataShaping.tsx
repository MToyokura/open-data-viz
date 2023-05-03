import {
  CommodityPriceResponseJson,
  IndividualTimeSeries,
} from "../types/CommodityPriceData";

/**
 * すべてが null の time_series を作成する
 */
export function createNullFilledTimeSeries(
  monthCodeNamePairs: Map<string, string>
): IndividualTimeSeries[] {
  const nullFilledTimeSeries: IndividualTimeSeries[] = [];
  monthCodeNamePairs.forEach((value, key) => {
    nullFilledTimeSeries.push({
      time_code: key,
      time_name: value,
      value: null,
    });
  });
  return nullFilledTimeSeries;
}

/**
 * xaxis で用いる月のコードと名前のペアを作成する
 */
export function createMonthCodeNamePairs(
  responseJson: CommodityPriceResponseJson[]
): Map<string, string> {
  const monthCodeNamePairs = new Map<string, string>();
  // For each response json, extract time_code and time_name pairs that don't yet exist in the monthCodeNamePairs
  responseJson.forEach((response) => {
    response.time_series.forEach((timeSeries) => {
      if (!monthCodeNamePairs.has(timeSeries.time_code)) {
        monthCodeNamePairs.set(timeSeries.time_code, timeSeries.time_name);
      }
    });
  });
  // Sort the monthCodeNamePairs by time_code
  const sortedMonthCodeNamePairs = new Map(
    Array.from(monthCodeNamePairs).sort((a, b) => {
      if (a[0] < b[0]) {
        return -1;
      }
      if (a[0] > b[0]) {
        return 1;
      }
      return 0;
    })
  );
  return sortedMonthCodeNamePairs;
}

/**
 * nullFilledTimeSeries に response のデータを入れる
 */
export function fillNullFilledTimeSeriesWithResponseData(
  nullFilledTimeSeries: IndividualTimeSeries[],
  response: CommodityPriceResponseJson
): CommodityPriceResponseJson {
  // Fill null values with the data from the API response
  const filledTimeSeries = nullFilledTimeSeries.map(
    (individualNullFilledTimeSeries) => {
      const matchedTimeSeries = response.time_series.find(
        (individualResponseTimeSeries) =>
          individualResponseTimeSeries.time_code ===
          individualNullFilledTimeSeries.time_code
      );
      if (matchedTimeSeries) {
        return {
          ...individualNullFilledTimeSeries,
          value: matchedTimeSeries.value,
        };
      }
      return individualNullFilledTimeSeries;
    }
  );
  // 入ってきた object を直接変更したくないのでクローンを作成して返す
  const clonedResponse = structuredClone(response);
  clonedResponse.time_series = filledTimeSeries;
  return clonedResponse;
}
