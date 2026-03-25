use crate::commands::observe::load_observations;
use crate::commands::principles::create_candidate;
use crate::models::reflection::{CandidatePrinciple, Pattern, Reflection};
use crate::store::{get_canopus_dir, get_today_str, is_initialized};
use chrono::{Duration, Local};
use colored::Colorize;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const OLLAMA_MODEL: &str = "qwen2.5:7b";

fn get_reflections_dir() -> PathBuf {
    get_canopus_dir().join("reflections")
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if !is_initialized() {
        println!("{}", "Canopus not initialized. Run `canopus init` first.".red());
        return Ok(());
    }

    let today = Local::now().date_naive();
    let seven_days_ago = today - Duration::days(6);
    let period_start = seven_days_ago.format("%Y-%m-%d").to_string();
    let period_end = get_today_str();

    // Step 1: Load 7 days of observations
    let mut obs_by_date: Vec<(String, Vec<_>)> = Vec::new();
    for i in 0..7 {
        let date = seven_days_ago + Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        let obs = load_observations(&date_str);
        obs_by_date.push((date_str, obs));
    }

    let days_with_obs = obs_by_date.iter().filter(|(_, obs)| !obs.is_empty()).count();
    let total_obs: usize = obs_by_date.iter().map(|(_, obs)| obs.len()).sum();

    if days_with_obs < 3 {
        println!(
            "Not enough data yet. Need observations from at least 3 days.\n\
             You have {} day(s). Keep using canopus analyze daily.",
            days_with_obs
        );
        return Ok(());
    }

    // Step 2: Build prompt
    let mut obs_text = String::new();
    let mut all_obs_ids: Vec<String> = Vec::new();
    for (date, obs_list) in &obs_by_date {
        if obs_list.is_empty() {
            continue;
        }
        obs_text.push_str(&format!("{}:\n", date));
        for o in obs_list {
            obs_text.push_str(&format!("- {}: {}\n", o.id, o.content));
            all_obs_ids.push(o.id.clone());
        }
        obs_text.push('\n');
    }

    let system = "你是 Canopus 反思系统。从一段时间的观察中归纳行为模式，并生成候选原则。\n\
                  规则：\n\
                  - 只基于提供的观察数据\n\
                  - 模式必须出现至少2次才能归纳\n\
                  - 候选原则必须可执行，不能模糊\n\
                  - 不要给建议，只归纳规律\n\
                  - 严格按照指定JSON格式输出，不要有任何其他文字";

    let user = format!(
        "以下是过去7天的观察记录：\n\n\
         {obs_text}\n\
         请分析并以JSON格式输出：\n\
         {{\n\
           \"patterns\": [\n\
             {{\n\
               \"description\": \"模式描述\",\n\
               \"frequency\": 出现次数,\n\
               \"example_dates\": [\"2026-03-20\", \"2026-03-22\"]\n\
             }}\n\
           ],\n\
           \"candidate_principles\": [\n\
             {{\n\
               \"title\": \"原则标题（简短）\",\n\
               \"description\": \"可执行的原则描述\",\n\
               \"domain\": \"执行力|决策|注意力|情绪|人际关系\",\n\
               \"supporting_pattern\": \"对应上面哪个模式\"\n\
             }}\n\
           ]\n\
         }}",
        obs_text = obs_text
    );

    let prompt = format!("<system>\n{}\n</system>\n{}", system, user);
    let raw = call_ollama(&prompt)?;

    // Step 3: Parse JSON
    let cleaned = strip_code_fences(&raw);

    #[derive(Deserialize)]
    struct LlmResponse {
        patterns: Vec<Pattern>,
        candidate_principles: Vec<CandidatePrinciple>,
    }

    let parsed = match serde_json::from_str::<LlmResponse>(&cleaned) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", format!("Failed to parse model response: {}", e).red());
            println!("{}", "Raw output:".yellow());
            println!("{}", raw);
            let raw_path = get_reflections_dir().join(format!("{}_raw.txt", period_end));
            let _ = fs::write(&raw_path, &raw);
            println!(
                "{}",
                format!("Raw output saved to {}", raw_path.display()).dimmed()
            );
            return Ok(());
        }
    };

    // Step 4: Display
    sep();
    println!(
        "{}",
        format!("REFLECTION  {} → {}", period_start, period_end).bold()
    );
    sep();
    println!("Observations analyzed: {}", total_obs);
    println!();

    println!("{}", "PATTERNS FOUND".bold());
    for (i, p) in parsed.patterns.iter().enumerate() {
        println!("{} {} ({}次)", circled(i + 1), p.description, p.frequency);
        if !p.example_dates.is_empty() {
            println!("  {} {}", "→".dimmed(), p.example_dates.join(", ").dimmed());
        }
    }
    println!();

    println!("{}", "CANDIDATE PRINCIPLES".bold());
    for (i, cp) in parsed.candidate_principles.iter().enumerate() {
        let domain = cp.domain.as_deref().unwrap_or("?");
        println!("{} [{}] {}", circled(i + 1), domain.cyan(), cp.title.bold());
        println!("  {}", cp.description);
        println!();
    }

    // Step 5: Confirm and create principles
    print!("{} [y/N]: ", "Add to principles library?");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();

    if input.trim().eq_ignore_ascii_case("y") {
        for cp in &parsed.candidate_principles {
            let num = create_candidate(
                cp.title.clone(),
                cp.description.clone(),
                cp.domain.clone(),
                format!(
                    "Identified via 7-day reflection: {}",
                    cp.supporting_pattern
                ),
                period_end.clone(),
            )?;
            println!("{} principle_{}: {}", "✓".green(), num, cp.title);
        }
    }

    // Step 6: Save reflection
    let reflection = Reflection {
        id: format!("reflect_{}", period_end.replace('-', "")),
        date: period_end.clone(),
        period_start,
        period_end: period_end.clone(),
        observations_used: all_obs_ids,
        patterns: parsed.patterns,
        candidate_principles: parsed.candidate_principles,
        created_at: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
    };

    let refl_path = get_reflections_dir().join(format!("{}.json", period_end));
    fs::write(&refl_path, serde_json::to_string_pretty(&reflection)?)?;
    println!("{}", "Reflection saved.".dimmed());

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn sep() {
    println!("{}", "━".repeat(38));
}

fn circled(n: usize) -> &'static str {
    match n {
        1 => "①",
        2 => "②",
        3 => "③",
        4 => "④",
        5 => "⑤",
        _ => "•",
    }
}

fn strip_code_fences(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("```json").unwrap_or(s);
    let s = s.strip_prefix("```").unwrap_or(s);
    let s = s.strip_suffix("```").unwrap_or(s);
    s.trim().to_string()
}

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
