use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::engine::Engine;
use crate::app::state::CompletionCard;
use crate::content::command_info::command_blurb;

use super::{
    constants::UiStyle,
    traits::Screen,
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
        }
    }
}

impl Screen for LearningScreen {
    fn update(&mut self, _engine: &Engine) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) -> anyhow::Result<()> {
        let zones = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

        render_header(frame, zones[0], engine);

        if zones[1].width >= WIDE_TERMINAL_THRESHOLD {
            let split =
                Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(zones[1]);
            render_main_feed(
                frame,
                split[0],
                engine,
                &self.status,
                &self.output_log,
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
            );
            render_activity_rail(
                frame,
                split[1],
                engine.current_step().run_commands.as_slice(),
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
                &self.status,
            );
        } else {
            render_main_feed(
                frame,
                zones[1],
                engine,
                &self.status,
                &self.output_log,
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
            );
        }

        render_command_bar(frame, zones[2], &self.command_input, &self.status);
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
}

fn render_header(frame: &mut Frame, area: Rect, engine: &Engine) {
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

fn render_main_feed(
    frame: &mut Frame,
    area: Rect,
    engine: &Engine,
    _status: &str,
    output_log: &[String],
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
) {
    let step = engine.current_step();
    let command_count = step.run_commands.len().min(5) as u16;
    // Each command now takes 2 lines (blurb + command), so double the count
    let step_panel_height =
        (6 + command_count * 2).clamp(6, area.height.saturating_sub(8).max(6));

    let chunks =
        Layout::vertical([Constraint::Length(step_panel_height), Constraint::Min(4)]).split(area);

    render_step_panel(frame, chunks[0], engine);
    render_terminal_feed(frame, chunks[1], output_log, hint_message, completion_card);
}

fn render_step_panel(frame: &mut Frame, area: Rect, engine: &Engine) {
    let step = engine.current_step();
    let block = titled_block("Step");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("  Objective: ", UiStyle::MUTED),
            Span::styled(
                step.objective.clone(),
                UiStyle::TEXT_PRIMARY.add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!(
                "  {} min  |  {}  |  {}",
                step.timebox_min,
                step.difficulty,
                step.domains.join(", ")
            ),
            UiStyle::MUTED,
        )]),
        Line::from(""),
    ];

    if step.run_commands.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no suggested commands — use objective + docs)",
            UiStyle::MUTED,
        )));
    } else {
        lines.push(Line::from(Span::styled("  Runbook:", UiStyle::MUTED)));
        for (idx, cmd) in step.run_commands.iter().take(5).enumerate() {
            let blurb = command_blurb(cmd);
            lines.push(Line::from(vec![
                Span::styled(format!("  [{:>1}] ", idx + 1), UiStyle::MUTED),
                Span::styled(blurb, UiStyle::TEXT_PRIMARY),
            ]));
            lines.push(Line::from(vec![
                Span::raw("      "),
                Span::styled(cmd.clone(), UiStyle::COMMAND),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_terminal_feed(
    frame: &mut Frame,
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

fn status_badge(status: &str) -> (&'static str, ratatui::style::Style) {
    let normalized = status.trim();
    let lower = normalized.to_ascii_lowercase();

    let (label, style) = if lower.starts_with("error")
        || lower.starts_with("fail")
        || lower.contains("failed")
        || lower.contains("error")
    {
        ("ERROR", UiStyle::ERROR)
    } else if lower.starts_with("warn") || lower.contains("warning") {
        ("WARN", UiStyle::WARNING)
    } else if lower.starts_with("ok")
        || lower.starts_with("done")
        || lower.starts_with("success")
        || lower.contains("complete")
        || lower.contains("ready")
    {
        ("SUCCESS", UiStyle::OK)
    } else {
        ("INFO", UiStyle::HIGHLIGHT)
    };

    (label, style.add_modifier(Modifier::BOLD))
}

fn render_activity_rail(
    frame: &mut Frame,
    area: Rect,
    run_commands: &[String],
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
    status: &str,
) {
    let block = titled_block("Activity");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let (status_label, status_style) = status_badge(status);
    let mut lines = vec![Line::from(vec![
        Span::styled("  Status ", UiStyle::MUTED),
        Span::styled(format!("[{status_label}]"), status_style),
        Span::raw(" "),
        Span::styled(status.to_string(), UiStyle::TEXT_PRIMARY),
    ])];

    if let Some(hint) = hint_message {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Hint",
            UiStyle::WARNING.add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {hint}"),
            UiStyle::WARNING,
        )));
    }

    if let Some(card) = completion_card {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Verify Result",
            UiStyle::OK.add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {}", card.done),
            UiStyle::OK,
        )));
    }

    if !run_commands.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  What You'll Do",
            UiStyle::HIGHLIGHT.add_modifier(Modifier::BOLD),
        )));
        for cmd in run_commands.iter().take(3) {
            let blurb = command_blurb(cmd);
            lines.push(Line::from(vec![
                Span::styled("  ▸ ", UiStyle::MUTED),
                Span::styled(blurb, UiStyle::TEXT_PRIMARY),
            ]));
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(cmd.clone(), UiStyle::COMMAND),
            ]));
        }
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn render_command_bar(frame: &mut Frame, area: Rect, command_input: &str, status: &str) {
    if area.height == 0 {
        return;
    }

    let help_text = footer_help_text(command_input);
    let (badge_label, badge_style) = mode_badge(status, command_input);

    if area.height == 1 {
        let compact = Line::from(vec![
            Span::styled(" ❯ ", UiStyle::PROMPT.add_modifier(Modifier::BOLD)),
            Span::styled(command_input.to_string(), UiStyle::TEXT_PRIMARY),
            Span::raw(" "),
            Span::styled(format!("[{badge_label}]"), badge_style),
        ]);
        frame.render_widget(Paragraph::new(compact), area);
        return;
    }

    if area.height == 2 {
        let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("  {help_text}"),
                UiStyle::MUTED,
            ))),
            rows[0],
        );

        let compact = Line::from(vec![
            Span::styled("  ❯ ", UiStyle::PROMPT.add_modifier(Modifier::BOLD)),
            Span::styled(command_input.to_string(), UiStyle::TEXT_PRIMARY),
            Span::raw("  "),
            Span::styled(format!("[{badge_label}]"), badge_style),
        ]);
        frame.render_widget(
            Paragraph::new(compact).block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(UiStyle::BORDER),
            ),
            rows[1],
        );
        return;
    }

    let bar = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {help_text}"),
            UiStyle::MUTED,
        ))),
        bar[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("  {status}"),
            UiStyle::TEXT_SECONDARY,
        )))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(UiStyle::BORDER),
        ),
        bar[1],
    );

    let input_line = Line::from(vec![
        Span::styled("  ❯ ", UiStyle::PROMPT.add_modifier(Modifier::BOLD)),
        Span::styled(command_input.to_string(), UiStyle::TEXT_PRIMARY),
        Span::raw("  "),
        Span::styled(format!("[{badge_label}]"), badge_style),
    ]);

    frame.render_widget(Paragraph::new(input_line), bar[2]);
}
