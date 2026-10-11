export type Contribution = {
  consumer_region_code: string;
  consumer_region_name: string;
  source_position: number;
  source_cat03_code: string;
  value_kind:
    | "reported"
    | "below_display_unit"
    | "no_fact"
    | "unknown_or_uncollected";
  raw_value: string;
};

export type PrefectureMonth = {
  month: string;
  consignor_prefecture_code: string;
  consignor_prefecture_name: string;
  reported_value_sum: string;
  source_unit: string;
  status: "valid" | "incomplete";
  reported_cells: number;
  below_display_unit_cells: number;
  no_fact_cells: number;
  unknown_or_uncollected_cells: number;
  contributions: Contribution[];
};

export type YearData = {
  year: number;
  table_id: string;
  source_unit: string;
  months: PrefectureMonth[];
};

export type AggregationData = {
  status: "valid" | "incomplete" | "invalid";
  coverage_warning: string;
  years: YearData[];
};

export type GeoCollection = {
  features: Array<{ properties?: { N03_001?: string }; geometry: unknown }>;
};
