import { VITE_STORAGE_BASE_URL } from "#lib/paths.js";

export const WORKERS_DATA_PATH = `${VITE_STORAGE_BASE_URL}/workers_by_age_employment_type_education_sex`;
export const WORKERS_OPTIONS_PATH = `${WORKERS_DATA_PATH}/meta_data/options.json`;
export const WORKERS_SERIES_PATH = `${WORKERS_DATA_PATH}/transformed_data`;
