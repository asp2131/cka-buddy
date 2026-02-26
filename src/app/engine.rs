use chrono::Utc;

use crate::content::models::Step;
use crate::progress::model::{CompletedStep, ProgressState};
use crate::progress::readiness::calculate_readiness;

pub struct Engine {
    pub steps: Vec<Step>,
    pub progress: ProgressState,
    pub current_index: usize,
    pub readiness: u8,
}

impl Engine {
    pub fn new(steps: Vec<Step>, progress: ProgressState) -> Self {
        let mut current_index = 0;
        if let Some(active_id) = progress.active_step_id.as_ref() {
            if let Some(idx) = steps.iter().position(|s| &s.id == active_id) {
                current_index = idx;
            }
        } else if let Some(idx) = steps
            .iter()
            .position(|s| !progress.completed.contains_key(&s.id))
        {
            current_index = idx;
        }

        let readiness = calculate_readiness(&steps, &progress);
        Self {
            steps,
            progress,
            current_index,
            readiness,
        }
    }

    pub fn current_step(&self) -> &Step {
        &self.steps[self.current_index]
    }

    pub fn prev_step(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.progress.active_step_id = Some(self.current_step().id.clone());
        }
    }

    pub fn next_step(&mut self) {
        if self.current_index + 1 < self.steps.len() {
            self.current_index += 1;
            self.progress.active_step_id = Some(self.current_step().id.clone());
        }
    }

    pub fn recommended_next(&self) -> Option<usize> {
        self.steps
            .iter()
            .position(|s| !self.progress.completed.contains_key(&s.id))
    }

    pub fn jump_recommended(&mut self) {
        if let Some(idx) = self.recommended_next() {
            self.current_index = idx;
            self.progress.active_step_id = Some(self.current_step().id.clone());
        }
    }

    pub fn jump_prev_completed(&mut self) {
        if self.current_index == 0 {
            return;
        }
        for idx in (0..self.current_index).rev() {
            let id = &self.steps[idx].id;
            if self.progress.completed.contains_key(id) {
                self.current_index = idx;
                self.progress.active_step_id = Some(self.current_step().id.clone());
                return;
            }
        }
    }

    pub fn record_attempt(&mut self) {
        let id = self.current_step().id.clone();
        *self.progress.attempts.entry(id).or_insert(0) += 1;
    }

    pub fn complete_current(&mut self) {
        let id = self.current_step().id.clone();
        let attempts = self.progress.attempts.get(&id).copied().unwrap_or(0);
        self.progress.completed.insert(
            id,
            CompletedStep {
                completed_at: Utc::now(),
                attempts,
            },
        );

        self.readiness = calculate_readiness(&self.steps, &self.progress);
        if self.current_index + 1 < self.steps.len() {
            self.current_index += 1;
            self.progress.active_step_id = Some(self.current_step().id.clone());
        }
    }
}
