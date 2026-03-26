use crate::models::task::{Quadrant, Task, TaskStatus};
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

fn check_init() -> bool {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        false
    } else {
        true
    }
}

fn resolve_task(tasks: &[Task], id_or_index: &str) -> Option<usize> {
    if let Ok(n) = id_or_index.parse::<usize>() {
        if n >= 1 && n <= tasks.len() {
            return Some(n - 1);
        }
    }
    tasks.iter().position(|t| t.id == id_or_index)
}

/// Adds a new task for today.
/// `quadrant_str` accepts: q1, q2, q3, q4, high, medium, low (default: q2)
pub fn run_add(
    title: String,
    quadrant_str: String,
    domain: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !check_init() {
        return Ok(());
    }

    let quadrant = Quadrant::from_str(&quadrant_str).ok_or_else(|| {
        format!(
            "Invalid quadrant '{}'. Use: q1, q2, q3, q4 (or high/medium/low)",
            quadrant_str
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
        quadrant,
        domain,
        skip_reason: None,
        notes: None,
    };

    tasks.push(task);
    save_tasks(&tasks)?;
    println!("{} {}", "Task added:".green().bold(), title);
    Ok(())
}

/// Lists all tasks for today.
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
        let quad_label = match task.quadrant {
            Quadrant::Q1 => "[Q1]".red().to_string(),
            Quadrant::Q2 => "[Q2]".blue().to_string(),
            Quadrant::Q3 => "[Q3]".yellow().to_string(),
            Quadrant::Q4 => "[Q4]".dimmed().to_string(),
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
            quad_label,
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

/// Marks a task as skipped by 1-based index or exact id.
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
