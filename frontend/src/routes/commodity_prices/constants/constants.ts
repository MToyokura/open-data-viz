import { VITE_STORAGE_BASE_URL } from "$lib/paths";

export const COMMODITY_PRICES_DATA_PATH = `${VITE_STORAGE_BASE_URL}/commodity_prices`;

export const COMMODITY_PRICES_TRANSFORMED_DATA_PATH = `${COMMODITY_PRICES_DATA_PATH}/transformed_data`;
export const CAT02_LOOKUP_DATA_PATH = `${COMMODITY_PRICES_DATA_PATH}/meta_data/cat02_lookup.json`;
export const AREA_LOOKUP_DATA_PATH = `${COMMODITY_PRICES_DATA_PATH}/meta_data/area_lookup.json`;
