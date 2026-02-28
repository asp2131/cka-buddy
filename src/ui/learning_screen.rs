use crossterm::event::KeyCode;
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
    button::Button,
    clickable_list::{ClickableList, ClickableListItem},
    cluster_view::ClusterScene,
    constants::UiStyle,
    traits::Screen,
    ui_action::UiAction,
    ui_frame::UiFrame,
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
    pub cluster_scene: ClusterScene,
    pub scroll_offset: usize,
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
            cluster_scene: ClusterScene::default(),
            scroll_offset: 0,
        }
    }

    pub fn rebuild_cluster_scene(&mut self, engine: &Engine) {
        let step = engine.current_step();
        let domain = step.domains.first().map(|s| s.as_str()).unwrap_or("cluster");
        let completed = engine.progress.completed.len();
        self.cluster_scene = ClusterScene::for_domain(domain, &step.run_commands, completed);
    }
}

impl Screen for LearningScreen {
    fn update(&mut self, _engine: &Engine) -> anyhow::Result<()> {
        self.cluster_scene.tick();
        Ok(())
    }

    fn render(&mut self, frame: &mut UiFrame, engine: &Engine, area: Rect) -> anyhow::Result<()> {
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

            let main_chunks = Layout::vertical([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(split[0]);

            render_main_feed(
                frame,
                main_chunks[0],
                engine,
                &self.status,
                &self.output_log,
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
                &self.command_input,
                self.scroll_offset,
            );
            render_action_buttons(frame, main_chunks[1], &self.command_input);

            render_cluster_rail(
                frame,
                split[1],
                &self.cluster_scene,
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
                &self.status,
            );
        } else {
            let main_chunks = Layout::vertical([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(zones[1]);

            render_main_feed(
                frame,
                main_chunks[0],
                engine,
                &self.status,
                &self.output_log,
                self.hint_message.as_deref(),
                self.completion_card.as_ref(),
                &self.command_input,
                self.scroll_offset,
            );
            render_action_buttons(frame, main_chunks[1], &self.command_input);
        }

        render_command_bar(frame.inner_frame(), zones[2], &self.command_input, &self.status);
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

    fn footer_spans(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("V", "Verify"),
            ("H", "Hint"),
            ("S", "Suggest"),
            ("→", "Next"),
            ("←", "Prev"),
            ("?", "Help"),
            ("Esc", "Clear"),
            ("Enter", "Run"),
        ]
    }
}

fn render_header(frame: &mut UiFrame, area: Rect, engine: &Engine) {
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

#[allow(clippy::too_many_arguments)]
fn render_main_feed(
    frame: &mut UiFrame,
    area: Rect,
    engine: &Engine,
    _status: &str,
    output_log: &[String],
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
    command_input: &str,
    scroll_offset: usize,
) {
    let step = engine.current_step();
    let command_count = step.run_commands.len().min(5) as u16;
    let step_panel_height =
        (6 + command_count).clamp(6, area.height.saturating_sub(8).max(6));

    let chunks =
        Layout::vertical([Constraint::Length(step_panel_height), Constraint::Min(4)]).split(area);

    render_step_panel(frame, chunks[0], engine, command_input, scroll_offset);
    render_terminal_feed(frame.inner_frame(), chunks[1], output_log, hint_message, completion_card);
}

fn render_step_panel(
    frame: &mut UiFrame,
    area: Rect,
    engine: &Engine,
    _command_input: &str,
    scroll_offset: usize,
) {
    let step = engine.current_step();
    let block = titled_block("Step");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let objective_lines = vec![
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

    let obj_height = objective_lines.len() as u16;
    let obj_area = Rect::new(inner.x, inner.y, inner.width, obj_height.min(inner.height));
    frame.render_widget(Paragraph::new(objective_lines).wrap(Wrap { trim: true }), obj_area);

    let remaining_height = inner.height.saturating_sub(obj_height);
    if remaining_height == 0 {
        return;
    }

    let list_area = Rect::new(
        inner.x,
        inner.y + obj_height,
        inner.width,
        remaining_height,
    );

    if step.run_commands.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  (no suggested commands — use objective + docs)",
                UiStyle::MUTED,
            ))),
            list_area,
        );
    } else {
        let items: Vec<ClickableListItem> = step
            .run_commands
            .iter()
            .take(5)
            .map(|cmd| ClickableListItem {
                label: cmd.clone(),
                description: command_blurb(cmd).to_string(),
                on_click: UiAction::SetCommandInput(cmd.clone()),
            })
            .collect();

        let title_line_area = Rect::new(list_area.x, list_area.y, list_area.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("  Runbook (click to load):", UiStyle::MUTED))),
            title_line_area,
        );

        let clickable_area = Rect::new(
            list_area.x + 2,
            list_area.y + 1,
            list_area.width.saturating_sub(2),
            list_area.height.saturating_sub(1),
        );

        if clickable_area.height > 0 {
            let list = ClickableList::new(&items).scroll_offset(scroll_offset);
            frame.render_interactive_widget(list, clickable_area);
        }
    }
}

