use std::{error::Error, process::Command};

/// Generate every product JSON and refresh the runtime product index.
pub fn run() -> Result<(), Box<dyn Error>> {
    let project_dir = env!("CARGO_MANIFEST_DIR");
    let script = std::path::Path::new(project_dir)
        .join("inspections/vegetable_production_relay/generate_product_data.py");
    let executable = std::env::current_exe()?;
    let status = Command::new("python3")
        .arg(script)
        .env("OPEN_DATA_VIZ_EXECUTABLE", executable)
        .status()?;

    if !status.success() {
        return Err(format!("vegetable data generation failed ({status})").into());
    }
    Ok(())
}
