use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use canopus::models::attention::ScreenTimeRecord;
use canopus::models::journal::JournalEntry;
use canopus::models::sleep::SleepRecord;
use canopus::models::task::{calc_exec_index, Quadrant, Task, TaskStatus};
use canopus::store::get_canopus_dir;
use canopus::store::get_data_dir;
use chrono::{Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
// ── Journal file helpers ──────────────────────────────────────────────────────

fn read_journal_file(path: &std::path::Path) -> Vec<JournalEntry> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    if let Ok(entries) = serde_json::from_str::<Vec<JournalEntry>>(&content) {
        return entries;
    }
    if let Ok(entry) = serde_json::from_str::<JournalEntry>(&content) {
        return vec![entry];
    }
    vec![]
}

// ── Query / body types ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct JournalQuery {
    days: Option<i64>,
}

#[derive(Deserialize)]
pub struct PostBody {
    content: String,
    mood: Option<String>,
    tags: Option<Vec<String>>,
}

// ── GET /api/journal?days=7 ───────────────────────────────────────────────────

pub async fn get_journal(Query(params): Query<JournalQuery>) -> Json<Vec<JournalEntry>> {
    let days = params.days.unwrap_or(7);
    let data_dir = get_data_dir();
    let today = Local::now().date_naive();
    let mut entries = Vec::new();

    for i in 0..days {
        let date = today - Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        let path = data_dir
            .join("journal")
            .join(format!("{}.json", date_str));

        entries.extend(read_journal_file(&path));
    }

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Json(entries)
}

// ── POST /api/journal ─────────────────────────────────────────────────────────

pub async fn post_journal(Json(body): Json<PostBody>) -> impl IntoResponse {
    if body.content.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "content is required"})),
        )
            .into_response();
    }

    let now_utc = Utc::now();
    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let id = format!("journal_{}", now_utc.format("%Y%m%d_%H%M%S"));
    let timestamp = now_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let entry = JournalEntry {
        id,
        timestamp,
        date: date_str.clone(),
        content: body.content.trim().to_string(),
        mood: body.mood,
        tags: body.tags.unwrap_or_default(),
    };

    let data_dir = get_data_dir();
    let journal_dir = data_dir.join("journal");

    if let Err(e) = fs::create_dir_all(&journal_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    let path = journal_dir.join(format!("{}.json", date_str));

    let mut entries = read_journal_file(&path);

    entries.push(entry.clone());

    match serde_json::to_string_pretty(&entries) {
        Ok(serialized) => match fs::write(&path, serialized) {
            Ok(_) => Json(entry).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── DELETE /api/journal/:id ───────────────────────────────────────────────────

pub async fn delete_journal(Path(id): Path<String>) -> impl IntoResponse {
    // Parse date from "journal_YYYYMMDD_HHMMSS"
    let parts: Vec<&str> = id.splitn(3, '_').collect();
    if parts.len() < 3 || parts[0] != "journal" {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "invalid id format"})),
        )
            .into_response();
    }

    let date_compact = parts[1];
    if date_compact.len() != 8 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "invalid id format"})),
        )
            .into_response();
    }

    let date_str = format!(
        "{}-{}-{}",
        &date_compact[..4],
        &date_compact[4..6],
        &date_compact[6..8]
    );

    let path = get_data_dir()
        .join("journal")
        .join(format!("{}.json", date_str));

    if !path.exists() {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response();
    }

    let mut entries = read_journal_file(&path);
    if entries.is_empty() {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response();
    }

    let before = entries.len();
    entries.retain(|e| e.id != id);

    if entries.len() == before {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "id not found"})),
        )
            .into_response();
    }

    if entries.is_empty() {
        if let Err(e) = fs::remove_file(&path) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to delete file: {}", e)})),
            ).into_response();
        }
    } else {
        match serde_json::to_string_pretty(&entries) {
            Ok(s) => {
                if let Err(e) = fs::write(&path, s) {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": format!("Failed to write file: {}", e)})),
                    ).into_response();
                }
            }
            Err(e) => return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response(),
        }
    }

    Json(json!({"deleted": true})).into_response()
}

// ── Sleep types ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SleepQuery {
    days: Option<i64>,
}

#[derive(Deserialize)]
pub struct SleepPostBody {
    bedtime: String,
    wake_time: String,
    date: Option<String>,
    quality_score: Option<u8>,
}

