use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::engine::Engine;
use crate::app::state::CompletionCard;
use crate::content::models::Step;

use super::{
    button::Button,
    clickable_list::ClickableList,
    constants::UiStyle,
    tamagotchi::KubeChanAssistant,
    traits::Screen,
    ui_frame::UiFrame,
    ui_action::UiAction,
    widgets::{
        default_block, difficulty_badge, domain_tag, ellipsize, footer_help_text, mode_badge,
        readiness_segments, step_type_badge, titled_block,
    },
};

const WIDE_TERMINAL_THRESHOLD: u16 = 120;

pub struct LearningScreen {
    pub command_input: String,
    pub status: String,
    pub output_log: Vec<String>,
    pub hint_message: Option<String>,
    pub completion_card: Option<CompletionCard>,
    pub tab_index: usize,
    pub tick: usize,
    pub pairing_command: Option<String>,
}

impl LearningScreen {
    pub fn new(status: String) -> Self {
        Self {
            command_input: String::new(),
            status,
            output_log: vec!["CKA Coach terminal started (/bin/zsh).".to_string()],
            hint_message: None,
            completion_card: None,
            tab_index: 0,
            tick: 0,
            pairing_command: None,
        }
    }
}

impl Screen for LearningScreen {
    fn update(&mut self, _engine: &Engine) -> anyhow::Result<()> {
        self.tick = self.tick.wrapping_add(1);
        Ok(())
    }

    fn render(&mut self, frame: &mut UiFrame<'_, '_>, engine: &Engine, area: Rect) -> anyhow::Result<()> {
        let pairing_height = if self.pairing_command.is_some() { 2u16 } else { 0 };

        let zones = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(pairing_height),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

        render_header(frame, zones[0], engine);

        if let Some(cmd) = &self.pairing_command {
            render_pairing_banner(frame, zones[1], cmd);
        }

        let content_area = zones[2];

        if content_area.width >= WIDE_TERMINAL_THRESHOLD {
            let split = Layout::horizontal([
                Constraint::Percentage(65),
                Constraint::Percentage(35),
            ])
            .split(content_area);

            let left = Layout::vertical([
                Constraint::Length(4),
                Constraint::Min(4),
            ])
            .split(split[0]);

            render_step_panel(frame, left[0], engine);
            render_terminal_feed(
                frame,
                left[1],
                &self.output_log,
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
            );

            render_assistant_panel(
                frame,
                split[1],
                engine.current_step(),
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
                engine.readiness,
                self.tick,
            );
        } else {
            let narrow = Layout::vertical([
                Constraint::Length(4),
                Constraint::Min(4),
            ])
            .split(content_area);

            render_step_panel(frame, narrow[0], engine);
            render_terminal_feed(
                frame,
                narrow[1],
                &self.output_log,
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
            );
        }

        render_prompt_bar(frame, zones[3], &self.command_input, &self.status, self.command_input.is_empty());
        Ok(())
    }

    fn handle_key_events(
        &mut self,
        key_event: crossterm::event::KeyEvent,
        _engine: &Engine,
    ) -> Option<UiAction> {
        use crossterm::event::KeyCode;
        match key_event.code {
            KeyCode::Enter => {
                let input = self.command_input.trim().to_string();
                self.command_input.clear();

                if input.is_empty() {
                    return Some(UiAction::None);
                }

                if let Some(command) = input.strip_prefix("! ") {
                    return Some(UiAction::ForceRunCommand(command.trim().to_string()));
                }

                if input.starts_with('/') {
                    let mut parts = input.split_whitespace();
                    let cmd = parts.next().unwrap_or_default();
                    return Some(match cmd {
                        "/quit" => UiAction::Quit,
                        "/new" => UiAction::NewSession,
                        "/next" => UiAction::NextStep,
                        "/prev" => UiAction::PrevStep,
                        "/back" => UiAction::JumpBack,
                        "/recommended" => UiAction::JumpRecommended,
                        "/verify" => UiAction::Verify,
                        "/hint" => UiAction::Hint,
                        "/suggest" => UiAction::Suggest(parts.next().and_then(|n| n.parse().ok())),
                        "/clear" => UiAction::ClearLog,
                        "/help" => UiAction::ShowHelp,
                        "/shell" => match parts.next() {
                            Some(mode) => UiAction::SetShellMode(mode.to_string()),
                            None => UiAction::ShowShellMode,
                        },
                        _ => UiAction::None,
                    });
                }

                Some(UiAction::RunCommand(input))
            }
            KeyCode::Backspace => {
                self.command_input.pop();
                self.tab_index = 0;
                Some(UiAction::None)
            }
            KeyCode::Esc => {
                self.command_input.clear();
                Some(UiAction::None)
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
                self.tab_index = 0;
                Some(UiAction::None)
            }
            _ => None,
        }
    }

