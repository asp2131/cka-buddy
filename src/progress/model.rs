use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletedStep {
    pub completed_at: DateTime<Utc>,
    pub attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgressState {
    pub active_step_id: Option<String>,
    pub completed: HashMap<String, CompletedStep>,
    pub attempts: HashMap<String, u32>,
}
