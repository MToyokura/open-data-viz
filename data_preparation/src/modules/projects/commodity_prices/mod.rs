pub const COMMODITY_STATS_DATA_ID: &str = "0003421913";
pub const OUTPUT_DIR: &str = "output/commodity_prices";
pub const OUTPUT_DIR_META: &str = "output/commodity_prices/meta_data";
pub const OUTPUT_DIR_RAW: &str = "output/commodity_prices/raw_data";
pub const OUTPUT_DIR_TRANSFORMED: &str = "output/commodity_prices/transformed_data";

pub mod fetch_transform_write_commodity_data;
pub mod get_commodity_meta_info;
pub mod process_commodity_prices;
