use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{StatefulWidget, Widget},
};

use crate::app::engine::Engine;

use super::{callback_registry::CallbackRegistry, ui_action::UiAction, ui_frame::UiFrame};

pub trait Screen {
    fn update(&mut self, engine: &Engine) -> anyhow::Result<()>;

    fn render(&mut self, frame: &mut UiFrame<'_, '_>, engine: &Engine, area: Rect) -> anyhow::Result<()>;

    fn handle_key_events(&mut self, _key_event: KeyEvent, _engine: &Engine) -> Option<UiAction> {
        None
    }

    fn footer_help(&self) -> String {
        String::new()
    }

    fn footer_spans(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}

pub trait InteractiveWidget: Widget {
    fn layer(&self) -> usize {
        0
    }

    fn before_rendering(&mut self, _area: Rect, _registry: &mut CallbackRegistry) {}

    fn hover_text(&self) -> Text<'_> {
        Text::default()
    }
}

pub trait InteractiveStatefulWidget: StatefulWidget {
    fn layer(&self) -> usize {
        0
    }

    fn before_rendering(&mut self, _area: Rect, _registry: &mut CallbackRegistry) {}

    fn hover_text(&self) -> Text<'_> {
        Text::default()
    }
}
