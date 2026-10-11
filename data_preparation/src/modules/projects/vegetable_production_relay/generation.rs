use std::{error::Error, process::Command};

/// Generate every product JSON and refresh the runtime product index.
pub fn run() -> Result<(), Box<dyn Error>> {
    let project_dir = env!("CARGO_MANIFEST_DIR");
    let executable = std::env::current_exe()?;

    for script_name in [
        "acquire_product_observations.py",
        "generate_product_data.py",
    ] {
        let script = std::path::Path::new(project_dir)
            .join("inspections/vegetable_production_relay")
            .join(script_name);
        let status = Command::new("python3")
            .arg(script)
            .env("OPEN_DATA_VIZ_EXECUTABLE", &executable)
            .status()?;
        if !status.success() {
            return Err(format!("{script_name} failed ({status})").into());
        }
    }
    Ok(())
}
