use crate::models::journal::JournalEntry;
use crate::store::{get_data_dir, get_today_str, is_initialized, read_json, write_json};
use chrono::Utc;
use colored::Colorize;
use std::io::{self, BufRead, Write};

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

    // Prompt before overwriting an existing entry
    if read_json::<JournalEntry>(&path).is_some() {
        println!(
            "{}",
            format!("A journal entry already exists for {}.", date_str).yellow()
        );
        if !confirm("Overwrite?") {
            println!("Aborted.");
            return Ok(());
        }
    }

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

    write_json(&path, &entry)?;
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

/// Prints a yes/no prompt and returns true if the user typed "y".
fn confirm(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().eq_ignore_ascii_case("y")
}
