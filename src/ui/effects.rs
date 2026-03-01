use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
};

fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    match (from, to) {
        (Color::Rgb(fr, fg, fb), Color::Rgb(tr, tg, tb)) => Color::Rgb(
            (fr as f32 + (tr as f32 - fr as f32) * t) as u8,
            (fg as f32 + (tg as f32 - fg as f32) * t) as u8,
            (fb as f32 + (tb as f32 - fb as f32) * t) as u8,
        ),
        _ => if t > 0.5 { to } else { from },
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(2)
}

fn ease_in_out_sine(t: f32) -> f32 {
    -(((t * std::f32::consts::PI).cos() - 1.0) / 2.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    FadeFromBlack,
    SweepRight,
    CoalesceIn,
    Dissolve,
    PulseGlow,
}

#[derive(Debug)]
pub struct VisualEffect {
    kind: EffectKind,
    duration: Duration,
    elapsed: Duration,
    color: Color,
}

impl VisualEffect {
    pub fn fade_from_black(duration_ms: u64) -> Self {
        Self {
            kind: EffectKind::FadeFromBlack,
            duration: Duration::from_millis(duration_ms),
            elapsed: Duration::ZERO,
            color: Color::Rgb(14, 18, 28),
        }
    }

    pub fn sweep_right(duration_ms: u64) -> Self {
        Self {
            kind: EffectKind::SweepRight,
            duration: Duration::from_millis(duration_ms),
            elapsed: Duration::ZERO,
            color: Color::Rgb(14, 18, 28),
        }
    }

    pub fn coalesce(duration_ms: u64) -> Self {
        Self {
            kind: EffectKind::CoalesceIn,
            duration: Duration::from_millis(duration_ms),
            elapsed: Duration::ZERO,
            color: Color::Rgb(14, 18, 28),
        }
    }

    pub fn done(&self) -> bool {
        self.elapsed >= self.duration
    }

    fn alpha(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    pub fn process(&mut self, dt: Duration, buf: &mut Buffer, area: Rect) {
        self.elapsed = self.elapsed.saturating_add(dt);
        let raw_alpha = self.alpha();

        match self.kind {
            EffectKind::FadeFromBlack => {
                let t = ease_out_cubic(raw_alpha);
                for y in area.y..area.bottom() {
                    for x in area.x..area.right() {
                        let cell = &mut buf[(x, y)];
                        let fg = cell.fg;
                        cell.set_fg(lerp_color(self.color, fg, t));
                        let bg = cell.bg;
                        cell.set_bg(lerp_color(self.color, bg, t));
                    }
                }
            }
            EffectKind::SweepRight => {
                let t = ease_out_quad(raw_alpha);
                let sweep_pos = (t * area.width as f32) as u16;
                let gradient_len = 8u16;
                for y in area.y..area.bottom() {
                    for x in area.x..area.right() {
                        let col = x.saturating_sub(area.x);
                        if col > sweep_pos + gradient_len {
                            let cell = &mut buf[(x, y)];
                            cell.set_fg(self.color);
                            cell.set_char(' ');
                        } else if col > sweep_pos {
                            let grad_t =
                                1.0 - ((col - sweep_pos) as f32 / gradient_len as f32);
                            let cell = &mut buf[(x, y)];
                            let fg = cell.fg;
                            cell.set_fg(lerp_color(self.color, fg, grad_t));
                        }
                    }
                }
            }
            EffectKind::CoalesceIn => {
                let t = ease_out_cubic(raw_alpha);
                let threshold = (t * 255.0) as u8;
                let mut hash_seed = 42u32;
                for y in area.y..area.bottom() {
                    for x in area.x..area.right() {
                        hash_seed = hash_seed.wrapping_mul(1103515245).wrapping_add(12345);
                        let cell_hash = ((hash_seed >> 16) & 0xFF) as u8;
                        if cell_hash > threshold {
                            let cell = &mut buf[(x, y)];
                            cell.set_char(' ');
                            cell.set_fg(self.color);
                        }
                    }
                }
            }
            EffectKind::Dissolve => {
                let t = ease_out_quad(raw_alpha);
                let threshold = ((1.0 - t) * 255.0) as u8;
                let mut hash_seed = 77u32;
                for y in area.y..area.bottom() {
                    for x in area.x..area.right() {
                        hash_seed = hash_seed.wrapping_mul(1103515245).wrapping_add(12345);
                        let cell_hash = ((hash_seed >> 16) & 0xFF) as u8;
                        if cell_hash > threshold {
                            let cell = &mut buf[(x, y)];
                            cell.set_char(' ');
                            cell.set_fg(self.color);
                        }
                    }
                }
            }
            EffectKind::PulseGlow => {
                let t = ease_in_out_sine(raw_alpha);
                for y in area.y..area.bottom() {
                    for x in area.x..area.right() {
                        let cell = &mut buf[(x, y)];
                        let fg = cell.fg;
                        cell.set_fg(lerp_color(fg, Color::Rgb(0, 220, 230), t * 0.3));
                    }
                }
            }
        }
    }
}

pub fn splash_title_effect() -> VisualEffect {
    VisualEffect::fade_from_black(900)
}

pub fn splash_menu_sweep() -> VisualEffect {
    VisualEffect::sweep_right(700)
}

pub fn screen_transition_in() -> VisualEffect {
    VisualEffect::fade_from_black(500)
}

pub fn header_sweep() -> VisualEffect {
    VisualEffect::sweep_right(500)
}

pub fn popup_appear() -> VisualEffect {
    VisualEffect::coalesce(400)
}

pub struct EffectState {
    effects: Vec<(String, VisualEffect, Rect)>,
    last_tick: Instant,
}

impl Default for EffectState {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectState {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            last_tick: Instant::now(),
        }
    }

    pub fn push(&mut self, id: impl Into<String>, effect: VisualEffect, area: Rect) {
        let id = id.into();
        self.effects.retain(|(eid, _, _)| *eid != id);
        self.effects.push((id, effect, area));
    }

    pub fn has(&self, id: &str) -> bool {
        self.effects.iter().any(|(eid, _, _)| eid == id)
    }

    pub fn process_all(&mut self, buf: &mut Buffer) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick);
        self.last_tick = now;

        for (_, effect, area) in &mut self.effects {
            effect.process(dt, buf, *area);
        }

        self.effects.retain(|(_, effect, _)| !effect.done());
    }

    pub fn clear(&mut self) {
        self.effects.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}