#[derive(Serialize)]
pub struct SleepSummary {
    date: String,
    bedtime: Option<String>,
    wake_time: Option<String>,
    duration_hours: f64,
}

// ── GET /api/sleep?days=7 ─────────────────────────────────────────────────────

pub async fn get_sleep(Query(params): Query<SleepQuery>) -> Json<Vec<Option<SleepSummary>>> {
    let days = params.days.unwrap_or(7);
    let data_dir = get_data_dir();
    let today = Local::now().date_naive();
    let mut result = Vec::new();

    for i in 0..days {
        let date = today - Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        let path = data_dir
            .join("sleep")
            .join(format!("{}.json", date_str));

        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(rec) = serde_json::from_str::<SleepRecord>(&content) {
                result.push(Some(SleepSummary {
                    date: rec.date,
                    bedtime: rec.bedtime,
                    wake_time: rec.wake_time,
                    duration_hours: rec.duration_hours,
                }));
                continue;
            }
        }
        result.push(None);
    }

    Json(result)
}

// ── POST /api/sleep ───────────────────────────────────────────────────────────

fn parse_hhmm(s: &str) -> Option<()> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 { return None; }
    let h: u32 = parts[0].parse().ok()?;
    let m: u32 = parts[1].parse().ok()?;
    if h > 23 || m > 59 { return None; }
    Some(())
}

