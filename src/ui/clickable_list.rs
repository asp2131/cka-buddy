use crossterm::event::{MouseButton, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::Widget,
};

use super::{
    callback_registry::CallbackRegistry, constants::UiStyle, traits::InteractiveWidget,
    ui_action::UiAction,
};

#[derive(Debug, Clone)]
pub struct ClickableList<'a> {
    items: Vec<String>,
    actions: Vec<UiAction>,
    style: Style,
    layer: usize,
    marker: &'a str,
}

impl<'a> ClickableList<'a> {
    pub fn new(items: Vec<String>, actions: Vec<UiAction>) -> Self {
        Self {
            items,
            actions,
            style: UiStyle::TEXT_PRIMARY,
            layer: 0,
            marker: "  ",
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }

    pub fn marker(mut self, marker: &'a str) -> Self {
        self.marker = marker;
        self
    }
}

impl Widget for ClickableList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let visible = self.items.len().min(area.height as usize);
        for i in 0..visible {
            let y = area.y + i as u16;
            let text = format!("{}{}", self.marker, self.items[i]);
            let line = Line::from(Span::styled(text, self.style));
            line.render(Rect::new(area.x, y, area.width, 1), buf);
        }
    }
}

impl InteractiveWidget for ClickableList<'_> {
    fn layer(&self) -> usize {
        self.layer
    }

    fn before_rendering(&mut self, area: Rect, registry: &mut CallbackRegistry) {
        let visible = self.items.len().min(area.height as usize);
        for i in 0..visible {
            if let Some(action) = self.actions.get(i).cloned() {
                let row = Rect::new(area.x, area.y + i as u16, area.width, 1);
                registry.register_mouse_callback_on_layer(
                    MouseEventKind::Down(MouseButton::Left),
                    Some(row),
                    action,
                    self.layer,
                );
            }
        }
    }

    fn hover_text(&self) -> Text<'_> {
        Text::from("Click a command to load it into the input bar")
    }
}
