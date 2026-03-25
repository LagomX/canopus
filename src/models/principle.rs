use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PrincipleStatus {
    Candidate,
    Validated,
    Confirmed,
    Deprecated,
}

impl std::fmt::Display for PrincipleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrincipleStatus::Candidate  => write!(f, "candidate"),
            PrincipleStatus::Validated  => write!(f, "validated"),
            PrincipleStatus::Confirmed  => write!(f, "confirmed"),
            PrincipleStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Evidence {
    pub date: String,
    pub observation_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Validation {
    pub date: String,
    pub decision: String,
    pub outcome: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusTransition {
    pub from: PrincipleStatus,
    pub to: PrincipleStatus,
    pub date: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Principle {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: PrincipleStatus,
    pub domain: Option<String>,
    pub evidence: Vec<Evidence>,
    pub validations: Vec<Validation>,
    pub history: Vec<StatusTransition>,
    pub created_at: String,
    pub updated_at: String,
}
