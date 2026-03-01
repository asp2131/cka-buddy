use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::Modifier,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
};

use super::{big_numbers::render_readiness_delta, constants::UiStyle};

#[derive(Debug, Clone)]
pub enum PopupMessage {
    StepComplete {
        title: String,
        what_changed: Vec<String>,
        next_commands: Vec<String>,
        readiness_before: u8,
        readiness_after: u8,
    },
    Help {
        commands: Vec<(&'static str, &'static str)>,
    },
    VerifyFail {
        message: String,
    },
    Tutorial {
        page: usize,
        total_pages: usize,
        content: Vec<(String, Vec<String>)>,
    },
}

impl PopupMessage {
    pub fn help() -> Self {
        Self::Help {
            commands: vec![
                ("/new", "Reset progress and restart from Project 00"),
                ("/next", "Go to the next step"),
                ("/prev", "Go to the previous step"),
                ("/back", "Jump to last completed step"),
                ("/recommended", "Jump to recommended next step"),
                ("/verify", "Run verification checks"),
                ("/hint", "Get a contextual hint from the coach"),
                ("/suggest [n]", "Load a suggested command"),
                ("/clear", "Clear the terminal output"),
                ("/help", "Show this help overlay"),
                ("/shell", "Show current shell mode"),
                ("/shell embedded", "Use embedded PTY runner"),
                ("/shell external", "Pair to one external terminal via PIN command"),
                ("/quit", "Save progress and exit"),
                ("Ctrl+C", "Quick quit from anywhere"),
            ],
        }
    }

    pub fn tutorial() -> Self {
        Self::Tutorial {
            page: 0,
            total_pages: 3,
            content: vec![
                (
                    "Welcome to CKA Buddy".to_string(),
                    vec![
                        "CKA Buddy is an interactive Kubernetes exam preparation tool.".to_string(),
                        "It guides you through hands-on exercises that mirror the CKA exam."
                            .to_string(),
                        "".to_string(),
                        "You'll work with a real terminal — run kubectl commands directly."
                            .to_string(),
                        "Each step has an objective, suggested commands, and verification."
                            .to_string(),
                    ],
                ),
                (
                    "Commands".to_string(),
                    vec![
                        "/next, /prev     — Navigate between steps".to_string(),
                        "/recommended     — Jump to suggested next step".to_string(),
                        "/back            — Return to last completed step".to_string(),
                        "/verify          — Check if current step is done".to_string(),
                        "/hint            — Get a contextual hint".to_string(),
                        "/suggest [n]     — Load a suggested command".to_string(),
                        "/clear           — Clear terminal output".to_string(),
                        "/help            — Show command reference".to_string(),
                        "/new             — Reset progress and restart".to_string(),
                        "/quit            — Save and exit".to_string(),
                        "".to_string(),
                        "Or type any kubectl command directly to execute it.".to_string(),
                    ],
                ),
                (
                    "Interface Guide".to_string(),
                    vec![
                        "Header:     Shows readiness %, step info, and progress bar".to_string(),
                        "Step Panel: Objective, domains, difficulty, and suggested commands"
                            .to_string(),
                        "Terminal:   Live output from your kubectl commands".to_string(),
                        "Command Bar: Type commands here. Slash commands for navigation."
                            .to_string(),
                        "".to_string(),
                        "Wide terminals (120+ cols) show an Activity Rail on the right."
                            .to_string(),
                        "Your progress is saved automatically.".to_string(),
                    ],
                ),
            ],
        }
    }

    pub fn navigate_tutorial(&mut self, delta: i32) {
        if let Self::Tutorial {
            page, total_pages, ..
        } = self
        {
            let new_page = (*page as i32 + delta).clamp(0, *total_pages as i32 - 1) as usize;
            *page = new_page;
        }
    }

    pub fn is_last_tutorial_page(&self) -> bool {
        matches!(self, Self::Tutorial { page, total_pages, .. } if *page + 1 >= *total_pages)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_rect = centered_rect(area, 70, 60);

        frame.render_widget(Clear, popup_rect);
        frame.render_widget(
            Block::default().style(ratatui::style::Style::default().bg(ratatui::style::Color::Rgb(14, 18, 28))),
            popup_rect,
        );

        let block = Block::bordered()
            .border_set(border::THICK)
            .border_style(UiStyle::HIGHLIGHT)
            .style(UiStyle::PANEL_BG);

        let inner = block.inner(popup_rect);
        frame.render_widget(block, popup_rect);

        match self {
            Self::StepComplete {
                title,
                what_changed,
                next_commands,
                readiness_before,
                readiness_after,
            } => {
                render_step_complete(
                    frame,
                    inner,
                    title,
                    what_changed,
                    next_commands,
                    *readiness_before,
                    *readiness_after,
                );
            }
            Self::Help { commands } => {
                render_help(frame, inner, commands);
            }
            Self::VerifyFail { message } => {
                render_verify_fail(frame, inner, message);
            }
            Self::Tutorial {
                page,
                total_pages,
                content,
            } => {
                render_tutorial(frame, inner, *page, *total_pages, content);
            }
        }
    }
}