pub async fn post_sleep(Json(body): Json<SleepPostBody>) -> impl IntoResponse {
    if body.bedtime.is_empty() || body.wake_time.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "all fields required"})),
        )
            .into_response();
    }

    if parse_hhmm(&body.bedtime).is_none() || parse_hhmm(&body.wake_time).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid time format. Use HH:MM (00:00\u{2013}23:59)"})),
        )
            .into_response();
    }

    let duration = compute_duration(&body.bedtime, &body.wake_time);
    let date_str = body
        .date
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let date_compact = date_str.replace('-', "");

    let quality = body.quality_score.unwrap_or(0).min(5);

    let record = SleepRecord {
        id: format!("sleep_{}", date_compact),
        date: date_str.clone(),
        duration_hours: duration,
        quality_score: quality,
        bedtime: Some(body.bedtime),
        wake_time: Some(body.wake_time),
        notes: None,
    };

    let data_dir = get_data_dir();
    let sleep_dir = data_dir.join("sleep");

    if let Err(e) = fs::create_dir_all(&sleep_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    let path = sleep_dir.join(format!("{}.json", date_str));

    match serde_json::to_string_pretty(&record) {
        Ok(serialized) => match fs::write(&path, serialized) {
            Ok(_) => Json(record).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

fn compute_duration(bedtime: &str, wake_time: &str) -> f64 {
    fn parse_mins(t: &str) -> i32 {
        let mut parts = t.splitn(2, ':');
        let h = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        let m = parts.next().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
        h * 60 + m
    }
    let bed  = parse_mins(bedtime);
    let wake = parse_mins(wake_time);
    let diff = if wake >= bed { wake - bed } else { wake + 1440 - bed };
    (diff as f64 / 60.0 * 10.0).round() / 10.0
}

// ── Task helpers ──────────────────────────────────────────────────────────────

fn tasks_path(date_str: &str) -> std::path::PathBuf {
    get_data_dir()
        .join("tasks")
        .join(format!("{}.json", date_str))
}

fn read_tasks(date_str: &str) -> Vec<Task> {
    let path = tasks_path(date_str);
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    serde_json::from_str::<Vec<Task>>(&content).unwrap_or_default()
}

fn write_tasks(date_str: &str, tasks: &[Task]) -> std::io::Result<()> {
    let dir = get_data_dir().join("tasks");
    fs::create_dir_all(&dir)?;
    let path = tasks_path(date_str);
    let s = serde_json::to_string_pretty(tasks).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;
    fs::write(path, s)
}

// ── Task query / body types ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TaskQuery {
    date: Option<String>,
}

#[derive(Deserialize)]
pub struct TaskPostBody {
    title: String,
    quadrant: Option<String>,
    domain: Option<String>,
}

#[derive(Deserialize)]
pub struct TaskPatchBody {
    status: Option<String>,
    skip_reason: Option<String>,
    notes: Option<String>,
    quadrant: Option<String>,
}

#[derive(Serialize)]
pub struct TasksResponse {
    tasks: Vec<Task>,
    exec_index: f64,
}

// ── GET /api/tasks?date=YYYY-MM-DD ────────────────────────────────────────────

pub async fn get_tasks(Query(params): Query<TaskQuery>) -> Json<TasksResponse> {
    let date_str = params
        .date
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let tasks = read_tasks(&date_str);
    let exec_index = calc_exec_index(&tasks);
    Json(TasksResponse { tasks, exec_index })
}

// ── POST /api/tasks ───────────────────────────────────────────────────────────

pub async fn post_task(Json(body): Json<TaskPostBody>) -> impl IntoResponse {
    if body.title.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "title is required"})),
        )
            .into_response();
    }

    let quadrant = match body.quadrant.as_deref() {
        Some(q) => match Quadrant::from_str(q) {
            Some(qv) => qv,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "invalid quadrant, use q1/q2/q3/q4"})),
                )
                    .into_response();
            }
        },
        None => Quadrant::Q2,
    };

    let date_str = Local::now().format("%Y-%m-%d").to_string();
    let date_compact = date_str.replace('-', "");
    let mut tasks = read_tasks(&date_str);
    let index = tasks.len() + 1;

    let task = Task {
        id: format!("task_{}_{:03}", date_compact, index),
        date: date_str.clone(),
        title: body.title.trim().to_string(),
        status: TaskStatus::Todo,
        quadrant,
        domain: body.domain,
        skip_reason: None,
        notes: None,
    };

    tasks.push(task.clone());

    match write_tasks(&date_str, &tasks) {
        Ok(_) => Json(task).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── PATCH /api/tasks/:id ──────────────────────────────────────────────────────

pub async fn patch_task(
    Path(id): Path<String>,
    Json(body): Json<TaskPatchBody>,
) -> impl IntoResponse {
    // Derive date from id: "task_YYYYMMDD_NNN"
    let parts: Vec<&str> = id.splitn(3, '_').collect();
    if parts.len() < 3 || parts[0] != "task" {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "invalid id"})),
        )
            .into_response();
    }
    let date_compact = parts[1];
    if date_compact.len() != 8 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "invalid id format"})),
        )
            .into_response();
    }
    let date_str = format!(
        "{}-{}-{}",
        &date_compact[..4],
        &date_compact[4..6],
        &date_compact[6..8]
    );

    let mut tasks = read_tasks(&date_str);
    let pos = tasks.iter().position(|t| t.id == id);
    let idx = match pos {
        Some(i) => i,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "task not found"})),
            )
                .into_response();
        }
    };

    if let Some(s) = body.status {
        tasks[idx].status = match s.as_str() {
            "done"         => TaskStatus::Done,
            "skipped"      => TaskStatus::Skipped,
            "partial"      => TaskStatus::Partial,
            "carried_over" => TaskStatus::CarriedOver,
            _              => TaskStatus::Todo,
        };

        // If a task leaves the "skipped" state, clear the skip reason to keep UI & data consistent.
        if tasks[idx].status != TaskStatus::Skipped {
            tasks[idx].skip_reason = None;
        }
    }
    if let Some(r) = body.skip_reason {
        tasks[idx].skip_reason = if r.is_empty() { None } else { Some(r) };
    }
    if let Some(n) = body.notes {
        tasks[idx].notes = if n.is_empty() { None } else { Some(n) };
    }
    if let Some(q) = body.quadrant {
        if let Some(qv) = Quadrant::from_str(&q) {
            tasks[idx].quadrant = qv;
        }
    }

    let updated = tasks[idx].clone();
    match write_tasks(&date_str, &tasks) {
        Ok(_) => Json(updated).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── DELETE /api/tasks/:id ─────────────────────────────────────────────────────

pub async fn delete_task(Path(id): Path<String>) -> impl IntoResponse {
    let parts: Vec<&str> = id.splitn(3, '_').collect();
    if parts.len() < 3 || parts[0] != "task" {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "invalid id"})),
        )
            .into_response();
    }
    let date_compact = parts[1];
    if date_compact.len() != 8 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "invalid id format"})),
        )
            .into_response();
    }
    let date_str = format!(
        "{}-{}-{}",
        &date_compact[..4],
        &date_compact[4..6],
        &date_compact[6..8]
    );

    let mut tasks = read_tasks(&date_str);
    let before = tasks.len();
    tasks.retain(|t| t.id != id);

    if tasks.len() == before {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "task not found"})),
        )
            .into_response();
    }

    match write_tasks(&date_str, &tasks) {
        Ok(_) => Json(json!({"deleted": true})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── Report types ──────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Report {
    pub id: String,
    pub generated_at: String,
    #[serde(rename = "type")]
    pub report_type: String,
    pub date: String,
    pub period_start: String,
    pub period_end: String,
    pub contradiction_score: f64,
    pub intensity_level: u8,
    pub data_summary: ReportDataSummary,
    pub analysis: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ReportDataSummary {
    pub journal_entries: usize,
    pub tasks_total: usize,
    pub tasks_done: usize,
    pub tasks_skipped: usize,
    pub sleep_hours: f64,
    pub screen_total_minutes: u32,
    pub screen_productive_minutes: u32,
    pub pickups: u32,
    pub notifications: u32,
}

#[derive(serde::Serialize)]
pub struct ReportListItem {
    pub id: String,
    pub generated_at: String,
    #[serde(rename = "type")]
    pub report_type: String,
    pub date: String,
    pub contradiction_score: f64,
    pub intensity_level: u8,
}

#[derive(serde::Deserialize)]
pub struct GenerateBody {
    #[serde(rename = "type")]
    pub report_type: String,
    pub date: String,
    pub brutal: Option<bool>,
}

struct DayData {
    date: String,
    journals: Vec<JournalEntry>,
    tasks: Vec<Task>,
    sleep: Option<SleepRecord>,
    screen: Option<ScreenTimeRecord>,
}

struct PreparedReport {
    report_type: String,
    date: String,
    period_start: String,
    period_end: String,
    contradiction_score: f64,
    intensity_level: u8,
    summary: ReportDataSummary,
    full_prompt: String,
    report_id: String,
    file_name: String,
    days_with_data: usize,
}

// ── Report file helpers ───────────────────────────────────────────────────────

fn reports_dir() -> std::path::PathBuf {
    get_canopus_dir().join("reports")
}

fn id_to_filename(id: &str) -> Option<String> {
    let without_prefix = id.strip_prefix("report_")?;
    let (date_compact, suffix) = if without_prefix.ends_with("_7d") {
        (&without_prefix[..8], "_7d")
    } else if without_prefix.len() == 8 {
        (without_prefix, "")
    } else {
        return None;
    };
    if date_compact.len() != 8 { return None; }
    let date = format!("{}-{}-{}", &date_compact[..4], &date_compact[4..6], &date_compact[6..8]);
    Some(format!("{}{}.json", date, suffix))
}

fn load_day_data_for_report(date_str: &str) -> (Option<Vec<JournalEntry>>, Option<Vec<Task>>, Option<SleepRecord>, Option<ScreenTimeRecord>) {
    let data_dir = get_data_dir();

    let journals: Option<Vec<JournalEntry>> = {
        let p = data_dir.join("journal").join(format!("{}.json", date_str));
        fs::read_to_string(&p).ok().and_then(|c| {
            serde_json::from_str::<Vec<JournalEntry>>(&c)
                .ok()
                .or_else(|| serde_json::from_str::<JournalEntry>(&c).ok().map(|e| vec![e]))
        })
    };

    let tasks: Option<Vec<Task>> = {
        let p = data_dir.join("tasks").join(format!("{}.json", date_str));
        fs::read_to_string(&p).ok().and_then(|c| serde_json::from_str(&c).ok())
    };

    let sleep: Option<SleepRecord> = {
        let p = data_dir.join("sleep").join(format!("{}.json", date_str));
        fs::read_to_string(&p).ok().and_then(|c| serde_json::from_str(&c).ok())
    };

    let screen: Option<ScreenTimeRecord> = {
        let p = data_dir.join("attention").join(format!("{}.json", date_str));
        fs::read_to_string(&p).ok().and_then(|c| serde_json::from_str(&c).ok())
    };

    (journals, tasks, sleep, screen)
}

fn report_task_gap(tasks: &[Task]) -> f64 {
    let q1_total = tasks.iter().filter(|t| t.quadrant == Quadrant::Q1).count();
    if q1_total == 0 { return 0.0; }
    let q1_unfinished = tasks.iter().filter(|t| {
        t.quadrant == Quadrant::Q1 && matches!(t.status, TaskStatus::Skipped | TaskStatus::Todo)
    }).count();
    q1_unfinished as f64 / q1_total as f64
}

fn report_attention_gap(screen: Option<&ScreenTimeRecord>) -> f64 {
    let screen = match screen { Some(s) => s, None => return 0.5 };
    let total = screen.usage.total_minutes;
    if total == 0 { return 0.5; }
    let productive: u32 = screen.usage.category_minutes.iter()
        .filter(|c| c.name.to_lowercase().contains("productive"))
        .map(|c| c.minutes).sum();
    1.0 - (productive as f64 / total as f64)
}

fn report_sleep_penalty(sleep: Option<&SleepRecord>) -> f64 {
    match sleep {
        None => 0.5,
        Some(s) => if s.duration_hours < 5.0 { 0.8 } else if s.duration_hours <= 6.0 { 0.5 } else { 0.2 }
    }
}

fn report_system_for_level(level: u8) -> &'static str {
    match level {
        1 => "你是 Canopus，一个认知观察系统。用中性、描述性的语气描述行为模式。只陈述观察到的事实和模式。不要给建议。不要说「建议你」。",
        3 => "你是 Canopus，一个认知对抗系统。用冷静但锋利的语气拆穿自我叙述。不要给建议。不要说「建议你」或「你应该」。不要在结尾总结。直接停在根因。",
        _ => "你是 Canopus，一个认知对抗系统。找出行为和叙述之间的矛盾，直接指出。不要安慰。不要给建议。不要说「建议你」或「你应该」。不要在结尾总结。只陈述矛盾、模式和推断。",
    }
}

fn build_single_user_prompt(
    date: &str,
    journals: &[JournalEntry],
    tasks: &[Task],
    sleep: Option<&SleepRecord>,
    screen: Option<&ScreenTimeRecord>,
    score: f64,
    level: u8,
) -> String {
    let journal_content = if journals.is_empty() {
        "无记录".to_string()
    } else {
        journals.iter().map(|j| j.content.as_str()).collect::<Vec<_>>().join("\n---\n")
    };
    let mood = journals.first().and_then(|j| j.mood.as_deref()).unwrap_or("未记录");

    let done = tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
    let skipped = tasks.iter().filter(|t| t.status == TaskStatus::Skipped).count();
    let todo = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Todo)).count();

    let task_list: String = tasks.iter().map(|t| {
        format!("  [{:?}] {} ({})", t.status, t.title, t.quadrant.label())
    }).collect::<Vec<_>>().join("\n");

    let skip_reasons: String = {
        let reasons: Vec<String> = tasks.iter()
            .filter(|t| t.status == TaskStatus::Skipped)
            .filter_map(|t| t.skip_reason.as_ref().map(|r| format!("「{}」: {}", t.title, r)))
            .collect();
        if reasons.is_empty() { "无".to_string() } else { reasons.join("、") }
    };

    let (total_min, prod_min, ratio, ent_min, pickups, notifs) = match screen {
        None => ("?".to_string(), "?".to_string(), "?".to_string(), "?".to_string(), "?".to_string(), "?".to_string()),
        Some(s) => {
            let total = s.usage.total_minutes;
            let prod: u32 = s.usage.category_minutes.iter()
                .filter(|c| c.name.to_lowercase().contains("productive")).map(|c| c.minutes).sum();
            let ent: u32 = s.usage.category_minutes.iter()
                .filter(|c| c.name.to_lowercase().contains("entertainment")).map(|c| c.minutes).sum();
            let r = if total > 0 { prod * 100 / total } else { 0 };
            (total.to_string(), prod.to_string(), r.to_string(), ent.to_string(),
             s.pickups.total.to_string(), s.notifications.total.to_string())
        }
    };

    let sleep_hours = sleep.map(|s| format!("{:.1}", s.duration_hours)).unwrap_or("?".to_string());

    format!(
        "日期：{date}\n\n\
         【日记】\n{journal_content}\n情绪：{mood}\n\n\
         【任务】\n完成：{done}  跳过：{skipped}  未完成：{todo}\n{task_list}\n跳过原因：{skip_reasons}\n\n\
         【屏幕时间】\n总计：{total_min}分钟\n生产力：{prod_min}分钟（{ratio}%）\n娱乐：{ent_min}分钟\n\
         拾起手机：{pickups}次\n通知：{notifs}次\n\n\
         【睡眠】\n时长：{sleep_hours}小时\n\n\
         【矛盾分】{score:.2}（Level {level}）\n\n\
         请按以下结构输出：\n① 对比：声称 vs 数据显示\n② 重构：真实原因推断\n③ 模式：行为规律归纳\n④ 根因：深层机制\n",
        date=date, journal_content=journal_content, mood=mood,
        done=done, skipped=skipped, todo=todo, task_list=task_list, skip_reasons=skip_reasons,
        total_min=total_min, prod_min=prod_min, ratio=ratio, ent_min=ent_min,
        pickups=pickups, notifs=notifs, sleep_hours=sleep_hours,
        score=score, level=level
    )
}

