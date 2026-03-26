use crate::store::{ensure_dir, get_canopus_dir, get_data_dir};
use colored::Colorize;
use serde_json::json;
use std::fs;

/// Creates the ~/.canopus directory structure and writes a default config.json.
/// If the directory already exists, prints a warning and exits without overwriting.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let canopus_dir = get_canopus_dir();

    if canopus_dir.join("config.json").exists() {
        println!(
            "{} {}",
            "Already initialized:".yellow(),
            canopus_dir.display()
        );
        return Ok(());
    }

    let data_dir = get_data_dir();
    ensure_dir(&data_dir.join("journal"))?;
    ensure_dir(&data_dir.join("tasks"))?;
    ensure_dir(&data_dir.join("sleep"))?;
    ensure_dir(&data_dir.join("attention"))?;
    ensure_dir(&canopus_dir.join("principles"))?;
    ensure_dir(&canopus_dir.join("observations"))?;
    ensure_dir(&canopus_dir.join("reflections"))?;
    ensure_dir(&canopus_dir.join("reports"))?;

    let config = json!({
        "version": "0.1.0",
        "timezone": "local"
    });
    fs::write(
        canopus_dir.join("config.json"),
        serde_json::to_string_pretty(&config)?,
    )?;

    println!("{}", "Canopus initialized successfully!".green().bold());
    println!("  Data directory: {}", canopus_dir.display().to_string().cyan());
    Ok(())
}
