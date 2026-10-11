mod modules;
use modules::estat_api::app_id::retrieve_estat_app_id;
use modules::prefectures::update_prefectures;
use modules::projects::commodity_prices::process_commodity_prices::process_commodity_prices;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("vegetable-relay-normalize-2024-daikon") => {
            let args: Vec<String> = std::env::args().collect();
            if args.len() != 6 {
                eprintln!(
                    "Usage: cargo run -- vegetable-relay-normalize-2024-daikon <observations-run-dir> <metadata-run-dir> <reviewed-codebook.json> <output-inspection-dir>"
                );
                std::process::exit(2);
            }
            match modules::projects::vegetable_production_relay::normalization::run(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
                std::path::Path::new(&args[4]),
                std::path::Path::new(&args[5]),
            ) {
                Ok(path) => println!("M5-A inspection only (NOT publishable): {}", path.display()),
                Err(error) => {
                    eprintln!("M5-A refused: {error}");
                    std::process::exit(1);
                }
            }
            return;
        }

        Some("vegetable-relay-normalize-2021-2023") => {
            let args: Vec<String> = std::env::args().collect();
            if args.len() != 6 {
                eprintln!(
                    "Usage: cargo run -- vegetable-relay-normalize-2021-2023 <observations-run-dir> <metadata-run-dir> <temporal-mapping.json> <output-inspection-dir>"
                );
                std::process::exit(2);
            }
            match modules::projects::vegetable_production_relay::normalization::run_2021_2023(
                std::path::Path::new(&args[2]),
                std::path::Path::new(&args[3]),
                std::path::Path::new(&args[4]),
                std::path::Path::new(&args[5]),
            ) {
                Ok(path) => println!("M5-B inspection only (NOT publishable): {}", path.display()),
                Err(error) => {
                    eprintln!("M5-B refused: {error}");
                    std::process::exit(1);
                }
            }
            return;
        }

        Some("vegetable-relay-acquire") | Some("vegetable-relay-acquisition-replay") => {
            let command = std::env::args().nth(1).unwrap();
            let first = std::env::args().nth(2).unwrap_or_default();
            let second = std::env::args().nth(3).unwrap_or_default();
            let result = if command == "vegetable-relay-acquire" {
                modules::projects::vegetable_production_relay::acquisition::acquire(
                    &std::env::var("ESTAT_APP_ID").unwrap_or_default(),
                    std::path::Path::new(&first),
                )
                .map(|p| format!("M4 raw capture: {} (publication blocked)", p.display()))
            } else {
                modules::projects::vegetable_production_relay::acquisition::replay(
                    std::path::Path::new(&first),
                    std::path::Path::new(&second),
                )
                .map(|r| {
                    format!(
                        "Verified {} raw observations; no semantic approval",
                        r.count
                    )
                })
            };
            match result {
                Ok(msg) => println!("{msg}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
            return;
        }

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
        Some("vegetable-relay-semantic-blocks") => {
            let path = std::env::args().nth(2).unwrap_or_default();
            if path.is_empty() {
                eprintln!("Usage: cargo run -- vegetable-relay-semantic-blocks <metadata-run-dir>");
                std::process::exit(2);
            }
            match modules::projects::vegetable_production_relay::semantic_probe::block_report(
                std::path::Path::new(&path),
            ) {
                Ok(output) => println!("Block evidence: {} (M3 remains blocked)", output.display()),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("vegetable-relay-semantic-probe") => {
            let path = std::env::args().nth(2).unwrap_or_default();
            let code = std::env::args().nth(3).unwrap_or_default();
            if path.is_empty() || code.is_empty() {
                eprintln!(
                    "Usage: cargo run -- vegetable-relay-semantic-probe <metadata-run-dir> <cat03-code>"
                );
                std::process::exit(2);
            }
            let id = std::env::var("ESTAT_APP_ID").unwrap_or_default();
            match modules::projects::vegetable_production_relay::semantic_probe::probe(
                &id,
                std::path::Path::new(&path),
                &code,
            ) {
                Ok(output) => println!("Probe captured: {} (M3 remains blocked)", output.display()),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("vegetable-relay-semantic-replay") => {
            let probe = std::env::args().nth(2).unwrap_or_default();
            let meta = std::env::args().nth(3).unwrap_or_default();
            if probe.is_empty() || meta.is_empty() {
                eprintln!(
                    "Usage: cargo run -- vegetable-relay-semantic-replay <probe-run-dir> <metadata-run-dir>"
                );
                std::process::exit(2);
            }
            if let Err(e) = modules::projects::vegetable_production_relay::semantic_probe::replay(
                std::path::Path::new(&probe),
                std::path::Path::new(&meta),
            ) {
                eprintln!("{e}");
                std::process::exit(1);
            }
            println!("Probe replay verified; M3 remains blocked");
            return;
        }
        Some("vegetable-relay-metadata-inspect") => {
            let id = std::env::var("ESTAT_APP_ID").unwrap_or_default();
            let table = std::env::args().nth(2).unwrap_or_default();
            if let Err(error) =
                modules::projects::vegetable_production_relay::metadata::inspect(&id, &table)
            {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        Some("vegetable-relay-metadata-replay") => {
            let path = std::env::args().nth(2).unwrap_or_default();
            if path.is_empty() {
                eprintln!("Usage: cargo run -- vegetable-relay-metadata-replay <metadata-run-dir>");
                std::process::exit(2);
            }
            if let Err(error) = modules::projects::vegetable_production_relay::metadata::replay(
                std::path::Path::new(&path),
            ) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            println!("Metadata capture verified; semantic gate remains blocked");
            return;
        }
        Some("vegetable-relay-metadata-report") => {
            let path = std::env::args().nth(2).unwrap_or_default();
            if path.is_empty() {
                eprintln!("Usage: cargo run -- vegetable-relay-metadata-report <metadata-run-dir>");
                std::process::exit(2);
            }
            if let Err(error) =
                modules::projects::vegetable_production_relay::metadata::write_report(
                    std::path::Path::new(&path),
                )
            {
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