fn build_weekly_user_prompt(
    period_start: &str,
    period_end: &str,
    days: &[DayData],
    days_with_data: usize,
    avg_score: f64,
    level: u8,
) -> String {
    let mut day_blocks = String::new();
    for d in days {
        let has_data = !d.journals.is_empty() || !d.tasks.is_empty() || d.sleep.is_some() || d.screen.is_some();
        if !has_data { continue; }

        let journal_text = if d.journals.is_empty() {
            "无".to_string()
        } else {
            d.journals.iter().map(|j| j.content.as_str()).collect::<Vec<_>>().join(" / ")
        };

        let done = d.tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
        let skipped = d.tasks.iter().filter(|t| t.status == TaskStatus::Skipped).count();
        let todo = d.tasks.iter().filter(|t| matches!(t.status, TaskStatus::Todo)).count();

        let sleep_str = d.sleep.as_ref().map(|s| format!("{:.1}", s.duration_hours)).unwrap_or("无".to_string());

        let (total_min, ratio) = d.screen.as_ref().map(|s| {
            let total = s.usage.total_minutes;
            let prod: u32 = s.usage.category_minutes.iter()
                .filter(|c| c.name.to_lowercase().contains("productive")).map(|c| c.minutes).sum();
            let r = if total > 0 { prod * 100 / total } else { 0 };
            (total.to_string(), r.to_string())
        }).unwrap_or(("无".to_string(), "0".to_string()));

        day_blocks.push_str(&format!(
            "--- {} ---\n日记：{}\n任务：完成{} 跳过{} 未完成{}\n睡眠：{}小时\n屏幕时间：{}分钟（生产力{}%）\n\n",
            d.date, journal_text, done, skipped, todo, sleep_str, total_min, ratio
        ));
    }

    format!(
        "分析周期：{period_start} 至 {period_end}\n\n\
         {day_blocks}\
         平均矛盾分：{avg_score:.2}（Level {level}）\n\
         数据天数：{days_with_data}/7\n\n\
         请按以下结构输出周报：\n① 本周核心矛盾\n② 重复出现的行为模式\n③ 执行力趋势分析\n④ 深层机制推断\n",
        period_start=period_start, period_end=period_end,
        day_blocks=day_blocks, avg_score=avg_score, level=level,
        days_with_data=days_with_data
    )
}

