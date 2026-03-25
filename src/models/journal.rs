use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub timestamp: String,
    pub date: String,
    pub content: String,
    pub mood_score: Option<u8>,
    pub energy_score: Option<u8>,
    pub tags: Vec<String>,
    pub intentions: Vec<String>,
}
