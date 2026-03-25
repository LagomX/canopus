use crate::models::attention::ScreenTimeRecord;
use crate::models::journal::JournalEntry;
use crate::models::sleep::SleepRecord;
use crate::models::task::{Priority, Task, TaskStatus};
use crate::store::{get_canopus_dir, get_data_dir, get_today_str, is_initialized, read_json};
use colored::Colorize;
use serde_json::{json, Value};

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const OLLAMA_MODEL: &str = "qwen2.5:7b";

/// Loads today's (or a specific date's) data, computes a contradiction score,
/// and calls the local Ollama API to generate a cognitive adversary analysis.
pub fn run(brutal: bool, date: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let date_str = date.unwrap_or_else(get_today_str);
    let data = get_data_dir();

    // ── Step 1: Load data ────────────────────────────────────────────────────
    let journal: Option<JournalEntry> =
        read_json(&data.join("journal").join(format!("{}.json", date_str)));
    let tasks: Option<Vec<Task>> =
        read_json(&data.join("tasks").join(format!("{}.json", date_str)));
    let sleep: Option<SleepRecord> =
        read_json(&data.join("sleep").join(format!("{}.json", date_str)));
    let screen: Option<ScreenTimeRecord> =
        read_json(&data.join("attention").join(format!("{}.json", date_str)));

    if journal.is_none() {
        eprintln!("{}", "Warning: no journal entry found.".yellow());
    }
    if tasks.is_none() {
        eprintln!("{}", "Warning: no tasks found.".yellow());
    }
    if sleep.is_none() {
        eprintln!("{}", "Warning: no sleep record found.".yellow());
    }
    if screen.is_none() {
        eprintln!("{}", "Warning: no attention record found.".yellow());
    }

    if journal.is_none() && tasks.is_none() && sleep.is_none() && screen.is_none() {
        return Err(format!(
            "No data found for {}. Record some data first.",
            date_str
        )
        .into());
    }

    // ── Step 2: Compute contradiction_score ──────────────────────────────────
    let task_gap = compute_task_gap(tasks.as_deref());
    let attention_gap = compute_attention_gap(screen.as_ref());
    let sleep_penalty = compute_sleep_penalty(sleep.as_ref());
    let contradiction_score = task_gap * 0.40 + attention_gap * 0.35 + sleep_penalty * 0.25;

    // ── Step 3: Determine intensity level ────────────────────────────────────
    let intensity: u8 = if brutal || contradiction_score >= 0.7 {
        3
    } else if contradiction_score >= 0.4 {
        2
    } else {
        1
    };

    // ── Step 4: Build prompt and call Ollama ─────────────────────────────────
    let profile = load_profile();
    let system_prompt = system_for_level(intensity, profile.as_deref());
    let user_prompt = build_user_prompt(
        &date_str,
        &journal,
        tasks.as_deref(),
        &sleep,
        &screen,
        contradiction_score,
        intensity,
    );
    let full_prompt = format!("<system>\n{}\n</system>\n{}", &system_prompt, user_prompt);

    // ── Step 5: Output ───────────────────────────────────────────────────────
    println!(
        "{}  {}",
        format!("矛盾分: {:.2}", contradiction_score).yellow().bold(),
        format!("Level {}", intensity).yellow()
    );
    println!("{}", "─".repeat(52));

    let analysis = call_ollama(&full_prompt)?;
    println!("{}", analysis.white());

    // ── Step 6: Auto-generate observations ───────────────────────────────────
    println!("\n{}", "── Auto-generating observations...".dimmed());
    if let Err(e) = crate::commands::observe::run_auto(&date_str) {
        eprintln!(
            "{}",
            format!("Warning: could not auto-generate observations: {}", e).yellow()
        );
    }

    Ok(())
}

// ── Score component helpers ──────────────────────────────────────────────────

