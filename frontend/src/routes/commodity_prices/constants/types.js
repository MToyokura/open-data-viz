/** @typedef {{ cat02_code: string, cat02_name: string }} Cat02LookupItem */

/** @typedef {Cat02LookupItem[]} Cat02Lookup */

/** @typedef {{ area_code: string, area_name: string }} AreaLookupItem */

/** @typedef {AreaLookupItem[]} AreaLookup */

/**
 * @typedef {Object} CommodityPrices
 * @property {string} cat02_code
 * @property {string} cat02_name
 * @property {string} area_code
 * @property {string} area_name
 * @property {{ time: string, value: string }[]} time_value_pairs
 */

export {};
