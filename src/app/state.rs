#[derive(Debug, Clone, Default)]
pub struct CompletionCard {
    pub done: String,
    pub what_changed: Vec<String>,
    pub next_commands: Vec<String>,
    pub verify_optional: Vec<String>,
}
