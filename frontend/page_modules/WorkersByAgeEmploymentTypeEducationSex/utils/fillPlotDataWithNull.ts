import { allNullData } from "../constants";
import { PlotData } from "../types";

/**
 * すべての年に値が入っている状態にするために、
 * 値のない年は null で埋める。
 */
export function createNullFilledPlotData(responseJson: PlotData): PlotData {
  const nullAnnualData = { ...allNullData, ...responseJson["data"] };
  const nullFilledAnnualData = { ...responseJson, data: nullAnnualData };
  return nullFilledAnnualData;
}