fn prepare_report(report_type: &str, date_str: &str, brutal: bool) -> Result<PreparedReport, String> {
    let end_date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| format!("日期格式无效: {}", date_str))?;

    let (period_start, days_to_load) = if report_type == "weekly" {
        let start = end_date - Duration::days(6);
        let days: Vec<String> = (0..7)
            .map(|i| (start + Duration::days(i)).format("%Y-%m-%d").to_string())
            .collect();
        (start.format("%Y-%m-%d").to_string(), days)
    } else {
        (date_str.to_string(), vec![date_str.to_string()])
    };
    let period_end = date_str.to_string();

    let mut all_days: Vec<DayData> = Vec::new();
    for day in &days_to_load {
        let (journals, tasks, sleep, screen) = load_day_data_for_report(day);
        all_days.push(DayData {
            date: day.clone(),
            journals: journals.unwrap_or_default(),
            tasks: tasks.unwrap_or_default(),
            sleep,
            screen,
        });
    }

    let days_with_data = all_days.iter().filter(|d| {
        !d.journals.is_empty() || !d.tasks.is_empty() || d.sleep.is_some() || d.screen.is_some()
    }).count();

    if days_with_data == 0 {
        return Err(format!("{}没有找到任何数据", date_str));
    }

    let all_journals_count: usize = all_days.iter().map(|d| d.journals.len()).sum();
    let all_tasks: Vec<&Task> = all_days.iter().flat_map(|d| d.tasks.iter()).collect();
    let tasks_done = all_tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
    let tasks_skipped = all_tasks.iter().filter(|t| t.status == TaskStatus::Skipped).count();

    let sleep_entries: Vec<&SleepRecord> = all_days.iter().filter_map(|d| d.sleep.as_ref()).collect();
    let avg_sleep = if sleep_entries.is_empty() { 0.0 }
        else { sleep_entries.iter().map(|s| s.duration_hours).sum::<f64>() / sleep_entries.len() as f64 };

    let mut total_screen = 0u32;
    let mut total_productive = 0u32;
    let mut total_pickups = 0u32;
    let mut total_notifs = 0u32;
    for d in &all_days {
        if let Some(s) = &d.screen {
            total_screen += s.usage.total_minutes;
            total_productive += s.usage.category_minutes.iter()
                .filter(|c| c.name.to_lowercase().contains("productive"))
                .map(|c| c.minutes).sum::<u32>();
            total_pickups += s.pickups.total;
            total_notifs += s.notifications.total;
        }
    }

    let scores: Vec<f64> = all_days.iter()
        .filter(|d| !d.tasks.is_empty() || d.sleep.is_some() || d.screen.is_some())
        .map(|d| {
            report_task_gap(&d.tasks) * 0.40
            + report_attention_gap(d.screen.as_ref()) * 0.35
            + report_sleep_penalty(d.sleep.as_ref()) * 0.25
        }).collect();

    let contradiction_score = if scores.is_empty() { 0.5 }
        else { scores.iter().sum::<f64>() / scores.len() as f64 };

    let intensity: u8 = if brutal || contradiction_score >= 0.7 { 3 }
        else if contradiction_score >= 0.4 { 2 }
        else { 1 };

    let summary = ReportDataSummary {
        journal_entries: all_journals_count,
        tasks_total: all_tasks.len(),
        tasks_done,
        tasks_skipped,
        sleep_hours: (avg_sleep * 10.0).round() / 10.0,
        screen_total_minutes: total_screen,
        screen_productive_minutes: total_productive,
        pickups: total_pickups,
        notifications: total_notifs,
    };

    let user_prompt = if report_type == "weekly" {
        build_weekly_user_prompt(&period_start, &period_end, &all_days, days_with_data, contradiction_score, intensity)
    } else {
        let d = &all_days[0];
        build_single_user_prompt(date_str, &d.journals, &d.tasks, d.sleep.as_ref(), d.screen.as_ref(), contradiction_score, intensity)
    };

    let full_prompt = format!("<system>\n{}\n</system>\n{}", report_system_for_level(intensity), user_prompt);

    let (report_id, file_name) = if report_type == "weekly" {
        (format!("report_{}_7d", date_str.replace('-', "")), format!("{}_7d.json", date_str))
    } else {
        (format!("report_{}", date_str.replace('-', "")), format!("{}.json", date_str))
    };

    Ok(PreparedReport {
        report_type: report_type.to_string(),
        date: date_str.to_string(),
        period_start,
        period_end,
        contradiction_score,
        intensity_level: intensity,
        summary,
        full_prompt,
        report_id,
        file_name,
        days_with_data,
    })
}

