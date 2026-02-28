use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use super::{
    callback_registry::CallbackRegistry,
    constants::UiStyle,
    traits::InteractiveWidget,
};

pub struct UiFrame<'a, 'b> {
    frame: &'a mut Frame<'b>,
    pub callback_registry: CallbackRegistry,
    hover_text_area: Rect,
}

impl<'a, 'b> UiFrame<'a, 'b> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        hover_text_area: Rect,
        mouse_position: Option<(u16, u16)>,
    ) -> Self {
        Self {
            frame,
            callback_registry: CallbackRegistry::new(mouse_position),
            hover_text_area,
        }
    }

    pub fn render_widget(&mut self, widget: impl Widget, area: Rect) {
        self.frame.render_widget(widget, area);
    }

    pub fn render_stateful_widget<W: StatefulWidget>(
        &mut self,
        widget: W,
        area: Rect,
        state: &mut W::State,
    ) {
        self.frame.render_widget(ratatui::widgets::Clear, area);
        self.frame.render_stateful_widget(widget, area, state);
    }

    pub fn render_interactive_widget(&mut self, mut widget: impl InteractiveWidget, area: Rect) {
        widget.before_rendering(area, &mut self.callback_registry);
        if self.is_hovered(area, widget.layer()) {
            let hover = widget.hover_text();
            if !hover.is_empty() {
                self.callback_registry.set_hover_text(hover);
            }
        }
        self.frame.render_widget(widget, area);
    }

    pub fn set_active_layer(&mut self, layer: usize) {
        self.callback_registry.set_active_layer(layer);
    }

    pub fn is_hovered(&self, area: Rect, layer: usize) -> bool {
        if self.callback_registry.active_layer() != layer {
            return false;
        }
        self.callback_registry.is_hovering(area)
    }

    pub fn area(&self) -> Rect {
        self.frame.area()
    }

    pub fn registry_mut(&mut self) -> &mut CallbackRegistry {
        &mut self.callback_registry
    }

    pub fn render_hover_text(&mut self) {
        let area = self.hover_text_area;
        if area.height == 0 || area.width == 0 {
            return;
        }
        let text = self
            .callback_registry
            .hover_text()
            .unwrap_or("Type commands or click actions")
            .to_string();
        let line = Line::from(vec![
            Span::styled(" ", UiStyle::MUTED),
            Span::styled(text, UiStyle::TEXT_SECONDARY),
        ]);
        self.frame.render_widget(Paragraph::new(line), area);
    }

    pub fn into_registry(self) -> CallbackRegistry {
        self.callback_registry
    }

    pub fn inner_frame(&mut self) -> &mut Frame<'b> {
        self.frame
    }
}
