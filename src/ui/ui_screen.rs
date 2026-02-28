use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::Clear,
};

use crate::app::engine::Engine;

use super::{
    constants::centered_clamped_viewport, learning_screen::LearningScreen, popup::PopupMessage,
    splash_screen::SplashScreen, traits::Screen, ui_action::UiAction, ui_frame::UiFrame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenState {
    Splash,
    Learning,
}

pub struct UiScreen {
    pub state: ScreenState,
    pub splash: SplashScreen,
    pub learning: LearningScreen,
    pub popup_stack: Vec<PopupMessage>,
}

impl UiScreen {
    pub fn new(initial_status: String, has_progress: bool) -> Self {
        Self {
            state: ScreenState::Splash,
            splash: SplashScreen::new(has_progress),
            learning: LearningScreen::new(initial_status),
            popup_stack: Vec::new(),
        }
    }

    pub fn push_popup(&mut self, popup: PopupMessage) {
        self.popup_stack.push(popup);
    }

    pub fn dismiss_popup(&mut self) {
        self.popup_stack.pop();
    }

    pub fn has_popup(&self) -> bool {
        !self.popup_stack.is_empty()
    }

    pub fn transition_to_learning(&mut self) {
        self.state = ScreenState::Learning;
    }

    pub fn update(&mut self, engine: &Engine) -> Result<()> {
        match self.state {
            ScreenState::Splash => self.splash.update(engine)?,
            ScreenState::Learning => self.learning.update(engine)?,
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame, engine: &Engine) -> Result<()> {
        let area = centered_clamped_viewport(frame.area());
        let mut ui_frame = UiFrame::new(frame, area, None);
        match self.state {
            ScreenState::Splash => self.splash.render(&mut ui_frame, engine, area)?,
            ScreenState::Learning => self.learning.render(&mut ui_frame, engine, area)?,
        }

        if let Some(popup) = self.popup_stack.last() {
            let f = ui_frame.inner_frame();
            f.render_widget(Clear, area);
            f.render_widget(
                ratatui::widgets::Block::default()
                    .style(Style::default().bg(Color::Rgb(14, 18, 28))),
                area,
            );
            match self.state {
                ScreenState::Splash => self.splash.render(&mut ui_frame, engine, area)?,
                ScreenState::Learning => self.learning.render(&mut ui_frame, engine, area)?,
            }
            popup.render(ui_frame.inner_frame(), area);
        }

        Ok(())
    }

    pub fn handle_key_events(&mut self, key: KeyEvent, engine: &Engine) -> Option<UiAction> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Some(UiAction::Quit);
        }

        // Popup takes priority
        if let Some(popup) = self.popup_stack.last_mut() {
            return handle_popup_key(key, popup);
        }

        match self.state {
            ScreenState::Splash => self.splash.handle_key_events(key, engine),
            ScreenState::Learning => self.learning.handle_key_events(key, engine),
        }
    }
}

fn handle_popup_key(key: KeyEvent, popup: &mut PopupMessage) -> Option<UiAction> {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => Some(UiAction::DismissPopup),
        KeyCode::Left => {
            if let PopupMessage::Tutorial { .. } = popup {
                popup.navigate_tutorial(-1);
            }
            Some(UiAction::None)
        }
        KeyCode::Right => {
            if let PopupMessage::Tutorial { .. } = popup {
                popup.navigate_tutorial(1);
            }
            Some(UiAction::None)
        }
        _ => Some(UiAction::None),
    }
}
