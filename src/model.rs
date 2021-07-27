use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct HeartBeat {
    pub(crate) branch: Option<String>,
    pub(crate) category: Option<String>,
    pub(crate) entity: Option<String>,
    pub(crate) is_write: Option<bool>,
    pub(crate) language: Option<String>,
    pub(crate) lineno: Option<i32>,
    pub(crate) lines: Option<i32>,
    pub(crate) project: Option<String>,
    pub(crate) time: Option<DateTime<Utc>>,
    pub(crate) user_agent: Option<String>,
    pub(crate) machine_name: Option<String>,
}
