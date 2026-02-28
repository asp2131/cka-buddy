use crossterm::event::{KeyCode, MouseEventKind};
use ratatui::layout::Rect;

use super::ui_action::UiAction;

#[derive(Debug, Clone)]
pub struct CallbackRegistry {
    mouse_callbacks: Vec<MouseCallback>,
    keyboard_callbacks: Vec<(KeyCode, UiAction)>,
    mouse_position: Option<(u16, u16)>,
    active_layer: usize,
}

#[derive(Debug, Clone)]
struct MouseCallback {
    kind: MouseEventKind,
    area: Option<Rect>,
    action: UiAction,
    layer: usize,
}

impl Default for CallbackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackRegistry {
    pub fn new() -> Self {
        Self {
            mouse_callbacks: Vec::new(),
            keyboard_callbacks: Vec::new(),
            mouse_position: None,
            active_layer: 0,
        }
    }

    pub fn with_mouse_position(mouse_position: Option<(u16, u16)>) -> Self {
        let mut registry = Self::new();
        registry.mouse_position = mouse_position;
        registry
    }

    pub fn register_mouse_callback(
        &mut self,
        kind: MouseEventKind,
        area: Option<Rect>,
        action: UiAction,
    ) {
        self.register_mouse_callback_on_layer(kind, area, action, 0);
    }

    pub fn register_mouse_callback_on_layer(
        &mut self,
        kind: MouseEventKind,
        area: Option<Rect>,
        action: UiAction,
        layer: usize,
    ) {
        self.mouse_callbacks.push(MouseCallback {
            kind,
            area,
            action,
            layer,
        });
    }

    pub fn register_keyboard_callback(&mut self, key: KeyCode, action: UiAction) {
        self.keyboard_callbacks.push((key, action));
    }

    pub fn is_hovering(&self, area: Rect) -> bool {
        let Some((x, y)) = self.mouse_position else {
            return false;
        };

        x >= area.x
            && x < area.x.saturating_add(area.width)
            && y >= area.y
            && y < area.y.saturating_add(area.height)
    }

    pub fn set_mouse_position(&mut self, x: u16, y: u16) {
        self.mouse_position = Some((x, y));
    }

    pub fn mouse_position(&self) -> Option<(u16, u16)> {
        self.mouse_position
    }

    pub fn set_active_layer(&mut self, layer: usize) {
        self.active_layer = layer;
    }

    pub fn active_layer(&self) -> usize {
        self.active_layer
    }

    pub fn resolve_mouse_event(&self, kind: MouseEventKind, x: u16, y: u16) -> Option<UiAction> {
        self.mouse_callbacks
            .iter()
            .rev()
            .find(|cb| {
                cb.layer == self.active_layer
                    && cb.kind == kind
                    && cb
                        .area
                        .map(|area| {
                            x >= area.x
                                && x < area.x.saturating_add(area.width)
                                && y >= area.y
                                && y < area.y.saturating_add(area.height)
                        })
                        .unwrap_or(true)
            })
            .map(|cb| cb.action.clone())
    }

    pub fn resolve_key_event(&self, key: KeyCode) -> Option<UiAction> {
        self.keyboard_callbacks
            .iter()
            .rev()
            .find(|(registered, _)| *registered == key)
            .map(|(_, action)| action.clone())
    }

    pub fn clear(&mut self) {
        self.mouse_callbacks.clear();
        self.keyboard_callbacks.clear();
    }
}
