//! Acquisition and preparation for e-Stat's 2025-base Consumer Price Index.

use std::path::PathBuf;

pub const STATS_DATA_ID: &str = "0004052037";
pub const OUTPUT_DIR: &str = "output/consumer_price_index";
pub const OUTPUT_DIR_META: &str = "output/consumer_price_index/meta_data";
pub const OUTPUT_DIR_RAW: &str = "output/consumer_price_index/raw_data";
pub const OUTPUT_DIR_TRANSFORMED: &str = "output/consumer_price_index/transformed_data";
pub const PAGE_SIZE: u32 = 100_000;
pub const ITEMS_PER_CHUNK: usize = 10;

mod inspection;
pub mod pipeline;
pub use inspection::run_inspection;

/// Run the CPI production command. `--transform-only` reuses saved API pages.
pub fn run(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("consumer-price-index") {
        return false;
    }
    let result = if args.get(1).map(String::as_str) == Some("--transform-only") {
        pipeline::transform()
    } else {
        match crate::modules::estat_api::app_id::retrieve_estat_app_id() {
            Ok(app_id) => pipeline::acquire(&app_id),
            Err(_) => {
                eprintln!("ESTAT_APP_ID not found");
                std::process::exit(1);
            }
        }
    };
    if let Err(error) = result {
        eprintln!("Consumer price index pipeline failed: {error}");
        std::process::exit(1);
    }
    true
}

pub(crate) fn path(directory: &str) -> PathBuf {
    PathBuf::from(directory)
}
