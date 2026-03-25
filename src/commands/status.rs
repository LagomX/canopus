use crate::models::{
    attention::ScreenTimeRecord, journal::JournalEntry, sleep::SleepRecord, task::Task,
};
use crate::models::task::TaskStatus;
use crate::store::{get_data_dir, get_today_str, is_initialized, read_json};
use colored::Colorize;

const LABEL_WIDTH: usize = 13; // label column padded to this width
const PREFIX: usize = 2;       // leading spaces inside the box
const MIN_RIGHT_PAD: usize = 2; // minimum trailing spaces before the border

/// Builds (plain_text, colored_text) for the screen-time status row.
fn screen_status(rec: &ScreenTimeRecord) -> (String, String) {
    let plain = format!(
        "✓ {}min  {} pickups  {} notifications",
        rec.usage.total_minutes, rec.pickups.total, rec.notifications.total
    );
    let colored = plain.green().to_string();
    (plain, colored)
}

/// Prints the full status box, computing its width to fit the widest row.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let today = get_today_str();
    let data = get_data_dir();

    let journal: Option<JournalEntry> =
        read_json(&data.join("journal").join(format!("{}.json", today)));
    let tasks: Option<Vec<Task>> =
        read_json(&data.join("tasks").join(format!("{}.json", today)));
    let sleep: Option<SleepRecord> =
        read_json(&data.join("sleep").join(format!("{}.json", today)));
    let screen: Option<ScreenTimeRecord> =
        read_json(&data.join("attention").join(format!("{}.json", today)));

    // Build every row as (label, plain_status, colored_status)
    let journal_row = if journal.is_some() {
        ("Journal", "✓ recorded".to_string(), "✓ recorded".green().to_string())
    } else {
        ("Journal", "✗ missing".to_string(), "✗ missing".yellow().to_string())
    };

    let tasks_row = match &tasks {
        Some(t) if !t.is_empty() => {
            let done = t.iter().filter(|t| t.status == TaskStatus::Done).count();
            let plain = format!("{} tasks ({}✓)", t.len(), done);
            let colored = plain.green().to_string();
            ("Tasks", plain, colored)
        }
        _ => ("Tasks", "✗ missing".to_string(), "✗ missing".yellow().to_string()),
    };

    let sleep_row = if sleep.is_some() {
        ("Sleep", "✓ recorded".to_string(), "✓ recorded".green().to_string())
    } else {
        ("Sleep", "✗ missing".to_string(), "✗ missing".yellow().to_string())
    };

    let screen_row = match &screen {
        Some(rec) => {
            let (plain, colored) = screen_status(rec);
            ("Screen Time", plain, colored)
        }
        None => ("Screen Time", "✗ missing".to_string(), "✗ missing".yellow().to_string()),
    };

    let rows = [journal_row, tasks_row, sleep_row, screen_row];

    // Compute box inner width to fit the widest row and the title
    let title = format!("Canopus — {}", today);
    let title_need = PREFIX + title.chars().count() + MIN_RIGHT_PAD;
    let inner = rows
        .iter()
        .map(|(_, plain, _)| PREFIX + LABEL_WIDTH + plain.chars().count() + MIN_RIGHT_PAD)
        .chain(std::iter::once(title_need))
        .max()
        .unwrap_or(29);

    // Header
    println!("┌{}┐", "─".repeat(inner));
    let title_pad = inner - PREFIX - title.chars().count();
    println!("│  {}{}│", title.bold().white(), " ".repeat(title_pad));
    println!("├{}┤", "─".repeat(inner));

    // Rows
    for (label, plain, colored) in &rows {
        let right_pad = inner - PREFIX - LABEL_WIDTH - plain.chars().count();
        println!("│  {:<LABEL_WIDTH$}{}{}│", label, colored, " ".repeat(right_pad));
    }

    println!("└{}┘", "─".repeat(inner));
    Ok(())
}
