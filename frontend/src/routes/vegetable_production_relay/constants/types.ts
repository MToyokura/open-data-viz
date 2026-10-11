export type Contribution = {
  consumer_region_code: string;
  consumer_region_name: string;
  source_position: number;
  source_cat03_code: string;
  value_kind: "reported" | "zero_symbol_unresolved" | "no_fact";
  raw_value: string;
};

export type PrefectureMonth = {
  month: string;
  consignor_prefecture_code: string;
  consignor_prefecture_name: string;
  reported_value_sum: string;
  source_unit: string | null;
  status: "all_reported" | "partial";
  reported_cells: number;
  zero_symbol_unresolved_cells: number;
  no_fact_cells: number;
  contributions: Contribution[];
};

export type YearData = {
  year: number;
  table_id: string;
  source_unit: string | null;
  months: PrefectureMonth[];
};

export type AggregationData = {
  publishable: boolean;
  years: YearData[];
};

export type GeoCollection = {
  features: Array<{ properties?: { N03_001?: string }; geometry: unknown }>;
};