fn render_action_buttons(frame: &mut UiFrame, area: Rect, command_input: &str) {
    let input_empty = command_input.is_empty();
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(UiStyle::BORDER);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width < 30 {
        return;
    }

    let buttons_area = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .split(inner);

    let verify = Button::new("Verify")
        .hotkey(KeyCode::Char('v'))
        .on_click(UiAction::Verify)
        .hover_text("Run verification checks for current step")
        .command_input_empty(input_empty);
    frame.render_interactive_widget(verify, buttons_area[0]);

    let hint = Button::new("Hint")
        .hotkey(KeyCode::Char('h'))
        .on_click(UiAction::Hint)
        .hover_text("Get a contextual hint from the coach")
        .command_input_empty(input_empty);
    frame.render_interactive_widget(hint, buttons_area[1]);

    let suggest = Button::new("Suggest")
        .hotkey(KeyCode::Char('s'))
        .on_click(UiAction::Suggest(None))
        .hover_text("Load next suggested command")
        .command_input_empty(input_empty);
    frame.render_interactive_widget(suggest, buttons_area[2]);

    let next = Button::new("Next →")
        .hotkey(KeyCode::Right)
        .on_click(UiAction::NextStep)
        .hover_text("Go to next step")
        .command_input_empty(input_empty);
    frame.render_interactive_widget(next, buttons_area[3]);

    let prev = Button::new("← Prev")
        .hotkey(KeyCode::Left)
        .on_click(UiAction::PrevStep)
        .hover_text("Go to previous step")
        .command_input_empty(input_empty);
    frame.render_interactive_widget(prev, buttons_area[4]);
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

fn render_cluster_rail(
    frame: &mut UiFrame,
    area: Rect,
    cluster_scene: &ClusterScene,
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
    status: &str,
) {
    let block = titled_block("Cluster");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 6 {
        return;
    }

    let status_height = 6u16;
    let cluster_height = inner.height.saturating_sub(status_height);

    let sections = Layout::vertical([
        Constraint::Length(cluster_height),
        Constraint::Length(status_height),
    ])
    .split(inner);

    frame.render_widget(cluster_scene, sections[0]);

    let (status_label, status_style) = status_badge(status);
    let mut status_lines = vec![Line::from(vec![
        Span::styled("  Status ", UiStyle::MUTED),
        Span::styled(format!("[{status_label}]"), status_style),
    ])];

    if let Some(hint) = hint_message {
        let truncated = ellipsize(hint, (sections[1].width as usize).saturating_sub(4));
        status_lines.push(Line::from(vec![
            Span::styled("  Hint: ", UiStyle::WARNING),
            Span::styled(truncated, UiStyle::WARNING),
        ]));
    }

    if let Some(card) = completion_card {
        let truncated = ellipsize(&card.done, (sections[1].width as usize).saturating_sub(4));
        status_lines.push(Line::from(vec![
            Span::styled("  ✓ ", UiStyle::OK),
            Span::styled(truncated, UiStyle::OK),
        ]));
    }

    frame.render_widget(
        Paragraph::new(status_lines).wrap(Wrap { trim: true }),
        sections[1],
    );
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
