mod modules;
use modules::estat_api::app_id::retrieve_estat_app_id;
use modules::prefectures::update_prefectures;
use modules::projects::commodity_prices::process_commodity_prices::process_commodity_prices;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("vegetable-relay-inspect") => {
            let id = match std::env::var("ESTAT_APP_ID") {
                Ok(id) => id,
                Err(_) => {
                    eprintln!("ESTAT_APP_ID not found");
                    std::process::exit(1);
                }
            };
            if let Err(error) =
                modules::projects::vegetable_production_relay::inspection::inspect_stats_list(&id)
            {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        Some("vegetable-relay-discover") => {
            let id = std::env::var("ESTAT_APP_ID").unwrap_or_default();
            if let Err(error) =
                modules::projects::vegetable_production_relay::discovery::inspect(&id)
            {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        Some("vegetable-relay-assess") => {
            let path = std::env::args().nth(2).unwrap_or_default();
            if path.is_empty() {
                eprintln!("Usage: cargo run -- vegetable-relay-assess <catalogue-run-dir>");
                std::process::exit(2);
            }
            if let Err(error) = modules::projects::vegetable_production_relay::assessment::run(
                std::path::Path::new(&path),
            ) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        Some("vegetable-relay-discovery-replay") => {
            let path = std::env::args().nth(2).unwrap_or_default();
            if path.is_empty() {
                eprintln!("Usage: cargo run -- vegetable-relay-discovery-replay <run-dir>");
                std::process::exit(2);
            }
            if let Err(error) = modules::projects::vegetable_production_relay::discovery::replay(
                std::path::Path::new(&path),
            ) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        Some("vegetable-relay-replay") => {
            let path = std::env::args().nth(2).unwrap_or_default();
            if path.is_empty() {
                eprintln!("Usage: cargo run -- vegetable-relay-replay <run-dir>");
                std::process::exit(2);
            }
            if let Err(error) = modules::projects::vegetable_production_relay::inspection::replay(
                std::path::Path::new(&path),
            ) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        _ => {}
    }
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
    println!("Usage: cargo run -- [commodity-prices|prefectures-update]");
}
