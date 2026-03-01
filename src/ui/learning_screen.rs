use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::engine::Engine;
use crate::app::state::CompletionCard;
use crate::content::command_info::command_blurb;

use super::{
    button::Button,
    clickable_list::ClickableList,
    cluster_view::ClusterView,
    constants::UiStyle,
    effects,
    traits::Screen,
    ui_action::UiAction,
    ui_frame::UiFrame,
    widgets::{
        difficulty_badge, domain_tag, ellipsize, footer_help_text, mode_badge,
        readiness_segments, step_type_badge,
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
    first_render: bool,
    tick: usize,
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
            first_render: true,
            tick: 0,
        }
    }
}

impl Screen for LearningScreen {
    fn update(&mut self, _engine: &Engine) -> anyhow::Result<()> {
        self.tick = self.tick.wrapping_add(1);
        Ok(())
    }

    fn render(
        &mut self,
        frame: &mut UiFrame<'_, '_>,
        engine: &Engine,
        area: Rect,
    ) -> anyhow::Result<()> {
        let zones = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(4),
        ])
        .split(area);

        render_header(frame, zones[0], engine, self.tick);

        if self.first_render {
            frame.push_effect("learning_header", effects::header_sweep(), zones[0]);
            frame.push_effect("learning_body", effects::screen_transition_in(), zones[1]);
            self.first_render = false;
        }

        if zones[1].width >= WIDE_TERMINAL_THRESHOLD {
            let split =
                Layout::horizontal([Constraint::Percentage(68), Constraint::Percentage(32)])
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
                self.tick,
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

        render_action_row(frame, zones[2], self.command_input.is_empty());
        render_command_bar(frame, zones[3], &self.command_input, &self.status);
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
            ("S".to_string(), "Suggest".to_string()),
            ("Enter".to_string(), "Run".to_string()),
            ("/help".to_string(), "Commands".to_string()),
        ]
    }
}

fn render_header(frame: &mut UiFrame<'_, '_>, area: Rect, engine: &Engine, tick: usize) {
    let step = engine.current_step();

    let header_block = Block::bordered()
        .border_style(UiStyle::BORDER_ACTIVE)
        .style(UiStyle::PANEL_BG);
    let inner = header_block.inner(area);
    frame.render_widget(header_block, area);

    let (bar_on, bar_off) = readiness_segments(engine.readiness, 16);

    let step_chip = format!(
        "[{:02}/{:02}]",
        engine.current_index + 1,
        engine.steps.len()
    );

    let title = ellipsize(&step.title, inner.width.saturating_sub(4) as usize);

    let pulse = if (tick / 5).is_multiple_of(2) { "◉" } else { "●" };

    let row1 = Line::from(vec![
        Span::styled(
            format!(" {pulse} CKA BUDDY "),
            UiStyle::HEADER.add_modifier(Modifier::BOLD),
        ),
        Span::styled("▐", UiStyle::MUTED),
        Span::styled(bar_on, UiStyle::OK),
        Span::styled(bar_off, UiStyle::MUTED),
        Span::styled("▌ ", UiStyle::MUTED),
        Span::styled(
            format!("{:>3}%", engine.readiness),
            UiStyle::TEXT_PRIMARY.add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", UiStyle::MUTED),
        Span::styled(&step_chip, UiStyle::HIGHLIGHT),
        Span::raw(" "),
        step_type_badge(&step.step_type),
        Span::raw(" "),
        difficulty_badge(&step.difficulty),
    ]);

    let mut row2_spans = vec![
        Span::styled("  ☸ ", UiStyle::HIGHLIGHT),
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
    frame: &mut UiFrame<'_, '_>,
    area: Rect,
    engine: &Engine,
    _status: &str,
    output_log: &[String],
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
) {
    let step = engine.current_step();
    let command_count = step.run_commands.len().min(5) as u16;
    let step_panel_height =
        (6 + command_count * 2).clamp(6, area.height.saturating_sub(8).max(6));

    let chunks =
        Layout::vertical([Constraint::Length(step_panel_height), Constraint::Min(4)]).split(area);

    render_step_panel(frame, chunks[0], engine);
    render_terminal_feed(frame, chunks[1], output_log, hint_message, completion_card);
}

fn render_step_panel(frame: &mut UiFrame<'_, '_>, area: Rect, engine: &Engine) {
    let step = engine.current_step();

    let block = Block::bordered()
        .border_style(UiStyle::BORDER_ACTIVE)
        .title(Span::styled(" ☸ Step ", UiStyle::HIGHLIGHT))
        .style(UiStyle::PANEL_BG);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(vec![
            Span::styled("  ◆ Objective: ", UiStyle::HIGHLIGHT),
            Span::styled(
                step.objective.clone(),
                UiStyle::TEXT_PRIMARY.add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!(
                "    ⏱ {} min  │  {}  │  {}",
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
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
    } else {
        lines.push(Line::from(Span::styled(
            "  ▾ Runbook:",
            UiStyle::TEXT_SECONDARY.add_modifier(Modifier::BOLD),
        )));
        for (idx, cmd) in step.run_commands.iter().take(5).enumerate() {
            let blurb = command_blurb(cmd);
            lines.push(Line::from(vec![
                Span::styled(format!("  [{idx}] "), UiStyle::MUTED),
                Span::styled(blurb, UiStyle::TEXT_PRIMARY),
            ]));
        }

        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);

        let list_top = inner.y.saturating_add(4);
        let available_rows = inner.bottom().saturating_sub(list_top);
        let rows = step.run_commands.len().min(5).min(available_rows as usize);
        if rows > 0 {
            let list_area = Rect::new(inner.x, list_top, inner.width, rows as u16);
            let items = step
                .run_commands
                .iter()
                .take(rows)
                .cloned()
                .collect::<Vec<_>>();
            let actions = items
                .iter()
                .map(|cmd| UiAction::SetCommandInput(cmd.clone()))
                .collect::<Vec<_>>();
            let list = ClickableList::new(items, actions)
                .style(UiStyle::COMMAND)
                .marker("      ");
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
    let block = Block::bordered()
        .border_style(UiStyle::BORDER)
        .title(Span::styled(" ▸ Terminal ", UiStyle::TEXT_SECONDARY))
        .style(UiStyle::PANEL_BG);
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
            Span::styled("  ◈ Hint: ", UiStyle::WARNING.add_modifier(Modifier::BOLD)),
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
                Span::styled(card.next_commands.join(" │ "), UiStyle::MUTED),
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
        ("OK", UiStyle::OK)
    } else {
        ("INFO", UiStyle::HIGHLIGHT)
    };

    (label, style.add_modifier(Modifier::BOLD))
}

fn render_activity_rail(
    frame: &mut UiFrame<'_, '_>,
    area: Rect,
    run_commands: &[String],
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
    status: &str,
    tick: usize,
) {
    let block = Block::bordered()
        .border_style(UiStyle::BORDER)
        .title(Span::styled(" ⚡ Activity ", UiStyle::HIGHLIGHT))
        .style(UiStyle::PANEL_BG);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::vertical([Constraint::Min(6), Constraint::Length(8)]).split(inner);

    let cluster = ClusterView::new("CLUSTER", "k8s", run_commands, tick);
    frame.render_widget(cluster, sections[0]);

    let (status_label, status_style) = status_badge(status);

    let heartbeat = if (tick / 8).is_multiple_of(2) { "◉" } else { "●" };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("  {heartbeat} Status "), UiStyle::MUTED),
            Span::styled(format!("[{status_label}]"), status_style),
        ]),
        Line::from(Span::styled(
            format!("  {}", ellipsize(status, inner.width.saturating_sub(4) as usize)),
            UiStyle::TEXT_SECONDARY,
        )),
    ];

    if let Some(hint) = hint_message {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ◈ ", UiStyle::WARNING),
            Span::styled(
                ellipsize(hint, inner.width.saturating_sub(6) as usize),
                UiStyle::WARNING,
            ),
        ]));
    }

    if let Some(card) = completion_card {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ✓ ", UiStyle::OK),
            Span::styled(
                ellipsize(&card.done, inner.width.saturating_sub(6) as usize),
                UiStyle::OK,
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), sections[1]);
}

