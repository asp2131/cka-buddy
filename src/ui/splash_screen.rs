use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::app::engine::Engine;

use super::{
    button::Button,
    constants::UiStyle,
    effects,
    traits::Screen,
    ui_action::UiAction,
    ui_frame::UiFrame,
};

const TITLE: [&str; 6] = [
    " ██████╗██╗  ██╗ █████╗     ██████╗ ██╗   ██╗██████╗ ██████╗ ██╗   ██╗",
    "██╔════╝██║ ██╔╝██╔══██╗    ██╔══██╗██║   ██║██╔══██╗██╔══██╗╚██╗ ██╔╝",
    "██║     █████╔╝ ███████║    ██████╔╝██║   ██║██║  ██║██║  ██║ ╚████╔╝ ",
    "██║     ██╔═██╗ ██╔══██║    ██╔══██╗██║   ██║██║  ██║██║  ██║  ╚██╔╝  ",
    "╚██████╗██║  ██╗██║  ██║    ██████╔╝╚██████╔╝██████╔╝██████╔╝   ██║   ",
    " ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝    ╚═════╝  ╚═════╝ ╚═════╝ ╚═════╝    ╚═╝   ",
];

const STARFIELD: [&str; 3] = [
    "  ·    ✦      ·         ✧    ·        ✦       ·    ✧       ·  ",
    "     ·     ✧       ·         ✦    ·       ✧        ·    ✦     ",
    " ✧       ·    ✦       ✧   ·         ✦        ·   ✧       ·   ",
];

const QUOTES: [&str; 15] = [
    "Pods are cattle, not pets.",
    "If it's not in version control, it doesn't exist.",
    "There is no cloud, it's just someone else's computer.",
    "kubectl get pods --all-namespaces — the most typed command in history",
    "etcd: because your cluster state shouldn't be a mystery.",
    "In containers we trust, in orchestrators we believe.",
    "The best incident response is prevention.",
    "Infrastructure as code, or it didn't happen.",
    "Namespaces: because sharing is overrated.",
    "A rolling update gathers no downtime.",
    "Your cluster is only as strong as its weakest RBAC policy.",
    "Probes don't lie — your app's health is what it is.",
    "The control plane is always watching.",
    "Scheduling is an art. Resource requests are science.",
    "Every outage is a learning opportunity.",
];

const SPINNER: [&str; 4] = ["◐", "◓", "◑", "◒"];

const K8S_WHEEL: [&str; 4] = ["☸ ", " ☸", " ☸", "☸ "];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SplashItem {
    Continue,
    NewSession,
    Quit,
}

pub struct SplashScreen {
    menu: Vec<SplashItem>,
    selected: usize,
    tick: usize,
    last_rotate: Instant,
    quote_index: usize,
    first_render: bool,
}

impl SplashScreen {
    pub fn new(has_progress: bool) -> Self {
        let mut menu = vec![];
        if has_progress {
            menu.push(SplashItem::Continue);
        }
        menu.push(SplashItem::NewSession);
        menu.push(SplashItem::Quit);

        Self {
            selected: 0,
            menu,
            tick: 0,
            last_rotate: Instant::now(),
            quote_index: 0,
            first_render: true,
        }
    }

    fn menu_label(item: SplashItem) -> &'static str {
        match item {
            SplashItem::Continue => "▸ Continue",
            SplashItem::NewSession => "▸ New Session",
            SplashItem::Quit => "▸ Quit",
        }
    }
}

impl Screen for SplashScreen {
    fn update(&mut self, _engine: &Engine) -> anyhow::Result<()> {
        self.tick = self.tick.wrapping_add(1);
        if self.last_rotate.elapsed() >= Duration::from_secs(8) {
            self.quote_index = (self.quote_index + 1) % QUOTES.len();
            self.last_rotate = Instant::now();
        }
        Ok(())
    }

