use std::path::Path;

use super::{
    OUTPUT_DIR, acquisition, aggregation, assessment, discovery, inspection, metadata,
    normalization, product_index, semantic_probe,
};

/// Runs a vegetable-relay command, returning whether the command was recognized.
pub fn run(args: &[String]) -> bool {
    let Some(command) = args.first().map(String::as_str) else {
        return false;
    };

    match command {
        "vegetable-relay" => generate_all(),
        "vegetable-relay-normalize-2021-2023" => normalize(args),
        "vegetable-relay-aggregate-2021-2023" => aggregate(args),
        "vegetable-relay-product-index" => generate_product_index(args),
        "vegetable-relay-acquire" | "vegetable-relay-acquisition-replay" => acquire_or_replay(args),
        "vegetable-relay-inspect" => inspect(),
        "vegetable-relay-discover" => discover(),
        "vegetable-relay-assess" => assess(args),
        "vegetable-relay-semantic-blocks" => semantic_blocks(args),
        "vegetable-relay-semantic-probe" => semantic_probe(args),
        "vegetable-relay-semantic-replay" => replay_semantic_probe(args),
        "vegetable-relay-metadata-inspect" => inspect_metadata(args),
        "vegetable-relay-metadata-replay" => replay_metadata(args),
        "vegetable-relay-metadata-report" => report_metadata(args),
        "vegetable-relay-discovery-replay" => replay_discovery(args),
        "vegetable-relay-replay" => replay_inspection(args),
        _ => false,
    }
}

fn generate_all() -> bool {
    if let Err(error) = super::generation::run() {
        fail(error);
    }
    true
}

fn normalize(args: &[String]) -> bool {
    if args.len() != 5 {
        usage(
            "vegetable-relay-normalize-2021-2023 <observations-run-dir> <metadata-run-dir> <temporal-mapping.json> <output-inspection-dir>",
        );
    }

    match normalization::run_2021_2023(
        Path::new(&args[1]),
        Path::new(&args[2]),
        Path::new(&args[3]),
        Path::new(&args[4]),
    ) {
        Ok(path) => println!("Normalized 2021–2023 records: {}", path.display()),
        Err(error) => fail(format!("M5-B refused: {error}")),
    }
    true
}

fn aggregate(args: &[String]) -> bool {
    if !(6..=9).contains(&args.len()) {
        usage(
            "vegetable-relay-aggregate-2021-2023 <2021-m5b.json> <2022-m5b.json> <2023-m5b.json> <metadata-runs-dir> <temporal-mapping.json> [output-dir [commodity-key commodity-label]]",
        );
    }

    let output_dir = args.get(6).map(String::as_str).unwrap_or(OUTPUT_DIR);
    if let Err(error) = std::fs::create_dir_all(output_dir) {
        fail(format!("Could not create output directory: {error}"));
    }
    let commodity = args.get(7).map(String::as_str).unwrap_or("veg-01");
    let commodity_label = args.get(8).map(String::as_str).unwrap_or("だいこん");
    let output_path = Path::new(output_dir).join(format!("{commodity}.json"));

    match aggregation::run_2021_2023_product(
        Path::new(&args[1]),
        Path::new(&args[2]),
        Path::new(&args[3]),
        Path::new(&args[4]),
        Path::new(&args[5]),
        &output_path,
        commodity,
        commodity_label,
    ) {
        Ok(path) => println!("Generated 2021–2023 aggregation: {}", path.display()),
        Err(error) => fail(format!("M6 aggregation refused: {error}")),
    }
    true
}

fn generate_product_index(args: &[String]) -> bool {
    let output_dir = args.get(1).map(String::as_str).unwrap_or(OUTPUT_DIR);
    match product_index::generate(Path::new(output_dir)) {
        Ok(path) => println!("Generated product index: {}", path.display()),
        Err(error) => fail(format!("Product index generation refused: {error}")),
    }
    true
}

fn acquire_or_replay(args: &[String]) -> bool {
    let command = &args[0];
    let first = args.get(1).map(String::as_str).unwrap_or_default();
    let second = args.get(2).map(String::as_str).unwrap_or_default();
    let result = if command == "vegetable-relay-acquire" {
        acquisition::acquire(
            &std::env::var("ESTAT_APP_ID").unwrap_or_default(),
            Path::new(first),
        )
        .map(|path| format!("Captured raw e-Stat observations: {}", path.display()))
    } else {
        acquisition::replay(Path::new(first), Path::new(second))
            .map(|replay| format!("Replayed {} raw observations successfully", replay.count))
    };
    match result {
        Ok(message) => println!("{message}"),
        Err(error) => fail(error),
    }
    true
}

