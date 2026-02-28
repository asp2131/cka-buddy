use crossterm::event::{KeyCode, MouseEventKind};
use ratatui::layout::Rect;

use super::ui_action::UiAction;

#[derive(Debug, Clone)]
struct MouseCallback {
    kind: MouseEventKind,
    area: Option<Rect>,
    action: UiAction,
    layer: usize,
}

#[derive(Debug, Clone)]
struct KeyCallback {
    key: KeyCode,
    action: UiAction,
    layer: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CallbackRegistry {
    mouse_callbacks: Vec<MouseCallback>,
    keyboard_callbacks: Vec<KeyCallback>,
    mouse_position: Option<(u16, u16)>,
    active_layer: usize,
    hover_text: Option<String>,
}

impl CallbackRegistry {
    pub fn new(mouse_position: Option<(u16, u16)>) -> Self {
        Self {
            mouse_position,
            ..Default::default()
        }
    }

    pub fn register_mouse_callback(
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

    pub fn register_keyboard_callback(&mut self, key: KeyCode, action: UiAction, layer: usize) {
        self.keyboard_callbacks.push(KeyCallback { key, action, layer });
    }

    pub fn is_hovering(&self, area: Rect) -> bool {
        if let Some((x, y)) = self.mouse_position {
            area.x <= x
                && x < area.x + area.width
                && area.y <= y
                && y < area.y + area.height
        } else {
            false
        }
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

    pub fn set_hover_text(&mut self, text: String) {
        self.hover_text = Some(text);
    }

    pub fn hover_text(&self) -> Option<&str> {
        self.hover_text.as_deref()
    }

    pub fn resolve_mouse_event(
        &self,
        kind: MouseEventKind,
        x: u16,
        y: u16,
    ) -> Option<UiAction> {
        for cb in self.mouse_callbacks.iter().rev() {
            if cb.layer != self.active_layer {
                continue;
            }
            let kind_matches = std::mem::discriminant(&cb.kind) == std::mem::discriminant(&kind);
            if !kind_matches {
                continue;
            }
            if let Some(area) = cb.area {
                if area.x <= x && x < area.x + area.width && area.y <= y && y < area.y + area.height {
                    return Some(cb.action.clone());
                }
            } else {
                return Some(cb.action.clone());
            }
        }
        None
    }

    pub fn resolve_key_event(&self, key: KeyCode) -> Option<UiAction> {
        for cb in self.keyboard_callbacks.iter().rev() {
            if cb.layer != self.active_layer {
                continue;
            }
            if cb.key == key {
                return Some(cb.action.clone());
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.mouse_callbacks.clear();
        self.keyboard_callbacks.clear();
        self.hover_text = None;
    }
}
