mod deterministic;
#[cfg(feature = "llm")]
mod llm;

use crate::content::models::Step;

pub trait CoachAdvisor {
    fn hint(&self, step: &Step, recent_output: &[String]) -> String;
}

pub fn build_coach() -> Box<dyn CoachAdvisor + Send + Sync> {
    #[cfg(feature = "llm")]
    {
        if let Ok(coach) = llm::LlmCoach::from_env() {
            return Box::new(coach);
        }
    }
    Box::new(deterministic::DeterministicCoach)
}
