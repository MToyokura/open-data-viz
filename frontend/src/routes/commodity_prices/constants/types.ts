export type Cat02Lookup = {
  cat02_code: string;
  cat02_name: string;
}[];

export type AreaLookup = {
  area_code: string;
  area_name: string;
}[];

export type CommodityPrices = {
  cat02_code: string;
  cat02_name: string;
  area_code: string;
  area_name: string;
  time_value_pairs: {
    time: string;
    value: string;
  }[];
};
