//! Inspection and production entry points for the 2024 social security accounts.
//! Classification rules are to be finalized from the captured table metadata.

pub const INCOME_TABLE_ID: &str = "0004050702";
pub const EXPENDITURE_TABLE_ID: &str = "0004050703";

pub mod inspection;
pub mod pipeline;

pub const OUTPUT_DIR: &str = "data_preparation/output/social_security_budget_flow";

pub fn run_inspection(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("social-security-budget-flow-inspect") {
        return false;
    }
    if args
        .get(1)
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!("Usage: cargo run -- social-security-budget-flow-inspect");
        println!(
            "Captures metadata and a small response sample for tables {INCOME_TABLE_ID} and {EXPENDITURE_TABLE_ID}."
        );
        return true;
    }
    if args.len() != 1 {
        eprintln!("Unknown inspection option; use --help for usage.");
        std::process::exit(2);
    }
    let app_id = match crate::modules::estat_api::app_id::retrieve_estat_app_id() {
        Ok(app_id) => app_id,
        Err(_) => {
            eprintln!("ESTAT_APP_ID not found");
            std::process::exit(1);
        }
    };
    if let Err(error) = inspection::run_inspection(&app_id) {
        eprintln!(
            "Inspection failed: {}",
            error.to_string().replace(&app_id, "[redacted]")
        );
        std::process::exit(1);
    }
    true
}

pub fn run(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("social-security-budget-flow") {
        return false;
    }
    let app_id = match crate::modules::estat_api::app_id::retrieve_estat_app_id() {
        Ok(app_id) => app_id,
        Err(_) => {
            eprintln!("ESTAT_APP_ID not found");
            std::process::exit(1);
        }
    };
    if let Err(error) = pipeline::run(&app_id) {
        eprintln!(
            "Social security budget flow failed: {}",
            error.to_string().replace(&app_id, "[redacted]")
        );
        std::process::exit(1);
    }
    true
}
