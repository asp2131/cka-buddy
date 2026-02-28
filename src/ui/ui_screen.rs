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
    callback_registry: CallbackRegistry,
}

impl UiScreen {
    pub fn new(initial_status: String, has_progress: bool) -> Self {
        Self {
            state: ScreenState::Splash,
            splash: SplashScreen::new(has_progress),
            learning: LearningScreen::new(initial_status),
            popup_stack: Vec::new(),
            callback_registry: CallbackRegistry::default(),
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
        mouse_position: Option<(u16, u16)>,
    ) -> Result<()> {
        let area = centered_clamped_viewport(frame.area());

        let (body_area, footer_area, hover_area) = split_shell(area);

        let mut ui_frame = UiFrame::new(frame, hover_area.unwrap_or(Rect::new(0, 0, 0, 0)), mouse_position);
        ui_frame.set_active_layer(if self.has_popup() { 1 } else { 0 });

        match self.state {
            ScreenState::Splash => self.splash.render(&mut ui_frame, engine, body_area)?,
            ScreenState::Learning => self.learning.render(&mut ui_frame, engine, body_area)?,
        }

        if self.popup_stack.last().is_some() {
            ui_frame.render_widget(Clear, body_area);
            ui_frame.render_widget(
                ratatui::widgets::Block::default()
                    .style(Style::default().bg(Color::Rgb(14, 18, 28))),
                body_area,
            );
            match self.state {
                ScreenState::Splash => self.splash.render(&mut ui_frame, engine, body_area)?,
                ScreenState::Learning => self.learning.render(&mut ui_frame, engine, body_area)?,
            }
        }

        if let Some(footer) = footer_area {
            render_footer(&mut ui_frame, footer, self.active_footer_spans());
        }

        if let Some(hover) = hover_area {
            render_hover_help(&mut ui_frame, hover);
        }

        self.callback_registry = ui_frame.into_registry();

        if let Some(popup) = self.popup_stack.last() {
            popup.render(frame, area);
        }
        Ok(())
    }

    fn active_footer_spans(&self) -> Vec<(String, String)> {
        if let Some(popup) = self.popup_stack.last() {
            return match popup {
                PopupMessage::Tutorial { .. } => vec![
                    ("←/→".to_string(), "Navigate".to_string()),
                    ("Enter".to_string(), "Start/Close".to_string()),
                    ("Esc".to_string(), "Close".to_string()),
                ],
                _ => vec![
                    ("Enter".to_string(), "Close".to_string()),
                    ("Esc".to_string(), "Close".to_string()),
                ],
            };
        }

        match self.state {
            ScreenState::Splash => self.splash.footer_spans(),
            ScreenState::Learning => self.learning.footer_spans(),
        }
    }

    pub fn callback_registry(&self) -> &CallbackRegistry {
        &self.callback_registry
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

fn split_shell(area: Rect) -> (Rect, Option<Rect>, Option<Rect>) {
    if area.height < 4 {
        return (area, None, None);
    }

    if area.height < 6 {
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
        return (chunks[0], Some(chunks[1]), None);
    }

    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);
    (chunks[0], Some(chunks[1]), Some(chunks[2]))
}

fn render_footer(frame: &mut UiFrame<'_, '_>, area: Rect, spans: Vec<(String, String)>) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let mut cells: Vec<Span<'static>> = Vec::new();
    for (key, desc) in spans {
        cells.push(Span::styled(format!(" {key} "), UiStyle::FOOTER_KEY));
        cells.push(Span::styled(format!(" {desc} "), UiStyle::FOOTER_DESC));
        cells.push(Span::raw(" "));
    }

    frame.render_widget(Paragraph::new(Line::from(cells)), area);
}

fn render_hover_help(frame: &mut UiFrame<'_, '_>, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let text = frame
        .hover_text()
        .unwrap_or("Type commands or click actions")
        .to_string();
    frame.render_widget(Paragraph::new(text).style(UiStyle::HOVER_HELP), area);
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
