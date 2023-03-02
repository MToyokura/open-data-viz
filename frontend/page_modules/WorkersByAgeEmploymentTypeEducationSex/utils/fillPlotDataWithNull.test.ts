import { PlotData, ResponseJson } from "../types";
import { createNullFilledPlotData } from "./fillPlotDataWithNull";

const responseJson: ResponseJson = {
  "id": "04100500",
  "年齢階級(詳細集計）": "35～44歳",
  "雇用形態": "非正規の職員・従業員",
  "教育": "卒業計",
  "性別": "総数",
  "data": {
    "2009年": "334",
    "2010年": "343",
    "2011年": null,
    "2012年": "362",
    "2013年": "382",
    "2014年": "391",
    "2015年": "385",
    "2016年": "374",
    "2017年": "365",
    "2018年": "365",
    "2019年": "352",
    "2020年": "327",
    "2021年": "315",
  },
};

const initialPlotData: PlotData = { ...responseJson, index: 123 };

test("creates null filled plot data", () => {
  const expected = {
    "index": 123,
    "id": "04100500",
    "年齢階級(詳細集計）": "35～44歳",
    "雇用形態": "非正規の職員・従業員",
    "教育": "卒業計",
    "性別": "総数",
    "data": {
      "2002年": null,
      "2003年": null,
      "2004年": null,
      "2005年": null,
      "2006年": null,
      "2007年": null,
      "2008年": null,
      "2009年": "334",
      "2010年": "343",
      "2011年": null,
      "2012年": "362",
      "2013年": "382",
      "2014年": "391",
      "2015年": "385",
      "2016年": "374",
      "2017年": "365",
      "2018年": "365",
      "2019年": "352",
      "2020年": "327",
      "2021年": "315",
    },
  };
  const actual = createNullFilledPlotData(initialPlotData);
  expect(actual).toEqual(expected);
});