fn render_step_complete(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    what_changed: &[String],
    next_commands: &[String],
    readiness_before: u8,
    readiness_after: u8,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(8),
        Constraint::Min(2),
        Constraint::Length(1),
    ])
    .split(area);

    let header = vec![
        Line::from(""),
        Line::from(Span::styled(
            " ✓ STEP COMPLETE ",
            UiStyle::OK.add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(format!("   {title}"), UiStyle::TEXT_PRIMARY)),
    ];
    frame.render_widget(Paragraph::new(header).centered(), chunks[0]);

    render_readiness_delta(frame, chunks[1], readiness_before, readiness_after);

    let mut body_lines = Vec::new();
    if !what_changed.is_empty() {
        body_lines.push(Line::from(Span::styled(
            " What changed:",
            UiStyle::TEXT_SECONDARY.add_modifier(Modifier::BOLD),
        )));
        for item in what_changed {
            body_lines.push(Line::from(vec![
                Span::styled("  ✓ ", UiStyle::OK),
                Span::styled(item.clone(), UiStyle::TEXT_PRIMARY),
            ]));
        }
    }
    if !next_commands.is_empty() {
        body_lines.push(Line::from(""));
        body_lines.push(Line::from(Span::styled(
            " Next suggested:",
            UiStyle::TEXT_SECONDARY.add_modifier(Modifier::BOLD),
        )));
        for cmd in next_commands {
            body_lines.push(Line::from(vec![
                Span::styled("  ▸ ", UiStyle::HIGHLIGHT),
                Span::styled(cmd.clone(), UiStyle::COMMAND),
            ]));
        }
    }
    frame.render_widget(
        Paragraph::new(body_lines).wrap(Wrap { trim: true }),
        chunks[2],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Press Enter to continue",
            UiStyle::MUTED,
        )))
        .centered(),
        chunks[3],
    );
}

fn render_help(frame: &mut Frame, area: Rect, commands: &[(&str, &str)]) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(2),
        Constraint::Length(1),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                " ☸ COMMAND REFERENCE ",
                UiStyle::HEADER.add_modifier(Modifier::BOLD),
            )),
        ])
        .centered(),
        chunks[0],
    );

    let mut lines = Vec::new();
    for (cmd, desc) in commands {
        lines.push(Line::from(vec![
            Span::styled(format!("  {cmd:<18}"), UiStyle::COMMAND),
            Span::styled(" │ ", UiStyle::BORDER),
            Span::styled(*desc, UiStyle::TEXT_SECONDARY),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Press Enter or Esc to close",
            UiStyle::MUTED,
        )))
        .centered(),
        chunks[2],
    );
}

fn render_verify_fail(frame: &mut Frame, area: Rect, message: &str) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(2),
        Constraint::Length(1),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                " ✖ NOT YET ",
                UiStyle::WARNING.add_modifier(Modifier::BOLD),
            )),
        ])
        .centered(),
        chunks[0],
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  ▸ ", UiStyle::WARNING),
                Span::styled(message.to_string(), UiStyle::TEXT_PRIMARY),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  Review the step objective and try again.",
                UiStyle::TEXT_SECONDARY,
            )),
        ])
        .wrap(Wrap { trim: true }),
        chunks[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Press Enter to close",
            UiStyle::MUTED,
        )))
        .centered(),
        chunks[2],
    );
}

fn render_tutorial(
    frame: &mut Frame,
    area: Rect,
    page: usize,
    total_pages: usize,
    content: &[(String, Vec<String>)],
) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(2),
        Constraint::Length(1),
    ])
    .split(area);

    if let Some((title, _)) = content.get(page) {
        let progress_dots: String = (0..total_pages)
            .map(|i| if i == page { "●" } else { "○" })
            .collect::<Vec<_>>()
            .join(" ");

        let header = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!(" ☸ {title} "),
                    UiStyle::HEADER.add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {progress_dots} "), UiStyle::MUTED),
            ]),
        ];
        frame.render_widget(Paragraph::new(header).centered(), chunks[0]);
    }

    if let Some((_, body)) = content.get(page) {
        let lines: Vec<Line> = body
            .iter()
            .map(|s| Line::from(Span::styled(format!("  {s}"), UiStyle::TEXT_PRIMARY)))
            .collect();
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), chunks[1]);
    }

    let nav_hint = if page + 1 >= total_pages {
        "Press Enter to start ▸"
    } else {
        "← → Navigate │ Enter to skip"
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(nav_hint, UiStyle::MUTED))).centered(),
        chunks[2],
    );
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_height = (area.height * percent_y) / 100;
    let popup_width = (area.width * percent_x) / 100;

    let vertical = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}
