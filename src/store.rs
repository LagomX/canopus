use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::PathBuf;

/// Returns the root ~/.canopus directory path.
pub fn get_canopus_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Cannot determine home directory")
        .join(".canopus")
}

/// Returns the ~/.canopus/data/ directory path.
pub fn get_data_dir() -> PathBuf {
    get_canopus_dir().join("data")
}

/// Returns today's date as "YYYY-MM-DD".
pub fn get_today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Returns true if ~/.canopus has been initialized.
pub fn is_initialized() -> bool {
    get_canopus_dir().exists()
}

/// Reads and deserializes a JSON file; returns None if the file is missing or unparseable.
pub fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Option<T> {
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Serializes data as pretty-printed JSON and writes it to disk, creating parent dirs as needed.
pub fn write_json<T: Serialize + ?Sized>(
    path: &PathBuf,
    data: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(data)?;
    fs::write(path, content)?;
    Ok(())
}

/// Creates a directory and all its parents if they do not already exist.
pub fn ensure_dir(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path)?;
    Ok(())
}
