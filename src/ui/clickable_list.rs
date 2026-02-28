use crossterm::event::MouseEventKind;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::Widget,
};

use super::{
    callback_registry::CallbackRegistry,
    constants::UiStyle,
    traits::InteractiveWidget,
    ui_action::UiAction,
};

#[derive(Debug, Clone)]
pub struct ClickableListItem {
    pub label: String,
    pub description: String,
    pub on_click: UiAction,
}

pub struct ClickableList<'a> {
    items: &'a [ClickableListItem],
    selected: Option<usize>,
    hover_index: Option<usize>,
    scroll_offset: usize,
    layer: usize,
}

impl<'a> ClickableList<'a> {
    pub fn new(items: &'a [ClickableListItem]) -> Self {
        Self {
            items,
            selected: None,
            hover_index: None,
            scroll_offset: 0,
            layer: 0,
        }
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }
}

impl InteractiveWidget for ClickableList<'_> {
    fn layer(&self) -> usize {
        self.layer
    }

    fn before_rendering(&mut self, area: Rect, registry: &mut CallbackRegistry) {
        if self.items.is_empty() || area.height == 0 {
            return;
        }

        let visible_rows = area.height as usize;

        // Register scroll callbacks (global within layer)
        registry.register_mouse_callback(
            MouseEventKind::ScrollUp,
            Some(area),
            UiAction::PreviousPanelIndex,
            self.layer,
        );
        registry.register_mouse_callback(
            MouseEventKind::ScrollDown,
            Some(area),
            UiAction::NextPanelIndex,
            self.layer,
        );

        // Detect hover and register per-row click callbacks
        let mut hover_idx = None;
        for row in 0..visible_rows {
            let item_idx = self.scroll_offset + row;
            if item_idx >= self.items.len() {
                break;
            }
            let row_rect = Rect::new(area.x, area.y + row as u16, area.width, 1);
            registry.register_mouse_callback(
                MouseEventKind::Down(crossterm::event::MouseButton::Left),
                Some(row_rect),
                self.items[item_idx].on_click.clone(),
                self.layer,
            );
            if registry.is_hovering(row_rect) {
                hover_idx = Some(item_idx);
            }
        }
        self.hover_index = hover_idx;
    }

    fn hover_text(&self) -> String {
        if let Some(idx) = self.hover_index
            && let Some(item) = self.items.get(idx)
        {
            return item.description.clone();
        }
        String::new()
    }
}

impl Widget for ClickableList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 || self.items.is_empty() {
            return;
        }

        let visible = area.height as usize;
        let total = self.items.len();
        let start = self.scroll_offset.min(total.saturating_sub(visible));

        for row in 0..visible {
            let idx = start + row;
            if idx >= total {
                break;
            }

            let item = &self.items[idx];
            let y = area.y + row as u16;
            let w = area.width as usize;

            let is_selected = self.selected == Some(idx);
            let is_hovered = self.hover_index == Some(idx);

            let style = if is_selected {
                UiStyle::HIGHLIGHT.add_modifier(Modifier::BOLD)
            } else if is_hovered {
                UiStyle::HOVER.add_modifier(Modifier::empty())
            } else {
                UiStyle::TEXT_SECONDARY
            };

            let prefix = if is_selected { "▸ " } else { "  " };
            let max_label = w.saturating_sub(3);
            let label = if item.label.len() > max_label {
                format!("{}...", &item.label[..max_label.saturating_sub(3)])
            } else {
                item.label.clone()
            };
            let text = format!("{}{}", prefix, label);
            let padding = w.saturating_sub(text.len());
            let padded = format!("{}{}", text, " ".repeat(padding));

            let line = Line::from(Span::styled(padded, style));
            buf.set_line(area.x, y, &line, area.width);
        }

        // Scroll indicators
        if start > 0 {
            buf.set_string(
                area.x + area.width.saturating_sub(2),
                area.y,
                "↑",
                UiStyle::MUTED,
            );
        }
        if start + visible < total {
            buf.set_string(
                area.x + area.width.saturating_sub(2),
                area.y + area.height.saturating_sub(1),
                "↓",
                UiStyle::MUTED,
            );
        }
    }
}
