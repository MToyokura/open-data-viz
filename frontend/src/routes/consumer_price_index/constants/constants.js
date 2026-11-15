import { VITE_STORAGE_BASE_URL } from "#lib/paths.js";

export const CPI_DATA_PATH = `${VITE_STORAGE_BASE_URL}/consumer_price_index/transformed_data`;
export const CPI_ITEMS_PATH = `${CPI_DATA_PATH}/items.json`;
export const CPI_REGIONS_PATH = `${CPI_DATA_PATH}/regions.json`;