// ── GET /api/reports ──────────────────────────────────────────────────────────

pub async fn get_reports() -> Json<Vec<ReportListItem>> {
    let dir = reports_dir();
    let mut items: Vec<ReportListItem> = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(report) = serde_json::from_str::<Report>(&content) {
                    items.push(ReportListItem {
                        id: report.id,
                        generated_at: report.generated_at,
                        report_type: report.report_type,
                        date: report.date,
                        contradiction_score: report.contradiction_score,
                        intensity_level: report.intensity_level,
                    });
                }
            }
        }
    }

    items.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));
    Json(items)
}

// ── GET /api/reports/:id ─────────────────────────────────────────────────────

pub async fn get_report_by_id(Path(id): Path<String>) -> impl IntoResponse {
    let dir = reports_dir();
    if let Some(filename) = id_to_filename(&id) {
        let path = dir.join(&filename);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(report) = serde_json::from_str::<Report>(&content) {
                return Json(report).into_response();
            }
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "report not found"}))).into_response()
}

// ── DELETE /api/reports/:id ───────────────────────────────────────────────────

pub async fn delete_report(Path(id): Path<String>) -> impl IntoResponse {
    let dir = reports_dir();
    if let Some(filename) = id_to_filename(&id) {
        let path = dir.join(&filename);
        if path.exists() {
            return match fs::remove_file(&path) {
                Ok(_) => Json(json!({"deleted": true})).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
            };
        }
    }
    (StatusCode::NOT_FOUND, Json(json!({"error": "report not found"}))).into_response()
}

