use crate::models::sleep::SleepRecord;
use crate::store::{get_data_dir, get_today_str, is_initialized, read_json, write_json};
use colored::Colorize;
use std::io::{self, Write};

/// Records sleep data for today (or prompts to overwrite if a record exists).
pub fn run(
    hours: f64,
    quality: u8,
    bedtime: Option<String>,
    wake: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    if !(1..=5).contains(&quality) {
        return Err("Quality score must be between 1 and 5.".into());
    }
    if hours <= 0.0 || hours > 24.0 {
        return Err("Duration must be between 0 and 24 hours.".into());
    }

    let today = get_today_str();
    let path = get_data_dir()
        .join("sleep")
        .join(format!("{}.json", today));

    if read_json::<SleepRecord>(&path).is_some() {
        println!(
            "{}",
            format!("A sleep record already exists for {}.", today).yellow()
        );
        if !confirm("Overwrite?") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let record = SleepRecord {
        id: format!("sleep_{}", today.replace("-", "")),
        date: today.clone(),
        duration_hours: hours,
        quality_score: quality,
        bedtime,
        wake_time: wake,
        notes: None,
    };

    write_json(&path, &record)?;
    println!(
        "{}",
        format!("Sleep recorded: {:.1}h, quality {}/5.", hours, quality)
            .green()
            .bold()
    );
    Ok(())
}

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().eq_ignore_ascii_case("y")
}
