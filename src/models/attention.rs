use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenTimeRecord {
    pub id: String,           // "attention_YYYY-MM-DD"
    pub date: String,         // "YYYY-MM-DD"
    pub source: String,       // "manual_from_screenshots"
    pub captured_at: String,  // ISO 8601 with timezone offset
    pub usage: ScreenTimeUsage,
    pub notifications: NotificationData,
    pub pickups: PickupData,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenTimeUsage {
    pub total_minutes: u32,
    pub category_minutes: Vec<CategoryMinutes>,
    pub top_apps: Vec<AppMinutes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryMinutes {
    pub name: String,
    pub minutes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppMinutes {
    pub name: String,
    pub minutes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationData {
    pub total: u32,
    pub top_apps: Vec<AppCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PickupData {
    pub total: u32,
    pub top_apps: Vec<AppCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppCount {
    pub name: String,
    pub count: u32,
}