// ── POST /api/reports/generate (SSE streaming) ────────────────────────────────

pub async fn generate_report(Json(body): Json<GenerateBody>) -> impl IntoResponse {
    use async_stream::stream;
    use axum::response::sse::{Event, KeepAlive, Sse};
    use futures::StreamExt as _;
    use std::convert::Infallible;

    let report_type = body.report_type.clone();
    let date = body.date.clone();
    let brutal = body.brutal.unwrap_or(false);

    let prepared_result = prepare_report(&report_type, &date, brutal);

    let s = stream! {
        let prepared = match prepared_result {
            Ok(p) => p,
            Err(e) => {
                yield Ok::<Event, Infallible>(Event::default().data(
                    serde_json::json!({"type":"error","message": e}).to_string()
                ));
                return;
            }
        };

        // Send summary event
        yield Ok::<Event, Infallible>(Event::default().data(
            serde_json::json!({
                "type": "summary",
                "contradiction_score": prepared.contradiction_score,
                "intensity_level": prepared.intensity_level,
                "period_start": prepared.period_start,
                "period_end": prepared.period_end,
                "report_type": prepared.report_type,
                "date": prepared.date,
                "data_summary": prepared.summary,
            }).to_string()
        ));

        // Call Ollama with streaming
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                yield Ok::<Event, Infallible>(Event::default().data(
                    serde_json::json!({"type":"error","message": format!("client build error: {}", e)}).to_string()
                ));
                return;
            }
        };

        let ollama_body = serde_json::json!({
            "model": "qwen2.5:7b",
            "prompt": prepared.full_prompt,
            "stream": true
        });

        let res = match client
            .post("http://localhost:11434/api/generate")
            .json(&ollama_body)
            .send().await
        {
            Ok(r) => r,
            Err(e) => {
                yield Ok::<Event, Infallible>(Event::default().data(
                    serde_json::json!({"type":"error","message": format!("Ollama连接失败: {}", e)}).to_string()
                ));
                return;
            }
        };

        let mut byte_stream = res.bytes_stream();
        let mut full_analysis = String::new();

        while let Some(chunk_result) = byte_stream.next().await {
            let chunk = match chunk_result {
                Ok(c) => c,
                Err(_) => break,
            };
            for line in String::from_utf8_lossy(&chunk).lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(tok) = v["response"].as_str() {
                        if !tok.is_empty() {
                            full_analysis.push_str(tok);
                            yield Ok::<Event, Infallible>(Event::default().data(
                                serde_json::json!({"type":"token","content":tok}).to_string()
                            ));
                        }
                    }
                }
            }
        }

        // Save report to disk
        let report = Report {
            id: prepared.report_id.clone(),
            generated_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%z").to_string(),
            report_type: prepared.report_type.clone(),
            date: prepared.date.clone(),
            period_start: prepared.period_start.clone(),
            period_end: prepared.period_end.clone(),
            contradiction_score: prepared.contradiction_score,
            intensity_level: prepared.intensity_level,
            data_summary: prepared.summary.clone(),
            analysis: full_analysis,
        };

        let rdir = reports_dir();
        let _ = fs::create_dir_all(&rdir);
        if let Ok(serialized) = serde_json::to_string_pretty(&report) {
            let _ = fs::write(rdir.join(&prepared.file_name), serialized);
        }

        yield Ok::<Event, Infallible>(Event::default().data(
            serde_json::json!({"type":"done","report_id": prepared.report_id}).to_string()
        ));
    };

    Sse::new(s).keep_alive(KeepAlive::default())
}
