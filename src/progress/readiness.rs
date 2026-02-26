use crate::content::models::Step;
use crate::progress::model::ProgressState;

pub fn calculate_readiness(steps: &[Step], progress: &ProgressState) -> u8 {
    let total_weight: u32 = steps.iter().map(|s| s.ready_weight).sum();
    if total_weight == 0 {
        return 0;
    }

    let completed_weight: u32 = steps
        .iter()
        .filter(|s| progress.completed.contains_key(&s.id))
        .map(|s| s.ready_weight)
        .sum();

    ((completed_weight as f64 / total_weight as f64) * 100.0).round() as u8
}
