use std::time::Instant;

use ratatui::layout::Rect;
use ratatui::style::Color;
use tachyonfx::{
    fx, fx::Glitch, Duration, EffectManager, IntoEffect, Interpolation, Motion,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectId {
    #[default]
    SplashTitle,
    SplashSubtitle,
    SplashMenu,
    SplashQuoteFade,
    ScreenTransition,
    VerifyRunning,
    VerifyResult,
    StepTransition,
    PopupOpen,
    PopupClose,
    TamagotchiPulse,
    TamagotchiMood,
    TamagotchiEvolve,
    DomainWash,
    AmbientBorder,
    CommandFeedback,
}

pub struct EffectEngine {
    pub manager: EffectManager<EffectId>,
    last_tick: Instant,
}

impl Default for EffectEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectEngine {
    pub fn new() -> Self {
        Self {
            manager: EffectManager::default(),
            last_tick: Instant::now(),
        }
    }

    pub fn elapsed(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;
        Duration::from_millis(elapsed.as_millis() as u32)
    }

    pub fn process(&mut self, buf: &mut ratatui::buffer::Buffer, area: Rect) {
        let elapsed = self.elapsed();
        self.manager.process_effects(elapsed, buf, area);
    }

    pub fn is_running(&self) -> bool {
        self.manager.is_running()
    }

    // --- Splash Effects ---

    pub fn trigger_splash_title(&mut self, area: Rect) {
        let effect = fx::parallel(&[
            fx::coalesce((800, Interpolation::QuadOut))
                .with_pattern(tachyonfx::pattern::RadialPattern::center()),
            fx::fade_from_fg(Color::Rgb(14, 18, 28), (600, Interpolation::SineOut)),
        ])
        .with_area(area);
        self.manager.add_unique_effect(EffectId::SplashTitle, effect);
    }

    pub fn trigger_splash_subtitle(&mut self, area: Rect) {
        let effect = fx::sequence(&[
            fx::sleep(400),
            fx::sweep_in(
                Motion::LeftToRight,
                8,
                2,
                Color::Rgb(14, 18, 28),
                (500, Interpolation::QuadOut),
            ),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::SplashSubtitle, effect);
    }

    pub fn trigger_splash_menu(&mut self, area: Rect) {
        let effect = fx::sequence(&[
            fx::sleep(600),
            fx::slide_in(
                Motion::DownToUp,
                3,
                0,
                Color::Rgb(14, 18, 28),
                (400, Interpolation::QuadOut),
            ),
        ])
        .with_area(area);
        self.manager.add_unique_effect(EffectId::SplashMenu, effect);
    }

    pub fn trigger_quote_crossfade(&mut self, area: Rect) {
        let effect = fx::sequence(&[
            fx::dissolve((300, Interpolation::SineIn)),
            fx::coalesce((300, Interpolation::SineOut)),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::SplashQuoteFade, effect);
    }

    // --- Screen Transitions ---

    pub fn trigger_screen_transition(&mut self, area: Rect) {
        let bg = Color::Rgb(14, 18, 28);
        let effect = fx::parallel(&[
            fx::dissolve((400, Interpolation::QuadIn)),
            fx::fade_to(bg, bg, (400, Interpolation::SineIn)),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::ScreenTransition, effect);
    }

    pub fn trigger_screen_appear(&mut self, area: Rect) {
        let bg = Color::Rgb(14, 18, 28);
        let effect = fx::parallel(&[
            fx::coalesce((500, Interpolation::QuadOut)),
            fx::fade_from(bg, bg, (500, Interpolation::SineOut)),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::ScreenTransition, effect);
    }

    // --- Step Navigation ---

    pub fn trigger_step_next(&mut self, area: Rect) {
        let bg = Color::Rgb(14, 18, 28);
        let effect = fx::sweep_in(
            Motion::RightToLeft,
            10,
            3,
            bg,
            (350, Interpolation::QuadOut),
        )
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::StepTransition, effect);
    }

    pub fn trigger_step_prev(&mut self, area: Rect) {
        let bg = Color::Rgb(14, 18, 28);
        let effect = fx::sweep_in(
            Motion::LeftToRight,
            10,
            3,
            bg,
            (350, Interpolation::QuadOut),
        )
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::StepTransition, effect);
    }

    // --- Verification Drama ---

    pub fn trigger_verify_running(&mut self, area: Rect) {
        let effect = fx::repeating(fx::ping_pong(fx::hsl_shift_fg(
            [0.0, -15.0, -8.0],
            (300, Interpolation::SineInOut),
        )))
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::VerifyRunning, effect);
    }

    pub fn cancel_verify_running(&mut self) {
        self.manager.cancel_unique_effect(EffectId::VerifyRunning);
    }

    pub fn trigger_verify_pass(&mut self, area: Rect) {
        self.cancel_verify_running();
        let green = Color::Rgb(80, 220, 120);
        let effect = fx::sequence(&[
            fx::parallel(&[
                fx::fade_from_fg(green, (400, Interpolation::QuadOut)),
                fx::lighten_fg(0.3, (200, Interpolation::SineOut)),
            ]),
            fx::fade_from_fg(green, (600, Interpolation::SineIn)),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::VerifyResult, effect);
    }

    pub fn trigger_verify_fail(&mut self, area: Rect) {
        self.cancel_verify_running();
        let red = Color::Rgb(240, 70, 70);
        let glitch = Glitch::builder()
            .cell_glitch_ratio(0.08)
            .action_start_delay_ms(0..50)
            .action_ms(30..80)
            .build()
            .into_effect();
        let effect = fx::sequence(&[
            fx::parallel(&[
                fx::fade_from_fg(red, (200, Interpolation::QuadOut)),
                fx::with_duration(Duration::from_millis(200), glitch),
            ]),
            fx::fade_from_fg(red, (400, Interpolation::SineIn)),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::VerifyResult, effect);
    }

    // --- Popup Effects ---

    pub fn trigger_popup_open(&mut self, area: Rect) {
        let effect = fx::parallel(&[
            fx::coalesce((300, Interpolation::QuadOut))
                .with_pattern(tachyonfx::pattern::RadialPattern::center()),
            fx::fade_from_fg(Color::Rgb(14, 18, 28), (250, Interpolation::SineOut)),
        ])
        .with_area(area);
        self.manager.add_unique_effect(EffectId::PopupOpen, effect);
    }

    pub fn trigger_popup_close(&mut self, area: Rect) {
        let effect = fx::parallel(&[
            fx::dissolve((200, Interpolation::QuadIn)),
            fx::fade_to_fg(Color::Rgb(14, 18, 28), (200, Interpolation::SineIn)),
        ])
        .with_area(area);
        self.manager.add_unique_effect(EffectId::PopupClose, effect);
    }

    // --- Tamagotchi Effects ---

    pub fn trigger_tamagotchi_pulse(&mut self, area: Rect) {
        let effect = fx::repeating(fx::ping_pong(fx::hsl_shift_fg(
            [0.0, 10.0, 8.0],
            (1500, Interpolation::SineInOut),
        )))
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::TamagotchiPulse, effect);
    }

    pub fn trigger_tamagotchi_happy(&mut self, area: Rect) {
        let effect = fx::sequence(&[
            fx::lighten_fg(0.25, (200, Interpolation::QuadOut)),
            fx::lighten_fg(0.25, (400, Interpolation::SineIn)).reversed(),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::TamagotchiMood, effect);
    }

    pub fn trigger_tamagotchi_sick(&mut self, area: Rect) {
        let glitch = Glitch::builder()
            .cell_glitch_ratio(0.05)
            .action_start_delay_ms(0..30)
            .action_ms(50..120)
            .build()
            .into_effect();
        let effect = fx::repeating(fx::sequence(&[
            fx::with_duration(Duration::from_millis(300), glitch),
            fx::sleep(2000),
        ]))
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::TamagotchiMood, effect);
    }

    pub fn trigger_tamagotchi_evolve(&mut self, area: Rect) {
        let effect = fx::parallel(&[
            fx::coalesce((600, Interpolation::QuadOut))
                .with_pattern(tachyonfx::pattern::RadialPattern::center()),
            fx::hsl_shift_fg([120.0, 30.0, 15.0], (600, Interpolation::SineOut)),
        ])
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::TamagotchiEvolve, effect);
    }

    // --- Domain Color Wash ---

    pub fn trigger_domain_wash(&mut self, area: Rect, domain: &str) {
        let hue_shift = match domain.to_lowercase().as_str() {
            d if d.contains("storage") => [270.0, 10.0, 5.0],
            d if d.contains("network") => [200.0, 10.0, 5.0],
            d if d.contains("workload") || d.contains("scheduling") => [30.0, 10.0, 5.0],
            d if d.contains("cluster") => [150.0, 10.0, 5.0],
            d if d.contains("security") || d.contains("rbac") => [340.0, 10.0, 5.0],
            d if d.contains("troubleshoot") => [45.0, 10.0, 5.0],
            _ => return,
        };
        let effect = fx::hsl_shift_fg(hue_shift, (500, Interpolation::SineInOut)).with_area(area);
        self.manager
            .add_unique_effect(EffectId::DomainWash, effect);
    }

    // --- Command Feedback ---

    pub fn trigger_command_sent(&mut self, area: Rect) {
        let effect = fx::sweep_in(
            Motion::LeftToRight,
            6,
            2,
            Color::Rgb(14, 18, 28),
            (250, Interpolation::QuadOut),
        )
        .with_area(area);
        self.manager
            .add_unique_effect(EffectId::CommandFeedback, effect);
    }
}