/// Ratio of skipped/todo high-priority tasks to all high-priority tasks.
fn compute_task_gap(tasks: Option<&[Task]>) -> f64 {
    let tasks = match tasks {
        Some(t) => t,
        None => return 0.5,
    };
    let total_high = tasks
        .iter()
        .filter(|t| t.priority == Priority::High)
        .count();
    if total_high == 0 {
        return 0.0;
    }
    let unfinished_high = tasks
        .iter()
        .filter(|t| {
            t.priority == Priority::High
                && matches!(t.status, TaskStatus::Skipped | TaskStatus::Todo)
        })
        .count();
    unfinished_high as f64 / total_high as f64
}

/// 1 - (productive_minutes / total_minutes). Falls back to 0.5 with no data.
fn compute_attention_gap(screen: Option<&ScreenTimeRecord>) -> f64 {
    let screen = match screen {
        Some(s) => s,
        None => return 0.5,
    };
    let total = screen.usage.total_minutes;
    if total == 0 {
        return 0.5;
    }
    let productive: u32 = screen
        .usage
        .category_minutes
        .iter()
        .filter(|c| c.name.to_lowercase().contains("productive"))
        .map(|c| c.minutes)
        .sum();
    1.0 - (productive as f64 / total as f64)
}

/// Converts sleep duration into a penalty score. Falls back to 0.5 with no data.
fn compute_sleep_penalty(sleep: Option<&SleepRecord>) -> f64 {
    match sleep {
        None => 0.5,
        Some(s) => {
            if s.duration_hours < 5.0 {
                0.8
            } else if s.duration_hours <= 6.0 {
                0.5
            } else {
                0.2
            }
        }
    }
}

// ── Prompt builders ──────────────────────────────────────────────────────────

