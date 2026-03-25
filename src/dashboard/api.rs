use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use canopus::models::journal::JournalEntry;
use canopus::models::sleep::SleepRecord;
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
