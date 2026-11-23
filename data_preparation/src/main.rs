mod modules;

use modules::estat_api::app_id::retrieve_estat_app_id;
use modules::prefectures::update_prefectures;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if modules::projects::marine_fishery_catch::run(&args) {
        return;
    }
    if modules::projects::asset_distribution::run(&args) {
        return;
    }
    if modules::projects::vegetable_production_relay::cli::run(&args) {
        return;
    }
    if modules::projects::workers_by_age_employment_type_education_sex::run(&args) {
        return;
    }
    if modules::projects::workers_by_age_employment_type_education_sex::run_inspection(&args) {
        return;
    }
    if modules::projects::consumer_price_index::run(&args) {
        return;
    }
    if modules::projects::consumer_price_index::run_inspection(&args) {
        return;
    }
    if modules::projects::social_security_budget_flow::run_inspection(&args) {
        return;
    }
    if modules::projects::social_security_budget_flow::run(&args) {
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
            if args.get(1).map(String::as_str) == Some("--transform-only") {
                if let Err(error) = modules::projects::commodity_prices::pipeline::transform() {
                    eprintln!("Commodity prices replay failed: {error}");
                    std::process::exit(1);
                }
                return;
            }
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

            if let Err(error) =
                modules::projects::commodity_prices::pipeline::acquire(&estat_app_id)
            {
                eprintln!("Commodity prices acquisition failed: {error}");
                std::process::exit(1);
            }
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
    println!("Usage: ./scripts/prepare.sh run -- <command> [arguments]");
    println!(
        "Commands include commodity-prices, prefectures-update, asset-distribution, vegetable-relay, workers-by-age-employment-type-education-sex, consumer-price-index, consumer-price-index-inspect, social-security-budget-flow, and marine-fishery-catch."
    );
    println!(
        "Vegetable inspections: vegetable-relay-inspect-workbooks, vegetable-relay-acquire-product-observations, vegetable-relay-verify-product-mappings, vegetable-relay-verify-daikon-mapping, vegetable-relay-validate-m3-m5."
    );
    println!("Workers data: workers-by-age-employment-type-education-sex [--transform-only].");
    println!("CPI data: consumer-price-index [--transform-only].");
    println!("Workers API inspection: workers-by-age-inspect [--limit N] [--scan-all-values].");
    println!("Social security API inspection: social-security-budget-flow-inspect.");
}
