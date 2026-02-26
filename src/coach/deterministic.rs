use crate::coach::CoachAdvisor;
use crate::content::models::Step;

pub struct DeterministicCoach;

impl CoachAdvisor for DeterministicCoach {
    fn hint(&self, step: &Step, _recent_output: &[String]) -> String {
        if let Some(hint) = step.fallback_hint.as_ref() {
            return format!("Hint: {}", hint);
        }
        if let Some(cmd) = step.run_commands.first() {
            return format!("Hint: start with `{}`", cmd);
        }
        "Hint: run the first checklist item, then use verify.".to_string()
    }
}
