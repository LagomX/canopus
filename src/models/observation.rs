use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Observation {
    pub id: String,
    pub date: String,
    pub content: String,
    pub source: ObservationSource,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ObservationSource {
    Auto,
    Manual,
}
