use crate::models::task::{Priority, Task, TaskStatus};
use crate::store::{get_data_dir, get_today_str, is_initialized, read_json, write_json};
use colored::Colorize;
use std::path::PathBuf;

fn tasks_path() -> PathBuf {
    get_data_dir()
        .join("tasks")
        .join(format!("{}.json", get_today_str()))
}

fn load_tasks() -> Vec<Task> {
    read_json(&tasks_path()).unwrap_or_default()
}

fn save_tasks(tasks: &[Task]) -> Result<(), Box<dyn std::error::Error>> {
    write_json(&tasks_path(), tasks)
}

/// Returns false and prints an error if ~/.canopus is not initialized.
fn check_init() -> bool {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        false
    } else {
        true
    }
}

/// Resolves a 1-based numeric index or an exact id string to a Vec index.
fn resolve_task(tasks: &[Task], id_or_index: &str) -> Option<usize> {
    if let Ok(n) = id_or_index.parse::<usize>() {
        if n >= 1 && n <= tasks.len() {
            return Some(n - 1);
        }
    }
    tasks.iter().position(|t| t.id == id_or_index)
}

/// Adds a new task to today's task file.
pub fn run_add(
    title: String,
    priority_str: String,
    domain: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !check_init() {
        return Ok(());
    }

    let priority = Priority::from_str(&priority_str).ok_or_else(|| {
        format!(
            "Invalid priority '{}'. Use: high, medium, low",
            priority_str
        )
    })?;

    let today = get_today_str();
    let today_compact = today.replace("-", "");
    let mut tasks = load_tasks();
    let index = tasks.len() + 1;

    let task = Task {
        id: format!("task_{}_{:03}", today_compact, index),
        date: today,
        title: title.clone(),
        status: TaskStatus::Todo,
        priority,
        domain,
        skip_reason: None,
        notes: None,
    };

    tasks.push(task);
    save_tasks(&tasks)?;
    println!("{} {}", "Task added:".green().bold(), title);
    Ok(())
}

/// Lists all tasks for today with status icons and priority labels.
pub fn run_list() -> Result<(), Box<dyn std::error::Error>> {
    if !check_init() {
        return Ok(());
    }

    let tasks = load_tasks();
    if tasks.is_empty() {
        println!("{}", "No tasks for today. Add one with `canopus task add`.".yellow());
        return Ok(());
    }

    println!("{}", format!("Tasks — {}", get_today_str()).bold().white());
    println!("{}", "─".repeat(52));

    for (i, task) in tasks.iter().enumerate() {
        let icon = task.status.icon();
        let priority_label = match task.priority {
            Priority::High => "[H]".red().to_string(),
            Priority::Medium => "[M]".yellow().to_string(),
            Priority::Low => "[L]".dimmed().to_string(),
        };
        let domain = task
            .domain
            .as_deref()
            .map(|d| format!(" ({})", d).dimmed().to_string())
            .unwrap_or_default();
        let skip_note = task
            .skip_reason
            .as_deref()
            .map(|r| format!(" — {}", r).dimmed().to_string())
            .unwrap_or_default();

        println!(
            "  {}. {} {} {}{}{}",
            i + 1,
            icon,
            priority_label,
            task.title,
            domain,
            skip_note,
        );
    }

    println!("{}", "─".repeat(52));
    let done = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Done)
        .count();
    println!("  {}/{} done", done.to_string().green(), tasks.len());
    Ok(())
}

/// Marks a task as done by 1-based index or exact id.
pub fn run_done(id_or_index: String) -> Result<(), Box<dyn std::error::Error>> {
    if !check_init() {
        return Ok(());
    }

    let mut tasks = load_tasks();
    match resolve_task(&tasks, &id_or_index) {
        Some(idx) => {
            tasks[idx].status = TaskStatus::Done;
            let title = tasks[idx].title.clone();
            save_tasks(&tasks)?;
            println!("{} {}", "✓ Done:".green().bold(), title);
        }
        None => println!("{} '{}'", "Task not found:".red(), id_or_index),
    }
    Ok(())
}

/// Marks a task as skipped by 1-based index or exact id, with an optional reason.
pub fn run_skip(
    id_or_index: String,
    reason: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !check_init() {
        return Ok(());
    }

    let mut tasks = load_tasks();
    match resolve_task(&tasks, &id_or_index) {
        Some(idx) => {
            tasks[idx].status = TaskStatus::Skipped;
            tasks[idx].skip_reason = reason.clone();
            let title = tasks[idx].title.clone();
            save_tasks(&tasks)?;
            let note = reason
                .map(|r| format!(" ({})", r))
                .unwrap_or_default();
            println!("{} {}{}", "✗ Skipped:".yellow().bold(), title, note);
        }
        None => println!("{} '{}'", "Task not found:".red(), id_or_index),
    }
    Ok(())
}
