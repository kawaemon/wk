use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HeartBeat {
    pub branch: Option<String>,
    pub category: Option<String>,
    pub entity: Option<String>,
    pub is_write: Option<bool>,
    pub language: Option<String>,
    pub lineno: Option<i32>,
    pub lines: Option<i32>,
    pub project: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub user_agent: Option<String>,
    pub machine_name: Option<String>,
}