    fn footer_help(&self) -> String {
        footer_help_text(&self.command_input)
    }

    fn footer_spans(&self) -> Vec<(String, String)> {
        vec![
            ("V".to_string(), "Verify".to_string()),
            ("H".to_string(), "Hint".to_string()),
            ("Enter".to_string(), "Run".to_string()),
            ("/help".to_string(), "Commands".to_string()),
        ]
    }
}

fn render_header(frame: &mut UiFrame<'_, '_>, area: Rect, engine: &Engine) {
    let step = engine.current_step();
    let block = default_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (bar_on, bar_off) = readiness_segments(engine.readiness, 12);

    let step_chip = format!(
        "[STEP {:02}/{:02}]",
        engine.current_index + 1,
        engine.steps.len()
    );

    let title = ellipsize(&step.title, inner.width.saturating_sub(4) as usize);

    let row1 = Line::from(vec![
        Span::styled(" CKA BUDDY ", UiStyle::HEADER.add_modifier(Modifier::BOLD)),
        Span::styled(" [", UiStyle::MUTED),
        Span::styled(bar_on, UiStyle::OK),
        Span::styled(bar_off, UiStyle::MUTED),
        Span::styled("] ", UiStyle::MUTED),
        Span::styled(format!("{:>3}%  ", engine.readiness), UiStyle::TEXT_PRIMARY),
        Span::styled(step_chip, UiStyle::HIGHLIGHT),
        Span::raw(" "),
        step_type_badge(&step.step_type),
        Span::raw(" "),
        difficulty_badge(&step.difficulty),
    ]);

    let mut row2_spans = vec![
        Span::raw("  "),
        Span::styled(title, UiStyle::TEXT_PRIMARY.add_modifier(Modifier::BOLD)),
        Span::raw("  "),
    ];
    for domain in &step.domains {
        row2_spans.push(domain_tag(domain));
        row2_spans.push(Span::raw(" "));
    }

    let header = vec![row1, Line::from(row2_spans)];
    frame.render_widget(Paragraph::new(header), inner);
}

fn render_pairing_banner(frame: &mut UiFrame<'_, '_>, area: Rect, cmd: &str) {
    if area.height == 0 {
        return;
    }
    let lines = vec![
        Line::from(vec![
            Span::styled(" ⚡ PAIR ", UiStyle::PAIRING_CMD.add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {cmd} "), UiStyle::COMMAND),
        ]),
        Line::from(Span::styled(
            "   Run the above command in another terminal to connect",
            UiStyle::MUTED,
        )),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_step_panel(frame: &mut UiFrame<'_, '_>, area: Rect, engine: &Engine) {
    let step = engine.current_step();
    let block = Block::bordered()
        .border_style(UiStyle::ACCENT_BORDER)
        .title(Span::styled(" Objective ", UiStyle::HEADER));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled(" ☸ ", UiStyle::HIGHLIGHT),
            Span::styled(
                ellipsize(&step.objective, inner.width.saturating_sub(6) as usize),
                UiStyle::TEXT_PRIMARY.add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!(
                    "   {} min │ {} │ {}",
                    step.timebox_min,
                    step.difficulty,
                    step.domains.join(", ")
                ),
                UiStyle::MUTED,
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);

    if !step.run_commands.is_empty() {
        let list_y = inner.y.saturating_add(2);
        let available = inner.bottom().saturating_sub(list_y);
        let rows = step.run_commands.len().min(5).min(available as usize);
        if rows > 0 {
            let list_area = Rect::new(inner.x, list_y, inner.width, rows as u16);
            let items: Vec<_> = step.run_commands.iter().take(rows).cloned().collect();
            let actions: Vec<_> = items
                .iter()
                .map(|cmd| UiAction::SetCommandInput(cmd.clone()))
                .collect();
            let list = ClickableList::new(items, actions)
                .style(UiStyle::COMMAND)
                .marker("   ");
            frame.render_interactive_widget(list, list_area);
        }
    }
}

fn render_terminal_feed(
    frame: &mut UiFrame<'_, '_>,
    area: Rect,
    output_log: &[String],
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
) {
    let block = titled_block("Terminal");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    let max_lines = 90usize;
    let start = output_log.len().saturating_sub(max_lines);
    for entry in &output_log[start..] {
        lines.push(Line::from(Span::styled(
            format!("  {entry}"),
            UiStyle::TEXT_SECONDARY,
        )));
    }

    if let Some(hint) = hint_message {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ● Hint: ", UiStyle::WARNING.add_modifier(Modifier::BOLD)),
            Span::styled(
                hint.to_string(),
                UiStyle::WARNING.add_modifier(Modifier::ITALIC),
            ),
        ]));
    }

    if let Some(card) = completion_card {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ✓ ", UiStyle::OK.add_modifier(Modifier::BOLD)),
            Span::styled(card.done.clone(), UiStyle::OK.add_modifier(Modifier::BOLD)),
        ]));
        for item in &card.what_changed {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(item.clone(), UiStyle::OK),
            ]));
        }
        if !card.next_commands.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("    Next: ", UiStyle::MUTED),
                Span::styled(card.next_commands.join(" | "), UiStyle::MUTED),
            ]));
        }
    }

    let scroll = lines.len().saturating_sub(inner.height as usize) as u16;
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        inner,
    );
}

