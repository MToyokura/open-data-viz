export type CommodityPriceResponseJson = {
  area_code: string;
  area_name: string;
  cat02_code: string;
  cat02_name: string;
  time_series: IndividualTimeSeries[];
};

export type IndividualTimeSeries = {
  time_code: string;
  time_name: string;
  value: string | null;
};
