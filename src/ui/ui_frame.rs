use ratatui::{
    Frame,
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
};

use super::{callback_registry::CallbackRegistry, effects::VisualEffect, traits::InteractiveWidget};

pub struct PendingEffect {
    pub id: String,
    pub effect: VisualEffect,
    pub area: Rect,
}

pub struct UiFrame<'a, 'b> {
    frame: &'a mut Frame<'b>,
    callback_registry: CallbackRegistry,
    hover_text_area: Rect,
    hover_text: Option<String>,
    pending_effects: Vec<PendingEffect>,
}

impl<'a, 'b> UiFrame<'a, 'b> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        hover_text_area: Rect,
        mouse_position: Option<(u16, u16)>,
    ) -> Self {
        Self {
            frame,
            callback_registry: CallbackRegistry::with_mouse_position(mouse_position),
            hover_text_area,
            hover_text: None,
            pending_effects: Vec::new(),
        }
    }

    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        self.frame.render_widget(widget, area);
    }

    pub fn render_stateful_widget<W>(&mut self, widget: W, area: Rect, state: &mut W::State)
    where
        W: StatefulWidget,
    {
        self.frame.render_stateful_widget(widget, area, state);
    }

    pub fn render_interactive_widget<W: InteractiveWidget>(&mut self, mut widget: W, area: Rect) {
        let layer = widget.layer();
        let is_hovered = self.is_hovered(area, layer);
        if is_hovered && self.hover_text.is_none() {
            self.hover_text = Some(widget.hover_text().to_string());
        }

        widget.before_rendering(area, &mut self.callback_registry);
        widget.render(area, self.frame.buffer_mut());
    }

    pub fn render_stateful_interactive_widget<W>(
        &mut self,
        mut widget: W,
        area: Rect,
        state: &mut W::State,
    ) where
        W: StatefulWidget + InteractiveWidget,
    {
        let layer = widget.layer();
        let is_hovered = self.is_hovered(area, layer);
        if is_hovered && self.hover_text.is_none() {
            self.hover_text = Some(widget.hover_text().to_string());
        }

        widget.before_rendering(area, &mut self.callback_registry);
        self.frame.render_stateful_widget(widget, area, state);
    }

    pub fn set_active_layer(&mut self, layer: usize) {
        self.callback_registry.set_active_layer(layer);
    }

    pub fn is_hovered(&self, area: Rect, layer: usize) -> bool {
        self.callback_registry.active_layer() == layer && self.callback_registry.is_hovering(area)
    }

    pub fn area(&self) -> Rect {
        self.frame.area()
    }

    pub fn hover_text_area(&self) -> Rect {
        self.hover_text_area
    }

    pub fn registry_mut(&mut self) -> &mut CallbackRegistry {
        &mut self.callback_registry
    }

    pub fn hover_text(&self) -> Option<&str> {
        self.hover_text.as_deref()
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.frame.buffer_mut()
    }

    pub fn push_effect(&mut self, id: impl Into<String>, effect: VisualEffect, area: Rect) {
        self.pending_effects.push(PendingEffect {
            id: id.into(),
            effect,
            area,
        });
    }

    pub fn has_pending_effect(&self, id: &str) -> bool {
        self.pending_effects.iter().any(|e| e.id == id)
    }

    pub fn drain_pending_effects(&mut self) -> Vec<PendingEffect> {
        std::mem::take(&mut self.pending_effects)
    }

    pub fn into_registry(self) -> (CallbackRegistry, Vec<PendingEffect>) {
        (self.callback_registry, self.pending_effects)
    }
}