fn render_assistant_panel(
    frame: &mut UiFrame<'_, '_>,
    area: Rect,
    step: &Step,
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
    readiness: u8,
    tick: usize,
) {
    let block = Block::bordered()
        .border_style(UiStyle::ACCENT_BORDER)
        .title(Span::styled(" ☸ Kube-chan ", UiStyle::HEADER));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let has_completion = completion_card.is_some();
    let assistant = KubeChanAssistant::new(
        readiness,
        tick,
        &step.objective,
        &step.domains,
        &step.difficulty,
        step.fallback_hint.as_deref(),
        &step.run_commands,
        hint_message,
        has_completion,
    );
    frame.render_widget(assistant, inner);
}

#[allow(clippy::too_many_arguments)]
fn render_prompt_bar(
    frame: &mut UiFrame<'_, '_>,
    area: Rect,
    command_input: &str,
    status: &str,
    enable_hotkeys: bool,
) {
    if area.height == 0 {
        return;
    }

    let (badge_label, badge_style) = mode_badge(status, command_input);

    if area.height == 1 {
        let line = Line::from(vec![
            Span::styled(" ❯ ", UiStyle::PROMPT.add_modifier(Modifier::BOLD)),
            Span::styled(command_input.to_string(), UiStyle::TEXT_PRIMARY),
            Span::raw("  "),
            Span::styled(format!("[{badge_label}]"), badge_style),
        ]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);

    let status_line = Line::from(vec![
        Span::styled(" ", UiStyle::MUTED),
        Span::styled(ellipsize(status, area.width.saturating_sub(4) as usize), UiStyle::TEXT_SECONDARY),
    ]);
    frame.render_widget(
        Paragraph::new(status_line).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(UiStyle::ACCENT_BORDER),
        ),
        rows[0],
    );

    let mut prompt_spans = vec![
        Span::styled(" ❯ ", UiStyle::PROMPT.add_modifier(Modifier::BOLD)),
        Span::styled(command_input.to_string(), UiStyle::TEXT_PRIMARY),
    ];

    let remaining = area.width.saturating_sub(4 + command_input.len() as u16);
    if remaining > 20 {
        prompt_spans.push(Span::raw("  "));
        prompt_spans.push(Span::styled(format!("[{badge_label}]"), badge_style));
    }

    frame.render_widget(Paragraph::new(Line::from(prompt_spans)), rows[1]);

    if enable_hotkeys {
        let verify = Button::new("Verify")
            .on_click(UiAction::Verify)
            .hover_text(Text::from("Run verification checks"))
            .hotkey(KeyCode::Char('v'));
        let hint_btn = Button::new("Hint")
            .on_click(UiAction::Hint)
            .hover_text(Text::from("Get a contextual hint"))
            .hotkey(KeyCode::Char('h'));
        frame.render_interactive_widget(verify, Rect::new(0, 0, 0, 0));
        frame.render_interactive_widget(hint_btn, Rect::new(0, 0, 0, 0));
    }
}
