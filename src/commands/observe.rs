use crate::models::attention::ScreenTimeRecord;
use crate::models::journal::JournalEntry;
use crate::models::observation::{Observation, ObservationSource};
use crate::models::sleep::SleepRecord;
use crate::models::task::{Task, TaskStatus};
use crate::store::{get_canopus_dir, get_data_dir, get_today_str, is_initialized, read_json};
use chrono::Local;
use colored::Colorize;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const OLLAMA_MODEL: &str = "qwen2.5:7b";

// ── Storage ───────────────────────────────────────────────────────────────────

fn get_observations_dir() -> PathBuf {
    get_canopus_dir().join("observations")
}

pub fn load_observations(date: &str) -> Vec<Observation> {
    let path = get_observations_dir().join(format!("{}.json", date));
    if !path.exists() {
        return vec![];
    }
    let content = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save_observations(date: &str, obs: &[Observation]) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_observations_dir().join(format!("{}.json", date));
    fs::write(&path, serde_json::to_string_pretty(obs)?)?;
    Ok(())
}

// ── Public entry points ───────────────────────────────────────────────────────

pub fn run(date: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }
    let date_str = date.unwrap_or_else(get_today_str);
    println!("{}", format!("Generating observations for {}...", date_str).cyan());
    generate(&date_str, ObservationSource::Manual)
}

/// Called automatically from analyze after printing the analysis.
pub fn run_auto(date: &str) -> Result<(), Box<dyn std::error::Error>> {
    generate(date, ObservationSource::Auto)
}

// ── Core logic ────────────────────────────────────────────────────────────────

fn generate(date: &str, source: ObservationSource) -> Result<(), Box<dyn std::error::Error>> {
    let data = get_data_dir();

    let journal: Option<JournalEntry> = {
        let p = data.join("journal").join(format!("{}.json", date));
        fs::read_to_string(&p).ok().and_then(|c| {
            serde_json::from_str::<Vec<JournalEntry>>(&c)
                .ok()
                .and_then(|mut v| v.pop())
                .or_else(|| serde_json::from_str::<JournalEntry>(&c).ok())
        })
    };
    let tasks: Option<Vec<Task>> =
        read_json(&data.join("tasks").join(format!("{}.json", date)));
    let sleep: Option<SleepRecord> =
        read_json(&data.join("sleep").join(format!("{}.json", date)));
    let screen: Option<ScreenTimeRecord> =
        read_json(&data.join("attention").join(format!("{}.json", date)));

    if journal.is_none() && tasks.is_none() && sleep.is_none() && screen.is_none() {
        return Err(format!(
            "No data found for {}. Record data first with: canopus journal, canopus task, etc.",
            date
        )
        .into());
    }

    let prompt = build_prompt(date, &journal, tasks.as_deref(), &sleep, &screen);
    let raw = call_ollama(&prompt)?;

    let mut existing = load_observations(date);
    let start_idx = existing.len() + 1;
    let date_compact = date.replace('-', "");
    let now_str = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let new_obs: Vec<Observation> = raw
        .lines()
        .filter(|l| l.trim_start().starts_with('-'))
        .enumerate()
        .map(|(i, line)| {
            let content = line.trim_start_matches('-').trim().to_string();
            Observation {
                id: format!("obs_{}_{:03}", date_compact, start_idx + i),
                date: date.to_string(),
                content,
                source,
                tags: vec![],
                created_at: now_str.clone(),
            }
        })
        .filter(|o| !o.content.is_empty())
        .collect();

    if new_obs.is_empty() {
        println!("{}", "No observations extracted from model output.".yellow());
        return Ok(());
    }

    for obs in &new_obs {
        println!("  {} {}: {}", "✓".green(), obs.id.dimmed(), obs.content);
    }

    existing.extend(new_obs);
    save_observations(date, &existing)?;

    Ok(())
}

// ── Prompt ────────────────────────────────────────────────────────────────────

