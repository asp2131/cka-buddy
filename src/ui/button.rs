use crossterm::event::{KeyCode, MouseButton, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use super::{
    callback_registry::CallbackRegistry, constants::UiStyle, traits::InteractiveWidget,
    ui_action::UiAction,
};

#[derive(Debug, Clone)]
pub struct Button<'a> {
    text: String,
    selected: bool,
    disabled: bool,
    style: Style,
    selected_style: Style,
    block: Option<Block<'a>>,
    on_click: Option<UiAction>,
    hotkey: Option<KeyCode>,
    hover_text: Option<Text<'a>>,
    layer: usize,
}

impl<'a> Button<'a> {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            selected: false,
            disabled: false,
            style: UiStyle::TEXT_SECONDARY,
            selected_style: UiStyle::HIGHLIGHT,
            block: None,
            on_click: None,
            hotkey: None,
            hover_text: None,
            layer: 0,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn on_click(mut self, action: UiAction) -> Self {
        self.on_click = Some(action);
        self
    }

    pub fn hotkey(mut self, key: KeyCode) -> Self {
        self.hotkey = Some(key);
        self
    }

    pub fn hover_text(mut self, text: Text<'a>) -> Self {
        self.hover_text = Some(text);
        self
    }

    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = self.block.unwrap_or_else(Block::bordered);

        if self.selected {
            block = block
                .border_set(border::THICK)
                .border_style(UiStyle::HIGHLIGHT);
        } else {
            block = block.border_style(UiStyle::BORDER);
        }

        let text_style = if self.disabled {
            UiStyle::MUTED
        } else if self.selected {
            self.selected_style
        } else {
            self.style
        };

        Paragraph::new(Line::from(format!(" {} ", self.text)))
            .style(text_style)
            .block(block)
            .centered()
            .render(area, buf);
    }
}

#[derive(Debug, Default)]
pub struct ButtonState;

impl StatefulWidget for Button<'_> {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        Widget::render(self, area, buf);
    }
}

impl InteractiveWidget for Button<'_> {
    fn layer(&self) -> usize {
        self.layer
    }

    fn before_rendering(&mut self, area: Rect, registry: &mut CallbackRegistry) {
        if let Some(action) = self.on_click.clone() {
            registry.register_mouse_callback_on_layer(
                MouseEventKind::Down(MouseButton::Left),
                Some(area),
                action.clone(),
                self.layer,
            );

            if let Some(key) = self.hotkey {
                registry.register_keyboard_callback(key, action);
            }
        }
    }

    fn hover_text(&self) -> Text<'_> {
        self.hover_text.clone().unwrap_or_default()
    }
}
