import { prefectures } from "../constants";

export type JapanGeoJsonProperties = {
  ADM0_EN: string;
  ADM0_JA: string;
  ADM0_PCODE: string;
  ADM1_EN: string;
  ADM1_JA: (typeof prefectures)[number];
  ADM1_PCODE: string;
};