fn load_profile() -> Option<String> {
    let path = get_canopus_dir().join("data").join("context.json");
    let content = std::fs::read_to_string(&path).ok()?;
    let v: Value = serde_json::from_str(&content).ok()?;

    let profile = &v["profile"];
    let name = profile["preferred_name"].as_str().unwrap_or("用户");
    let location = profile["current_location"].as_str().unwrap_or("");
    let education = profile["education"].as_str().unwrap_or("");
    let background = profile["background"].as_str().unwrap_or("");
    let identity: Vec<&str> = profile["current_identity"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
        .unwrap_or_default();

    // Extract key projects/goals from experiences
    let experiences = v["experiences"].as_array();
    let goals: Vec<String> = experiences
        .map(|exps| {
            exps.iter()
                .filter_map(|e| {
                    let t = e["type"].as_str()?;
                    let name = e["name"].as_str()?;
                    let role = e["role"].as_str().unwrap_or("");
                    match t {
                        "project" | "study" => Some(format!("{}: {}", t, name)),
                        "work" if !role.is_empty() => Some(format!("工作: {} ({})", name, role)),
                        _ => None,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let mut parts = vec![format!("用户姓名: {}", name)];
    if !location.is_empty() { parts.push(format!("所在地: {}", location)); }
    if !education.is_empty() { parts.push(format!("教育: {}", education)); }
    if !background.is_empty() { parts.push(format!("背景: {}", background)); }
    if !identity.is_empty() { parts.push(format!("当前身份: {}", identity.join("、"))); }
    if !goals.is_empty() { parts.push(format!("当前项目/学习: {}", goals.join("；"))); }

    Some(parts.join("\n"))
}

fn system_for_level(level: u8, profile: Option<&str>) -> String {
    let profile_block = match profile {
        Some(p) => format!("\n\n【用户背景】\n{}", p),
        None => String::new(),
    };

    let base = match level {
        1 => format!(
            "你是 Canopus，一个认知观察系统。用中性、描述性的语气分析用户今天的行为模式。{}",
            profile_block
        ),
        3 => format!(
            "你是 Canopus，一个认知对抗系统。用冷静但锋利的语气拆穿用户的自我叙述。\
             找出回避机制，重构真实原因，不要软化结论。\
             结合用户的背景和目标来判断行为的真实代价。{}",
            profile_block
        ),
        _ => format!(
            "你是 Canopus，一个认知对抗系统。你的职责是找出用户行为和叙述之间的矛盾。\
             直接指出矛盾，不要安慰，不要验证用户的叙述，除非有数据支撑。\
             结合用户的背景和目标来评估今天的行为是否与其声称的优先级一致。{}",
            profile_block
        ),
    };
    base
}

fn build_user_prompt(
    date: &str,
    journal: &Option<JournalEntry>,
    tasks: Option<&[Task]>,
    sleep: &Option<SleepRecord>,
    screen: &Option<ScreenTimeRecord>,
    score: f64,
    intensity: u8,
) -> String {
    // Journal
    let (journal_content, mood_str, energy_str) = match journal {
        Some(j) => (
            j.content.clone(),
            j.mood_score
                .map(|m| m.to_string())
                .unwrap_or_else(|| "?".to_string()),
            j.energy_score
                .map(|e| e.to_string())
                .unwrap_or_else(|| "?".to_string()),
        ),
        None => ("(无日记)".to_string(), "?".to_string(), "?".to_string()),
    };

    // Tasks
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

    // Attention
    let (total_min, productive_min, productive_ratio, entertainment_min, top_apps_str) =
        match screen {
            Some(s) => {
                let total = s.usage.total_minutes;
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
                let ratio = if total > 0 { productive * 100 / total } else { 0 };
                let apps: Vec<String> = s
                    .usage
                    .top_apps
                    .iter()
                    .take(5)
                    .map(|a| format!("{}({}min)", a.name, a.minutes))
                    .collect();
                (
                    total.to_string(),
                    productive.to_string(),
                    ratio.to_string(),
                    entertainment.to_string(),
                    if apps.is_empty() {
                        "(无)".to_string()
                    } else {
                        apps.join("、")
                    },
                )
            }
            None => (
                "?".to_string(),
                "?".to_string(),
                "?".to_string(),
                "?".to_string(),
                "(无注意力数据)".to_string(),
            ),
        };

    // Sleep
    let (sleep_hours_str, sleep_quality_str) = match sleep {
        Some(s) => (format!("{:.1}", s.duration_hours), s.quality_score.to_string()),
        None => ("?".to_string(), "?".to_string()),
    };

    format!(
        "今日数据 ({date}):\n\
         \n\
         【日记】\n\
         {journal_content}\n\
         情绪: {mood}/10  精力: {energy}/10\n\
         \n\
         【任务】\n\
         完成: {done}  跳过: {skipped}  未完成: {todo}\n\
         跳过的任务: {skipped_detail}\n\
         \n\
         【注意力】\n\
         总屏幕时间: {total_min}分钟\n\
         生产力类: {productive_min}分钟 ({productive_ratio}%)\n\
         娱乐类: {entertainment_min}分钟\n\
         Top Apps: {top_apps}\n\
         \n\
         【睡眠】\n\
         时长: {sleep_hours}小时  质量: {quality}/5\n\
         \n\
         【矛盾分】{score:.2} (Level {intensity})\n\
         \n\
         请按以下结构输出分析：\n\
         ① 对比：你声称 vs 数据显示\n\
         ② 重构：真实原因推断\n\
         ③ 模式：行为规律归纳\n\
         ④ 根因：深层机制\n",
        date = date,
        journal_content = journal_content,
        mood = mood_str,
        energy = energy_str,
        done = done_str,
        skipped = skipped_str,
        todo = todo_str,
        skipped_detail = skipped_detail,
        total_min = total_min,
        productive_min = productive_min,
        productive_ratio = productive_ratio,
        entertainment_min = entertainment_min,
        top_apps = top_apps_str,
        sleep_hours = sleep_hours_str,
        quality = sleep_quality_str,
        score = score,
        intensity = intensity,
    )
}

// ── Ollama API call ──────────────────────────────────────────────────────────

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
