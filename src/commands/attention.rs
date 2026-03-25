use crate::models::attention::{
    AppCount, AppMinutes, CategoryMinutes, NotificationData, PickupData, ScreenTimeRecord,
    ScreenTimeUsage,
};
use crate::store::{get_data_dir, get_today_str, is_initialized, read_json, write_json};
use chrono::Local;
use colored::Colorize;
use std::io::{self, Write};

/// Interactively collects full attention/screen-time data and saves it for today.
pub fn run_today() -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let today = get_today_str();
    let path = get_data_dir()
        .join("attention")
        .join(format!("{}.json", today));

    if read_json::<ScreenTimeRecord>(&path).is_some() {
        println!(
            "{}",
            format!("An attention record already exists for {}.", today).yellow()
        );
        if !confirm("Overwrite?") {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("{}", "— Screen Time —".cyan().bold());

    // 1. Total minutes
    let total_minutes = read_u32("  Total screen time (minutes): ");

    // 2. Categories
    let cat_count = read_u32("  How many categories? ");
    let mut category_minutes = Vec::new();
    for i in 0..cat_count {
        let name = read_string(&format!("    Category {} name: ", i + 1));
        let minutes = read_u32(&format!("    Category '{}' minutes: ", name));
        category_minutes.push(CategoryMinutes { name, minutes });
    }

    // 3. Top apps (usage)
    let app_count = read_u32("  How many top apps? ");
    let mut top_apps = Vec::new();
    for i in 0..app_count {
        let name = read_string(&format!("    App {} name: ", i + 1));
        let minutes = read_u32(&format!("    App '{}' minutes: ", name));
        top_apps.push(AppMinutes { name, minutes });
    }

    println!("{}", "— Notifications —".cyan().bold());

    // 4. Total notifications
    let notif_total = read_u32("  Total notifications: ");

    // 5. Notification sources
    let notif_src_count = read_u32("  How many notification sources? ");
    let mut notif_apps = Vec::new();
    for i in 0..notif_src_count {
        let name = read_string(&format!("    Source {} name: ", i + 1));
        let count = read_u32(&format!("    '{}' count: ", name));
        notif_apps.push(AppCount { name, count });
    }

    println!("{}", "— Pickups —".cyan().bold());

    // 6. Total pickups
    let pickup_total = read_u32("  Total pickups: ");

    // 7. Pickup sources
    let pickup_src_count = read_u32("  How many pickup sources? ");
    let mut pickup_apps = Vec::new();
    for i in 0..pickup_src_count {
        let name = read_string(&format!("    Source {} name: ", i + 1));
        let count = read_u32(&format!("    '{}' count: ", name));
        pickup_apps.push(AppCount { name, count });
    }

    // 8. Notes
    let notes_raw = read_string("  Notes (press Enter to skip): ");
    let notes = if notes_raw.trim().is_empty() {
        None
    } else {
        Some(notes_raw.trim().to_string())
    };

    let captured_at = Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string();

    let record = ScreenTimeRecord {
        id: format!("attention_{}", today),
        date: today.clone(),
        source: "manual_from_screenshots".to_string(),
        captured_at,
        usage: ScreenTimeUsage {
            total_minutes,
            category_minutes,
            top_apps,
        },
        notifications: NotificationData {
            total: notif_total,
            top_apps: notif_apps,
        },
        pickups: PickupData {
            total: pickup_total,
            top_apps: pickup_apps,
        },
        notes,
    };

    write_json(&path, &record)?;
    println!(
        "{}",
        format!(
            "Attention recorded: {}min, {} pickups, {} notifications.",
            total_minutes, pickup_total, notif_total
        )
        .green()
        .bold()
    );
    Ok(())
}

/// Reads a non-negative integer from stdin, retrying on invalid input.
fn read_u32(prompt: &str) -> u32 {
    loop {
        print!("{}", prompt);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        match input.trim().parse::<u32>() {
            Ok(n) => return n,
            Err(_) => println!("{}", "Please enter a valid number (0 or more).".yellow()),
        }
    }
}

/// Reads a non-empty string from stdin, retrying if blank.
fn read_string(prompt: &str) -> String {
    loop {
        print!("{}", prompt);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let trimmed = input.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
        println!("{}", "Please enter a value.".yellow());
    }
}

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().eq_ignore_ascii_case("y")
}
