use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepType {
    Project,
    Bug,
    Exam,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub id: String,
    pub title: String,
    pub step_type: StepType,
    pub domains: Vec<String>,
    pub difficulty: String,
    pub timebox_min: u32,
    pub objective: String,
    pub run_items: Vec<String>,
    pub run_commands: Vec<String>,
    pub success_check_commands: Vec<String>,
    pub success_contains: Vec<String>,
    pub verify_commands: Vec<String>,
    pub fallback_hint: Option<String>,
    pub what_changed: Vec<String>,
    pub optional: bool,
    pub path: PathBuf,
    pub ready_weight: u32,
}
