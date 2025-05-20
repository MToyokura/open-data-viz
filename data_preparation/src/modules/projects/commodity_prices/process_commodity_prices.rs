use crate::modules::projects::commodity_prices::{
    fetch_transform_write_commodity_data::{
        GetCommodityStatsDataParams, fetch_transform_write_commodity_data,
    },
    get_commodity_meta_info::{CommodityMetaData, fetch_write_transform_commodity_metadata},
};

/// Fetches, transforms, and writes all commodity prices data to files.
pub fn process_commodity_prices(app_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let commodity_meta_data: CommodityMetaData =
        match fetch_write_transform_commodity_metadata(app_id) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error fetching commodity meta data: {}", e);
                return Err(e); // Directly propagate the error from get_commodity_meta_info
            }
        };

    // Fetch for each cat02_code, transform, and write the data
    for (_i, cat02_entry) in commodity_meta_data.cat02_lookup.iter().enumerate() {
        let params = GetCommodityStatsDataParams {
            app_id,
            cat02_entry: &cat02_entry,
            area_lookup: &commodity_meta_data.area_lookup,
        };

        fetch_transform_write_commodity_data(params)?;
        println!(
            "  Successfully fetched and transformed data for {}",
            &cat02_entry.cat02_code
        );
    }

    Ok(())
}
