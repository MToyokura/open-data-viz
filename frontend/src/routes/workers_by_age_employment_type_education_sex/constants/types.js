/** @typedef {{ code: string, name: string }} DimensionOption */
/** @typedef {{ age: DimensionOption[], employment_type: DimensionOption[], education: DimensionOption[], sex: DimensionOption[] }} Dimensions */
/** @typedef {{ age_code: string, employment_type_code: string, education_code: string, sex_code: string, path: string }} SeriesIndexItem */
/** @typedef {{ metadata: { stats_data_id: string, title: string, unit: string, period: string, fetched_at: string }, dimensions: Dimensions, series: SeriesIndexItem[] }} WorkersOptions */
/** @typedef {{ year: string, value: number|null, reason?: string|null }} YearValue */
/** @typedef {{ age_code: string, age_name: string, employment_type_code: string, employment_type_name: string, education_code: string, education_name: string, sex_code: string, sex_name: string, unit: string, time_value_pairs: YearValue[] }} WorkersSeries */
export {};
