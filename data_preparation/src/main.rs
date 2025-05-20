mod modules;
use modules::estat_api::app_id::retrieve_estat_app_id;
use modules::projects::commodity_prices::process_commodity_prices::process_commodity_prices;

fn main() {
    let estat_app_id = match retrieve_estat_app_id() {
        Ok(app_id) => {
            println!("ESTAT_APP_ID found");
            app_id // Return the ESTAT_APP_ID if successful
        }
        Err(_) => {
            eprintln!("ESTAT_APP_ID not found");
            return; // Exit if the ESTAT_APP_ID cannot be retrieved
        }
    };

    process_commodity_prices(&estat_app_id).unwrap();
}
