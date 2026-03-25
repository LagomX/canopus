use crate::models::journal::JournalEntry;
use crate::store::{get_data_dir, get_today_str, is_initialized, write_json};
use chrono::Utc;
use colored::Colorize;
use std::fs;
use std::io::{self, BufRead};

/// Records a journal entry. Uses interactive stdin input when `text` is None.
pub fn run(
    text: Option<String>,
    date: Option<String>,
    mood: Option<u8>,
    energy: Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let date_str = date.unwrap_or_else(get_today_str);
    let path = get_data_dir()
        .join("journal")
        .join(format!("{}.json", date_str));

    // Validate optional scores
    if let Some(m) = mood {
        if !(1..=10).contains(&m) {
            return Err("Mood score must be between 1 and 10.".into());
        }
    }
    if let Some(e) = energy {
        if !(1..=10).contains(&e) {
            return Err("Energy score must be between 1 and 10.".into());
        }
    }

    let content = match text {
        Some(t) => t,
        None => read_multiline_input(),
    };

    if content.trim().is_empty() {
        println!("{}", "No content entered. Aborted.".yellow());
        return Ok(());
    }

    let now = Utc::now();
    let entry = JournalEntry {
        id: format!("journal_{}", now.format("%Y%m%d_%H%M%S")),
        timestamp: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        date: date_str.clone(),
        content,
        mood_score: mood,
        energy_score: energy,
        tags: vec![],
        intentions: vec![],
    };

    // Read existing entries (array format; fall back to single-object for old files)
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let mut entries: Vec<JournalEntry> = serde_json::from_str::<Vec<JournalEntry>>(&raw)
        .ok()
        .or_else(|| serde_json::from_str::<JournalEntry>(&raw).ok().map(|e| vec![e]))
        .unwrap_or_default();

    entries.push(entry);
    write_json(&path, &entries)?;
    println!(
        "{}",
        format!("Journal entry saved for {}.", date_str).green().bold()
    );
    Ok(())
}

/// Reads multi-line text from stdin, stopping after two consecutive empty lines.
fn read_multiline_input() -> String {
    println!(
        "{}",
        "Enter your journal entry (press Enter twice to finish):".cyan()
    );
    let stdin = io::stdin();
    let mut lines: Vec<String> = Vec::new();
    let mut consecutive_empty = 0usize;

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        if line.is_empty() {
            consecutive_empty += 1;
            if consecutive_empty >= 2 {
                break;
            }
            lines.push(line);
        } else {
            consecutive_empty = 0;
            lines.push(line);
        }
    }

    // Strip trailing blank lines
    while lines.last().map_or(false, |l: &String| l.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

