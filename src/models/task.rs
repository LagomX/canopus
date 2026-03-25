use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    Done,
    Skipped,
    Partial,
    CarriedOver,
}

impl TaskStatus {
    /// Returns the display icon for this status.
    pub fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "□",
            TaskStatus::Done => "✓",
            TaskStatus::Skipped => "✗",
            TaskStatus::Partial => "◑",
            TaskStatus::CarriedOver => "→",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn from_str(s: &str) -> Option<Priority> {
        match s.to_lowercase().as_str() {
            "high" => Some(Priority::High),
            "medium" => Some(Priority::Medium),
            "low" => Some(Priority::Low),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub date: String,
    pub title: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub domain: Option<String>,
    pub skip_reason: Option<String>,
    pub notes: Option<String>,
}
