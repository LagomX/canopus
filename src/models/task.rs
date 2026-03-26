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

/// Eisenhower Quadrant — replaces the old Priority enum.
/// Default is Q2 (important, not urgent) for backward compat with old files.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Quadrant {
    /// Urgent + Important
    Q1,
    /// Not Urgent + Important (default)
    #[default]
    Q2,
    /// Urgent + Not Important
    Q3,
    /// Not Urgent + Not Important
    Q4,
}

impl Quadrant {
    /// Execution weight used to compute the daily Execution Index.
    pub fn weight(&self) -> u32 {
        match self {
            Quadrant::Q1 => 4,
            Quadrant::Q2 => 3,
            Quadrant::Q3 => 1,
            Quadrant::Q4 => 0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Quadrant::Q1 => "Q1",
            Quadrant::Q2 => "Q2",
            Quadrant::Q3 => "Q3",
            Quadrant::Q4 => "Q4",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Quadrant::Q1 => "#FF3B30",
            Quadrant::Q2 => "#007AFF",
            Quadrant::Q3 => "#FF9500",
            Quadrant::Q4 => "#8E8E93",
        }
    }

    pub fn from_str(s: &str) -> Option<Quadrant> {
        match s.to_lowercase().as_str() {
            "q1" | "high"   => Some(Quadrant::Q1),
            "q2" | "medium" => Some(Quadrant::Q2),
            "q3" | "low"    => Some(Quadrant::Q3),
            "q4"            => Some(Quadrant::Q4),
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
    /// Old files may have `priority` instead — serde ignores unknown fields,
    /// and `#[serde(default)]` fills in Q2 when `quadrant` is absent.
    #[serde(default)]
    pub quadrant: Quadrant,
    pub domain: Option<String>,
    pub skip_reason: Option<String>,
    pub notes: Option<String>,
}

/// Compute the Execution Index (0–100) for a slice of tasks.
pub fn calc_exec_index(tasks: &[Task]) -> f64 {
    let max: u32 = tasks.iter().map(|t| t.quadrant.weight()).sum();
    if max == 0 {
        return 0.0;
    }
    let actual: f64 = tasks
        .iter()
        .map(|t| {
            let w = t.quadrant.weight() as f64;
            match t.status {
                TaskStatus::Done => w,
                TaskStatus::Partial => w * 0.5,
                _ => 0.0,
            }
        })
        .sum();
    (actual / max as f64 * 100.0).round().min(100.0)
}