fn build_prompt(
    date: &str,
    journal: &Option<JournalEntry>,
    tasks: Option<&[Task]>,
    sleep: &Option<SleepRecord>,
    screen: &Option<ScreenTimeRecord>,
) -> String {
    let (journal_content, mood_str, energy_str) = match journal {
        Some(j) => (
            j.content.clone(),
            j.mood_score.map(|m| m.to_string()).unwrap_or_else(|| "?".to_string()),
            j.energy_score.map(|e| e.to_string()).unwrap_or_else(|| "?".to_string()),
        ),
        None => ("(无日记)".to_string(), "?".to_string(), "?".to_string()),
    };

    let (done_str, skipped_str, todo_str, skipped_detail) = match tasks {
        Some(t) => {
            let done = t.iter().filter(|t| t.status == TaskStatus::Done).count();
            let skipped = t.iter().filter(|t| t.status == TaskStatus::Skipped).count();
            let todo = t.iter().filter(|t| t.status == TaskStatus::Todo).count();
            let detail: Vec<String> = t
                .iter()
                .filter(|t| t.status == TaskStatus::Skipped)
                .map(|t| match &t.skip_reason {
                    Some(r) => format!("「{}」({})", t.title, r),
                    None => format!("「{}」", t.title),
                })
                .collect();
            let detail_str = if detail.is_empty() {
                "(无)".to_string()
            } else {
                detail.join("、")
            };
            (
                done.to_string(),
                skipped.to_string(),
                todo.to_string(),
                detail_str,
            )
        }
        None => (
            "?".to_string(),
            "?".to_string(),
            "?".to_string(),
            "(无任务数据)".to_string(),
        ),
    };

    let (total_min, productive_min, entertainment_min, pickups, notifications) = match screen {
        Some(s) => {
            let productive: u32 = s
                .usage
                .category_minutes
                .iter()
                .filter(|c| c.name.to_lowercase().contains("productive"))
                .map(|c| c.minutes)
                .sum();
            let entertainment: u32 = s
                .usage
                .category_minutes
                .iter()
                .filter(|c| c.name.to_lowercase().contains("entertainment"))
                .map(|c| c.minutes)
                .sum();
            (
                s.usage.total_minutes.to_string(),
                productive.to_string(),
                entertainment.to_string(),
                s.pickups.total.to_string(),
                s.notifications.total.to_string(),
            )
        }
        None => (
            "?".to_string(),
            "?".to_string(),
            "?".to_string(),
            "?".to_string(),
            "?".to_string(),
        ),
    };

    let (sleep_hours, sleep_quality) = match sleep {
        Some(s) => (format!("{:.1}", s.duration_hours), s.quality_score.to_string()),
        None => ("?".to_string(), "?".to_string()),
    };

    let system = "你是 Canopus 观察系统。你的唯一职责是从数据中提炼原始观察。\n\
                  规则：\n\
                  - 只陈述事实，不做判断，不给建议\n\
                  - 每条观察必须有数据支撑\n\
                  - 最多输出3条观察\n\
                  - 每条观察一行，用「-」开头\n\
                  - 不要有任何其他输出";

    let user = format!(
        "日期：{date}\n\n\
         日记：{journal}\n\
         情绪：{mood}/10  精力：{energy}/10\n\n\
         任务：完成 {done} 跳过 {skipped} 未完成 {todo}\n\
         跳过的任务：{skipped_detail}\n\n\
         屏幕时间：总 {total}分钟\n\
           productive: {productive}分钟\n\
           entertainment: {entertainment}分钟\n\
         拾起手机：{pickups}次\n\
         通知：{notifications}次\n\n\
         睡眠：{hours}小时  质量：{quality}/5\n\n\
         请输出2–3条原始观察，只陈述事实。",
        date = date,
        journal = journal_content,
        mood = mood_str,
        energy = energy_str,
        done = done_str,
        skipped = skipped_str,
        todo = todo_str,
        skipped_detail = skipped_detail,
        total = total_min,
        productive = productive_min,
        entertainment = entertainment_min,
        pickups = pickups,
        notifications = notifications,
        hours = sleep_hours,
        quality = sleep_quality,
    );

    format!("<system>\n{}\n</system>\n{}", system, user)
}

// ── Ollama ────────────────────────────────────────────────────────────────────

fn call_ollama(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    eprintln!("{}", format!("Calling Ollama ({})...", OLLAMA_MODEL).dimmed());

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let body = json!({
        "model": OLLAMA_MODEL,
        "prompt": prompt,
        "stream": false
    });

    let response = client
        .post(OLLAMA_URL)
        .json(&body)
        .send()
        .map_err(|e| format!("Failed to connect to Ollama at {}: {}", OLLAMA_URL, e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned HTTP {}", response.status()).into());
    }

    let json: Value = response.json()?;
    let text = json["response"]
        .as_str()
        .ok_or("Ollama response missing 'response' field")?
        .to_string();

    Ok(text)
}
