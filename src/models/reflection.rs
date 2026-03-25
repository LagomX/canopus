use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reflection {
    pub id: String,
    pub date: String,
    pub period_start: String,
    pub period_end: String,
    pub observations_used: Vec<String>,
    pub patterns: Vec<Pattern>,
    pub candidate_principles: Vec<CandidatePrinciple>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pattern {
    pub description: String,
    pub frequency: u32,
    pub example_dates: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CandidatePrinciple {
    pub title: String,
    pub description: String,
    pub domain: Option<String>,
    pub supporting_pattern: String,
}
