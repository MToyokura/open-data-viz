// here
export type Contribution = {
  consumer_region_code: string;
  consumer_region_name: string;
  value_kind:
    "reported" | "below_display_unit" | "no_fact" | "unknown_or_uncollected";
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
};

export type YearData = {
  year: number;
  table_id: string;
  source_unit: string;
  months: PrefectureMonth[];
};

export type AggregationData = {
  schema_version: 2;
  status: "valid" | "incomplete";
  commodity: string;
  commodity_label: string;
  mapping_warnings: string[];
  years: YearData[];
};

export type PrefectureDetail = {
  schema_version: 2;
  commodity: string;
  consignor_prefecture_code: string;
  months: Array<{ month: string; contributions: Contribution[] }>;
};

export type RelayProduct = {
  key: string;
  label: string;
};

export type RelayProductIndex = {
  schema_version: 1;
  products: RelayProduct[];
};

export type GeoCollection = {
  features: Array<{ properties?: { N03_001?: string }; geometry: unknown }>;
};

export type RelaySankeyNode = {
  id: string;
  name: string;
  side: "source" | "target";
  color: string;
  order: number;
};
export type RelaySankeyLink = { source: string; target: string; value: number };
export type RelaySankeyData = {
  nodes: RelaySankeyNode[];
  links: RelaySankeyLink[];
  total: number;
  missing: Record<Exclude<Contribution["value_kind"], "reported">, number>;
};
