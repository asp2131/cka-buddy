use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::app::engine::Engine;

use super::{
    button::Button,
    constants::UiStyle,
    traits::Screen,
    ui_action::UiAction,
};

const TITLE: [&str; 6] = [
    " ██████╗██╗  ██╗ █████╗     ██████╗ ██╗   ██╗██████╗ ██████╗ ██╗   ██╗",
    "██╔════╝██║ ██╔╝██╔══██╗    ██╔══██╗██║   ██║██╔══██╗██╔══██╗╚██╗ ██╔╝",
    "██║     █████╔╝ ███████║    ██████╔╝██║   ██║██║  ██║██║  ██║ ╚████╔╝ ",
    "██║     ██╔═██╗ ██╔══██║    ██╔══██╗██║   ██║██║  ██║██║  ██║  ╚██╔╝  ",
    "╚██████╗██║  ██╗██║  ██║    ██████╔╝╚██████╔╝██████╔╝██████╔╝   ██║   ",
    " ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝    ╚═════╝  ╚═════╝ ╚═════╝ ╚═════╝    ╚═╝   ",
];

const QUOTES: [&str; 15] = [
    "Pods are cattle, not pets.",
    "If it's not in version control, it doesn't exist.",
    "There is no cloud, it's just someone else's computer.",
    "kubectl get pods --all-namespaces - the most typed command in history",
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
        }
    }

    fn menu_label(item: SplashItem) -> &'static str {
        match item {
            SplashItem::Continue => "Continue",
            SplashItem::NewSession => "New Session",
            SplashItem::Quit => "Quit",
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

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) -> anyhow::Result<()> {
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(TITLE.len() as u16 + 2),
            Constraint::Length(1),
            Constraint::Length(self.menu.len() as u16 * 3),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .flex(Flex::Center)
        .split(area);

        let title_block = Block::bordered().border_style(UiStyle::BORDER);
        let title_lines = TITLE
            .iter()
            .map(|line| Line::from(Span::styled(*line, UiStyle::HEADER)))
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(title_lines)
                .block(title_block)
                .centered()
                .wrap(Wrap { trim: false }),
            chunks[1],
        );

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Kubernetes Exam Preparation", UiStyle::TEXT_SECONDARY),
                Span::raw("  "),
                Span::styled(SPINNER[self.tick % SPINNER.len()], UiStyle::HIGHLIGHT),
            ]))
            .centered(),
            chunks[2],
        );

        let menu_rows = Layout::vertical(vec![Constraint::Length(3); self.menu.len()])
            .flex(Flex::Center)
            .split(chunks[3]);
        for (idx, item) in self.menu.iter().enumerate() {
            frame.render_widget(
                Button::new(Self::menu_label(*item)).selected(idx == self.selected),
                menu_rows[idx],
            );
        }

        if !engine.progress.completed.is_empty() {
            let percent = engine.readiness;
            let done = engine.progress.completed.len();
            let total = engine.steps.len();
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(format!("{percent}% ready"), UiStyle::OK),
                    Span::styled(format!(" | {done}/{total} steps complete"), UiStyle::TEXT_SECONDARY),
                ]))
                .centered(),
                chunks[4],
            );
        }

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("\"", UiStyle::MUTED),
                Span::styled(QUOTES[self.quote_index], UiStyle::TEXT_SECONDARY),
                Span::styled("\"", UiStyle::MUTED),
            ]))
            .centered()
            .wrap(Wrap { trim: true }),
            chunks[5],
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
                Some(SplashItem::Continue | SplashItem::NewSession) => Some(UiAction::StartSession),
                None => Some(UiAction::None),
            },
            _ => None,
        }
    }

    fn footer_help(&self) -> String {
        "↑/↓ Select • Enter Confirm".to_string()
    }
}
