pub(crate) mod app_id;
mod decoder;
mod error;
mod get;
pub(crate) mod get_meta_info;

// Keep catalog discovery available for future acquisitions.
pub(crate) mod get_stats_data;
#[allow(dead_code)]
mod get_stats_list;

pub(crate) use error::EstatError;
pub(crate) use get::EstatClient;
