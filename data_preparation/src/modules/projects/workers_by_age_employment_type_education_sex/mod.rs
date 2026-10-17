//! Production pipeline for e-Stat table 0003006608.
//!
//! The table's classification codes are intentionally read from the fetched
//! metadata at runtime; they must not be duplicated as assumptions in Rust.

pub const STATS_DATA_ID: &str = "0003006608";
pub const OUTPUT_DIR: &str = "output/workers_by_age_employment_type_education_sex";
pub const OUTPUT_DIR_META: &str = "output/workers_by_age_employment_type_education_sex/meta_data";
pub const OUTPUT_DIR_RAW: &str = "output/workers_by_age_employment_type_education_sex/raw_data";
pub const OUTPUT_DIR_TRANSFORMED: &str =
    "output/workers_by_age_employment_type_education_sex/transformed_data";

pub mod inspection;
pub mod pipeline;

/// Run the production command. Returns false for unrelated commands.
pub fn run(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("workers-by-age-employment-type-education-sex") {
        return false;
    }

    let result = if args.get(1).map(String::as_str) == Some("--transform-only") {
        pipeline::transform()
    } else {
        let app_id = match crate::modules::estat_api::app_id::retrieve_estat_app_id() {
            Ok(app_id) => app_id,
            Err(_) => {
                eprintln!("ESTAT_APP_ID not found");
                std::process::exit(1);
            }
        };
        pipeline::acquire(&app_id)
    };
    if let Err(error) = result {
        eprintln!("Workers-by-age pipeline failed: {error}");
        std::process::exit(1);
    }
    true
}

/// Run the Rust e-Stat inspection capture command.
pub fn run_inspection(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("workers-by-age-inspect") {
        return false;
    }
    let mut limit = 3_u32;
    let mut scan_all = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--scan-all-values" => scan_all = true,
            "--limit" => {
                index += 1;
                limit = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(0);
            }
            "--help" | "-h" => {
                println!(
                    "Usage: cargo run -- workers-by-age-inspect [--limit N] [--scan-all-values]"
                );
                return true;
            }
            _ => {
                eprintln!("Unknown inspection option; use --help for usage.");
                std::process::exit(2);
            }
        }
        index += 1;
    }
    let app_id = match crate::modules::estat_api::app_id::retrieve_estat_app_id() {
        Ok(app_id) => app_id,
        Err(_) => {
            eprintln!("ESTAT_APP_ID not found");
            std::process::exit(1);
        }
    };
    if inspection::run(&app_id, limit, scan_all).is_err() {
        eprintln!("Inspection failed; details omitted to protect request credentials.");
        std::process::exit(1);
    }
    true
}
