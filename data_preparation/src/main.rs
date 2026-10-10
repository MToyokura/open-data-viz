mod modules;
use modules::estat_api::app_id::retrieve_estat_app_id;
use modules::prefectures::update_prefectures;
use modules::projects::commodity_prices::process_commodity_prices::process_commodity_prices;

fn main() {
    let command = std::env::args().nth(1);
    match command.as_deref() {
        Some("--help") | Some("-h") => {
            print_usage();
            return;
        }
        Some("prefectures-update") => {
            if let Err(error) = update_prefectures() {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        Some("commodity-prices") => {}
        None => {
            print_usage();
            std::process::exit(2);
        }
        Some(command) => {
            eprintln!("Unknown command: {command}");
            print_usage();
            std::process::exit(2);
        }
    }

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

fn print_usage() {
    println!(
        "Usage: cargo run -- [commodity-prices|prefectures-update]"
    );
}
