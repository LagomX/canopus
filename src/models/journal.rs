use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: String,
    pub timestamp: String,
    pub date: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mood: Option<String>,
    pub tags: Vec<String>,
}
