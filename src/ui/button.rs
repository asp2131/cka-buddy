use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

use super::constants::UiStyle;

#[derive(Debug, Clone)]
pub struct Button<'a> {
    text: String,
    selected: bool,
    disabled: bool,
    style: Style,
    selected_style: Style,
    block: Option<Block<'a>>,
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
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = self.block.unwrap_or_else(Block::bordered);

        if self.selected {
            block = block.border_set(border::THICK).border_style(UiStyle::HIGHLIGHT);
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