    fn render(
        &mut self,
        frame: &mut UiFrame<'_, '_>,
        engine: &Engine,
        area: Rect,
    ) -> anyhow::Result<()> {
        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(TITLE.len() as u16 + 2),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(self.menu.len() as u16 * 3 + 1),
            Constraint::Min(2),
            Constraint::Length(3),
        ])
        .flex(Flex::Center)
        .split(area);

        let star_idx = (self.tick / 3) % STARFIELD.len();
        let star_line = STARFIELD[star_idx];
        let star_style = if (self.tick / 6).is_multiple_of(2) {
            UiStyle::STAR_DIM
        } else {
            UiStyle::STAR_BRIGHT
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(star_line, star_style))).centered(),
            chunks[0],
        );

        let title_block = Block::bordered()
            .border_style(UiStyle::BORDER_ACTIVE)
            .style(UiStyle::PANEL_BG);
        let title_inner = title_block.inner(chunks[1]);
        frame.render_widget(title_block, chunks[1]);

        let title_lines = TITLE
            .iter()
            .map(|line| Line::from(Span::styled(*line, UiStyle::HEADER)))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(title_lines).centered().wrap(Wrap { trim: false }),
            title_inner,
        );

        if self.first_render {
            frame.push_effect("splash_title", effects::splash_title_effect(), chunks[1]);
            frame.push_effect("splash_menu", effects::splash_menu_sweep(), chunks[4]);
            self.first_render = false;
        }

        let wheel = K8S_WHEEL[self.tick % K8S_WHEEL.len()];
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(wheel, UiStyle::HIGHLIGHT),
                Span::styled(
                    " Kubernetes Exam Preparation ",
                    UiStyle::TEXT_SECONDARY.add_modifier(Modifier::ITALIC),
                ),
                Span::styled(
                    SPINNER[self.tick % SPINNER.len()],
                    UiStyle::HIGHLIGHT,
                ),
            ]))
            .centered(),
            chunks[2],
        );

        let sep_width = area.width.min(60) as usize;
        let sep = "─".repeat(sep_width);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(sep, UiStyle::BORDER))).centered(),
            chunks[3],
        );

        let menu_rows = Layout::vertical(vec![Constraint::Length(3); self.menu.len()])
            .flex(Flex::Center)
            .split(chunks[4]);
        for (idx, item) in self.menu.iter().enumerate() {
            let btn = Button::new(Self::menu_label(*item)).selected(idx == self.selected);
            frame.render_widget(btn, menu_rows[idx]);
        }

        if !engine.progress.completed.is_empty() {
            let percent = engine.readiness;
            let done = engine.progress.completed.len();
            let total = engine.steps.len();

            let bar_width = 20usize;
            let filled = (usize::from(percent) * bar_width) / 100;
            let empty = bar_width.saturating_sub(filled);

            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("  ▐{}{}▌ ", "█".repeat(filled), "░".repeat(empty)),
                            if percent >= 60 { UiStyle::OK } else { UiStyle::WARNING },
                        ),
                        Span::styled(
                            format!("{percent}% ready"),
                            UiStyle::TEXT_PRIMARY.add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(Span::styled(
                        format!("  {done}/{total} steps complete"),
                        UiStyle::TEXT_SECONDARY,
                    )),
                ])
                .centered(),
                chunks[5],
            );
        }

        let quote_area = chunks[6];
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(" \"", UiStyle::MUTED),
                    Span::styled(
                        QUOTES[self.quote_index],
                        UiStyle::TEXT_SECONDARY.add_modifier(Modifier::ITALIC),
                    ),
                    Span::styled("\" ", UiStyle::MUTED),
                ]),
            ])
            .centered()
            .wrap(Wrap { trim: true }),
            quote_area,
        );

        Ok(())
    }

    fn handle_key_events(&mut self, key_event: KeyEvent, _engine: &Engine) -> Option<UiAction> {
        match key_event.code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Some(UiAction::None)
            }
            KeyCode::Down => {
                if self.selected + 1 < self.menu.len() {
                    self.selected += 1;
                }
                Some(UiAction::None)
            }
            KeyCode::Enter => match self.menu.get(self.selected).copied() {
                Some(SplashItem::Quit) => Some(UiAction::Quit),
                Some(SplashItem::Continue) => Some(UiAction::StartSession),
                Some(SplashItem::NewSession) => Some(UiAction::NewSession),
                None => Some(UiAction::None),
            },
            _ => None,
        }
    }

    fn footer_help(&self) -> String {
        "↑/↓ Select • Enter Confirm • Ctrl+C Quit".to_string()
    }

    fn footer_spans(&self) -> Vec<(String, String)> {
        vec![
            ("↑/↓".to_string(), "Select".to_string()),
            ("Enter".to_string(), "Confirm".to_string()),
            ("Ctrl+C".to_string(), "Quit".to_string()),
        ]
    }
}
