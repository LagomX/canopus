use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SleepRecord {
    pub id: String,
    pub date: String,
    pub duration_hours: f64,
    pub quality_score: u8,
    pub bedtime: Option<String>,
    pub wake_time: Option<String>,
    pub notes: Option<String>,
}
