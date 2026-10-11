mod modules;

use modules::estat_api::app_id::retrieve_estat_app_id;
use modules::prefectures::update_prefectures;
use modules::projects::commodity_prices::process_commodity_prices::process_commodity_prices;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if modules::projects::vegetable_production_relay::cli::run(&args) {
        return;
    }

    match args.first().map(String::as_str) {
        Some("--help") | Some("-h") => {
            print_usage();
        }
        Some("prefectures-update") => {
            if let Err(error) = update_prefectures() {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        Some("commodity-prices") => {
            let estat_app_id = match retrieve_estat_app_id() {
                Ok(app_id) => {
                    println!("ESTAT_APP_ID found");
                    app_id
                }
                Err(_) => {
                    eprintln!("ESTAT_APP_ID not found");
                    return;
                }
            };

            process_commodity_prices(&estat_app_id).unwrap();
        }
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
}

fn print_usage() {
    println!("Usage: cargo run -- <command> [arguments]");
    println!("Commands include commodity-prices, prefectures-update, and vegetable-relay.");
    println!(
        "Vegetable inspections: vegetable-relay-inspect-workbooks, vegetable-relay-acquire-product-observations, vegetable-relay-verify-product-mappings, vegetable-relay-verify-daikon-mapping, vegetable-relay-validate-m3-m5."
    );
}
