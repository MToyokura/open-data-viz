/** @typedef {Object} Contribution
 * @property {string} consumer_region_code
 * @property {string} consumer_region_name
 * @property {"reported" | "below_display_unit" | "no_fact" | "unknown_or_uncollected"} value_kind
 * @property {string} raw_value
 */

/** @typedef {Object} PrefectureMonth
 * @property {string} month
 * @property {string} consignor_prefecture_code
 * @property {string} consignor_prefecture_name
 * @property {string} reported_value_sum
 * @property {string} source_unit
 * @property {"valid" | "incomplete"} status
 * @property {number} reported_cells
 * @property {number} below_display_unit_cells
 * @property {number} no_fact_cells
 * @property {number} unknown_or_uncollected_cells
 */

/** @typedef {Object} YearData
 * @property {number} year
 * @property {string} table_id
 * @property {string} source_unit
 * @property {PrefectureMonth[]} months
 */

/** @typedef {Object} AggregationData
 * @property {2} schema_version
 * @property {"valid" | "incomplete"} status
 * @property {string} commodity
 * @property {string} commodity_label
 * @property {string[]} mapping_warnings
 * @property {YearData[]} years
 */

/** @typedef {Object} PrefectureDetail
 * @property {2} schema_version
 * @property {string} commodity
 * @property {string} consignor_prefecture_code
 * @property {{month: string, contributions: Contribution[]}[]} months
 */

/** @typedef {{key: string, label: string}} RelayProduct */

/** @typedef {Object} RelayProductIndex
 * @property {1} schema_version
 * @property {RelayProduct[]} products
 */

/** @typedef {Object} GeoCollection
 * @property {{properties?: {N03_001?: string}, geometry: unknown}[]} features
 */

/** @typedef {Object} RelaySankeyNode
 * @property {string} id
 * @property {string} name
 * @property {"source" | "target"} side
 * @property {string} color
 * @property {number} order
 */

/** @typedef {{source: string, target: string, value: number}} RelaySankeyLink */

/** @typedef {Object} RelaySankeyData
 * @property {RelaySankeyNode[]} nodes
 * @property {RelaySankeyLink[]} links
 * @property {number} total
 * @property {Record<Exclude<Contribution["value_kind"], "reported">, number>} missing
 */

export {};
