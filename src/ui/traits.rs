use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use crate::app::engine::Engine;

use super::ui_action::UiAction;

pub trait Screen {
    fn update(&mut self, engine: &Engine) -> anyhow::Result<()>;

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) -> anyhow::Result<()>;

    fn handle_key_events(&mut self, _key_event: KeyEvent, _engine: &Engine) -> Option<UiAction> {
        None
    }

    fn footer_help(&self) -> String {
        String::new()
    }
}