fn render_action_row(frame: &mut UiFrame<'_, '_>, area: Rect, enable_hotkeys: bool) {
    if area.height < 3 {
        return;
    }

    let rows = Layout::horizontal([
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Min(0),
    ])
    .split(area);

    let verify = {
        let b = Button::new("⚡ Verify")
            .on_click(UiAction::Verify)
            .hover_text(Text::from("Run verification checks for current step"));
        if enable_hotkeys {
            b.hotkey(KeyCode::Char('v'))
        } else {
            b
        }
    };

    let hint = {
        let b = Button::new("◈ Hint")
            .on_click(UiAction::Hint)
            .hover_text(Text::from("Get a contextual hint from the coach"));
        if enable_hotkeys {
            b.hotkey(KeyCode::Char('h'))
        } else {
            b
        }
    };

    let suggest = {
        let b = Button::new("▸ Suggest")
            .on_click(UiAction::Suggest(None))
            .hover_text(Text::from("Load next suggested command"));
        if enable_hotkeys {
            b.hotkey(KeyCode::Char('s'))
        } else {
            b
        }
    };

    let next = {
        let b = Button::new("Next →")
            .on_click(UiAction::NextStep)
            .hover_text(Text::from("Go to next step"));
        if enable_hotkeys {
            b.hotkey(KeyCode::Right)
        } else {
            b
        }
    };

    let prev = {
        let b = Button::new("← Prev")
            .on_click(UiAction::PrevStep)
            .hover_text(Text::from("Go to previous step"));
        if enable_hotkeys {
            b.hotkey(KeyCode::Left)
        } else {
            b
        }
    };

    frame.render_interactive_widget(verify, rows[0]);
    frame.render_interactive_widget(hint, rows[1]);
    frame.render_interactive_widget(suggest, rows[2]);
    frame.render_interactive_widget(next, rows[3]);
    frame.render_interactive_widget(prev, rows[4]);
}

fn render_command_bar(frame: &mut UiFrame<'_, '_>, area: Rect, command_input: &str, status: &str) {
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

    let cursor_blink = "█";
    let input_line = Line::from(vec![
        Span::styled("  ❯ ", UiStyle::PROMPT.add_modifier(Modifier::BOLD)),
        Span::styled(command_input.to_string(), UiStyle::TEXT_PRIMARY),
        Span::styled(cursor_blink, UiStyle::HIGHLIGHT),
        Span::raw("  "),
        Span::styled(format!("[{badge_label}]"), badge_style),
    ]);

    frame.render_widget(Paragraph::new(input_line), bar[2]);
}
