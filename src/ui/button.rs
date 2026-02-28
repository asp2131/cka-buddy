use crossterm::event::{KeyCode, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use super::{
    callback_registry::CallbackRegistry,
    constants::UiStyle,
    traits::InteractiveWidget,
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
    hotkey: Option<KeyCode>,
    on_click: Option<UiAction>,
    is_hovered: bool,
    hover_text_str: Option<String>,
    layer: usize,
    command_input_empty: bool,
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
            hotkey: None,
            on_click: None,
            is_hovered: false,
            hover_text_str: None,
            layer: 0,
            command_input_empty: true,
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

    pub fn hotkey(mut self, key: KeyCode) -> Self {
        self.hotkey = Some(key);
        self
    }

    pub fn on_click(mut self, action: UiAction) -> Self {
        self.on_click = Some(action);
        self
    }

    pub fn hover_text(mut self, text: impl Into<String>) -> Self {
        self.hover_text_str = Some(text.into());
        self
    }

    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }

    pub fn command_input_empty(mut self, empty: bool) -> Self {
        self.command_input_empty = empty;
        self
    }

    fn hotkey_char(&self) -> Option<char> {
        match self.hotkey {
            Some(KeyCode::Char(c)) => Some(c),
            _ => None,
        }
    }
}

impl InteractiveWidget for Button<'_> {
    fn layer(&self) -> usize {
        self.layer
    }

    fn before_rendering(&mut self, area: Rect, registry: &mut CallbackRegistry) {
        if self.disabled {
            return;
        }
        if let Some(ref action) = self.on_click {
            registry.register_mouse_callback(
                MouseEventKind::Down(crossterm::event::MouseButton::Left),
                Some(area),
                action.clone(),
                self.layer,
            );
        }
        if let Some(key) = self.hotkey
            && self.command_input_empty
            && let Some(ref action) = self.on_click
        {
            registry.register_keyboard_callback(key, action.clone(), self.layer);
        }
        self.is_hovered = registry.is_hovering(area);
    }

    fn hover_text(&self) -> String {
        self.hover_text_str.clone().unwrap_or_default()
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = self.block.clone().unwrap_or_else(Block::bordered);

        if self.selected || self.is_hovered {
            block = block
                .border_set(border::THICK)
                .border_style(if self.selected {
                    UiStyle::HIGHLIGHT
                } else {
                    UiStyle::BUTTON_ACTIVE
                });
        } else {
            block = block.border_style(UiStyle::BORDER);
        }

        let inner = block.inner(area);
        block.render(area, buf);

        let text_style = if self.disabled {
            UiStyle::MUTED
        } else if self.is_hovered {
            UiStyle::BUTTON_ACTIVE
        } else if self.selected {
            self.selected_style
        } else {
            self.style
        };

        let hotkey_char = self.hotkey_char();
        let display = format!(" {} ", self.text);

        if let Some(hk) = hotkey_char {
            let lower_hk = hk.to_lowercase().next().unwrap_or(hk);
            let mut spans = Vec::new();
            let mut found = false;

            for ch in display.chars() {
                if !found && ch.to_lowercase().next().unwrap_or(ch) == lower_hk {
                    spans.push(Span::styled(
                        ch.to_string(),
                        UiStyle::HOTKEY_UNDERLINE.add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
                    ));
                    found = true;
                } else {
                    spans.push(Span::styled(ch.to_string(), text_style));
                }
            }

            let line = Line::from(spans);
            let p = Paragraph::new(line).centered();
            p.render(inner, buf);
        } else {
            Paragraph::new(Line::from(display))
                .style(text_style)
                .centered()
                .render(inner, buf);
        }
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
