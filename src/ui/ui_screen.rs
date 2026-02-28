use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use crate::app::engine::Engine;

use super::{
    callback_registry::CallbackRegistry,
    constants::{UiStyle, centered_clamped_viewport},
    learning_screen::LearningScreen,
    popup::PopupMessage,
    splash_screen::SplashScreen,
    traits::Screen,
    ui_action::UiAction,
    ui_frame::UiFrame,
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

    pub fn render(
        &mut self,
        frame: &mut Frame,
        engine: &Engine,
        mouse_pos: Option<(u16, u16)>,
    ) -> CallbackRegistry {
        let area = centered_clamped_viewport(frame.area());

        let (body, footer, hover) = split_shell(area);

        let mut ui_frame = UiFrame::new(frame, hover, mouse_pos);

        if self.has_popup() {
            ui_frame.set_active_layer(1);
        }

        match self.state {
            ScreenState::Splash => {
                let _ = self.splash.render(&mut ui_frame, engine, body);
            }
            ScreenState::Learning => {
                let _ = self.learning.render(&mut ui_frame, engine, body);
            }
        }

        let footer_spans = match self.state {
            ScreenState::Splash => self.splash.footer_spans(),
            ScreenState::Learning => self.learning.footer_spans(),
        };
        render_footer(&mut ui_frame, footer, &footer_spans);

        ui_frame.render_hover_text();

        if let Some(popup) = self.popup_stack.last() {
            let f = ui_frame.inner_frame();
            f.render_widget(Clear, area);
            f.render_widget(
                ratatui::widgets::Block::default()
                    .style(Style::default().bg(Color::Rgb(14, 18, 28))),
                area,
            );
            let body2 = Rect::new(body.x, body.y, body.width, body.height);
            match self.state {
                ScreenState::Splash => {
                    let _ = self.splash.render(&mut ui_frame, engine, body2);
                }
                ScreenState::Learning => {
                    let _ = self.learning.render(&mut ui_frame, engine, body2);
                }
            }
            popup.render(ui_frame.inner_frame(), area);
        }

        ui_frame.into_registry()
    }

    pub fn handle_key_events(&mut self, key: KeyEvent, engine: &Engine) -> Option<UiAction> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Some(UiAction::Quit);
        }

        if let Some(popup) = self.popup_stack.last_mut() {
            return handle_popup_key(key, popup);
        }

        match self.state {
            ScreenState::Splash => self.splash.handle_key_events(key, engine),
            ScreenState::Learning => self.learning.handle_key_events(key, engine),
        }
    }
}

fn split_shell(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);
    (chunks[0], chunks[1], chunks[2])
}

fn render_footer(frame: &mut UiFrame, area: Rect, spans: &[(&str, &str)]) {
    if area.width == 0 || spans.is_empty() {
        return;
    }
    let mut parts: Vec<Span> = Vec::new();
    for (key, desc) in spans {
        parts.push(Span::styled(format!(" {} ", key), UiStyle::FOOTER_KEY));
        parts.push(Span::styled(format!(" {} ", desc), UiStyle::FOOTER_DESC));
    }
    frame.render_widget(Paragraph::new(Line::from(parts)), area);
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