fn inspect() -> bool {
    let id = match std::env::var("ESTAT_APP_ID") {
        Ok(id) => id,
        Err(_) => fail("ESTAT_APP_ID not found"),
    };
    if let Err(error) = inspection::inspect_stats_list(&id) {
        fail(error);
    }
    true
}

fn discover() -> bool {
    let id = std::env::var("ESTAT_APP_ID").unwrap_or_default();
    if let Err(error) = discovery::inspect(&id) {
        fail(error);
    }
    true
}

fn assess(args: &[String]) -> bool {
    let path = required_path(args, "vegetable-relay-assess <catalogue-run-dir>");
    if let Err(error) = assessment::run(Path::new(path)) {
        fail(error);
    }
    true
}

fn semantic_blocks(args: &[String]) -> bool {
    let path = required_path(args, "vegetable-relay-semantic-blocks <metadata-run-dir>");
    match semantic_probe::block_report(Path::new(path)) {
        Ok(output) => println!("Block evidence: {} (M3 remains blocked)", output.display()),
        Err(error) => fail(error),
    }
    true
}

fn semantic_probe(args: &[String]) -> bool {
    let path = args.get(1).map(String::as_str).unwrap_or_default();
    let code = args.get(2).map(String::as_str).unwrap_or_default();
    if path.is_empty() || code.is_empty() {
        usage("vegetable-relay-semantic-probe <metadata-run-dir> <cat03-code>");
    }
    let id = std::env::var("ESTAT_APP_ID").unwrap_or_default();
    match semantic_probe::probe(&id, Path::new(path), code) {
        Ok(output) => println!("Probe captured: {} (M3 remains blocked)", output.display()),
        Err(error) => fail(error),
    }
    true
}

fn replay_semantic_probe(args: &[String]) -> bool {
    let (probe, metadata) = required_paths(
        args,
        "vegetable-relay-semantic-replay <probe-run-dir> <metadata-run-dir>",
    );
    if let Err(error) = semantic_probe::replay(Path::new(probe), Path::new(metadata)) {
        fail(error);
    }
    println!("Probe replay verified; M3 remains blocked");
    true
}

fn inspect_metadata(args: &[String]) -> bool {
    let id = std::env::var("ESTAT_APP_ID").unwrap_or_default();
    let table = args.get(1).map(String::as_str).unwrap_or_default();
    if let Err(error) = metadata::inspect(&id, table) {
        fail(error);
    }
    true
}

fn replay_metadata(args: &[String]) -> bool {
    let path = required_path(args, "vegetable-relay-metadata-replay <metadata-run-dir>");
    if let Err(error) = metadata::replay(Path::new(path)) {
        fail(error);
    }
    println!("Metadata capture verified; semantic gate remains blocked");
    true
}

fn report_metadata(args: &[String]) -> bool {
    let path = required_path(args, "vegetable-relay-metadata-report <metadata-run-dir>");
    if let Err(error) = metadata::write_report(Path::new(path)) {
        fail(error);
    }
    true
}

fn replay_discovery(args: &[String]) -> bool {
    let path = required_path(args, "vegetable-relay-discovery-replay <run-dir>");
    if let Err(error) = discovery::replay(Path::new(path)) {
        fail(error);
    }
    true
}

fn replay_inspection(args: &[String]) -> bool {
    let path = required_path(args, "vegetable-relay-replay <run-dir>");
    if let Err(error) = inspection::replay(Path::new(path)) {
        fail(error);
    }
    true
}

fn required_path<'a>(args: &'a [String], command_usage: &str) -> &'a str {
    let path = args.get(1).map(String::as_str).unwrap_or_default();
    if path.is_empty() {
        usage(command_usage);
    }
    path
}

fn required_paths<'a>(args: &'a [String], command_usage: &str) -> (&'a str, &'a str) {
    let first = args.get(1).map(String::as_str).unwrap_or_default();
    let second = args.get(2).map(String::as_str).unwrap_or_default();
    if first.is_empty() || second.is_empty() {
        usage(command_usage);
    }
    (first, second)
}

fn usage(command_usage: &str) -> ! {
    eprintln!("Usage: cargo run -- {command_usage}");
    std::process::exit(2);
}

fn fail(error: impl std::fmt::Display) -> ! {
    eprintln!("{error}");
    std::process::exit(1);
}
