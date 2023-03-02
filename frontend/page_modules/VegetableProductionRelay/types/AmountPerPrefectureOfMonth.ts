// 参考文献
// https://codyarose.com/blog/object-keys-from-array-in-typescript/
// https://steveholgado.com/typescript-types-from-arrays/

import { months, prefectures, years } from "../constants";

export type AmountPerPrefectureOfMonth = {
  [Prefecture in (typeof prefectures)[number]]: number;
};

export type MonthAmount = {
  [Month in (typeof months)[number]]: AmountPerPrefectureOfMonth;
};

export type VegetableJson = {
  [Year in (typeof years)[number]]: MonthAmount;
};
