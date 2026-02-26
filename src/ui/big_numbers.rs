use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
};

use super::constants::UiStyle;

const DIGIT_HEIGHT: u16 = 6;

fn big_text(rows: &[&str]) -> Vec<Line<'static>> {
    rows.iter()
        .map(|line| {
            let mut spans = Vec::new();
            for c in line.chars() {
                if c == '█' {
                    spans.push(Span::styled(
                        "█",
                        UiStyle::OK.add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(c.to_string(), UiStyle::HIGHLIGHT));
                }
            }
            Line::from(spans)
        })
        .collect()
}

fn digit_lines(d: u8) -> Vec<Line<'static>> {
    match d {
        0 => big_text(&[
            " ██████╗ ",
            "██╔═████╗",
            "██║██╔██║",
            "████╔╝██║",
            "╚██████╔╝",
            " ╚═════╝ ",
        ]),
        1 => big_text(&[" ██╗", "███║", "╚██║", " ██║", " ██║", " ╚═╝"]),
        2 => big_text(&[
            "██████╗ ",
            "╚════██╗",
            " █████╔╝",
            "██╔═══╝ ",
            "███████╗",
            "╚══════╝",
        ]),
        3 => big_text(&[
            "██████╗ ",
            "╚════██╗",
            " █████╔╝",
            " ╚═══██╗",
            "██████╔╝",
            "╚═════╝ ",
        ]),
        4 => big_text(&[
            "██╗  ██╗",
            "██║  ██║",
            "███████║",
            "╚════██║",
            "     ██║",
            "     ╚═╝",
        ]),
        5 => big_text(&[
            "███████╗",
            "██╔════╝",
            "███████╗",
            "╚════██║",
            "███████║",
            "╚══════╝",
        ]),
        6 => big_text(&[
            " ██████╗ ",
            "██╔════╝ ",
            "███████╗ ",
            "██╔═══██╗",
            "╚██████╔╝",
            " ╚═════╝ ",
        ]),
        7 => big_text(&[
            "███████╗",
            "╚════██║",
            "    ██╔╝",
            "   ██╔╝ ",
            "   ██║  ",
            "   ╚═╝  ",
        ]),
        8 => big_text(&[
            " █████╗ ",
            "██╔══██╗",
            "╚█████╔╝",
            "██╔══██╗",
            "╚█████╔╝",
            " ╚════╝ ",
        ]),
        9 => big_text(&[
            " █████╗ ",
            "██╔══██╗",
            "╚██████║",
            " ╚═══██║",
            " █████╔╝",
            " ╚════╝ ",
        ]),
        _ => big_text(&["   ", "██╗", "╚═╝", "██╗", "╚═╝", "   "]),
    }
}

fn percent_lines() -> Vec<Line<'static>> {
    big_text(&[
        "██╗██╗",
        "╚═╝██║",
        "  ██╔╝",
        " ██╔╝ ",
        "██╔╝  ",
        "╚═╝██╗",
    ])
}

fn arrow_lines() -> Vec<Line<'static>> {
    big_text(&[
        "       ",
        "  ▶▶▶  ",
        " ▶▶▶▶▶ ",
        "  ▶▶▶  ",
        "       ",
        "       ",
    ])
}

pub fn render_readiness_delta(
    frame: &mut Frame,
    area: Rect,
    before: u8,
    after: u8,
) {
    let digit_width: u16 = 10;
    let arrow_width: u16 = 9;
    let percent_width: u16 = 8;

    let before_digits = split_digits(before);
    let after_digits = split_digits(after);

    let before_cols = before_digits.len() as u16;
    let after_cols = after_digits.len() as u16;

    let total_width = (before_cols * digit_width)
        + percent_width
        + arrow_width
        + (after_cols * digit_width)
        + percent_width;

    if area.width < total_width || area.height < DIGIT_HEIGHT {
        let fallback = Line::from(vec![
            Span::styled(format!("{before}%"), UiStyle::WARNING),
            Span::styled(" → ", UiStyle::TEXT_SECONDARY),
            Span::styled(format!("{after}%"), UiStyle::OK),
        ]);
        frame.render_widget(Paragraph::new(fallback).centered(), area);
        return;
    }

    let mut constraints = Vec::new();
    for _ in &before_digits {
        constraints.push(Constraint::Length(digit_width));
    }
    constraints.push(Constraint::Length(percent_width));
    constraints.push(Constraint::Length(arrow_width));
    for _ in &after_digits {
        constraints.push(Constraint::Length(digit_width));
    }
    constraints.push(Constraint::Length(percent_width));

    let y_offset = (area.height.saturating_sub(DIGIT_HEIGHT)) / 2;
    let render_area = Rect::new(
        area.x + (area.width.saturating_sub(total_width)) / 2,
        area.y + y_offset,
        total_width,
        DIGIT_HEIGHT,
    );

    let cols = Layout::horizontal(constraints).split(render_area);

    let mut col_idx = 0;
    for d in &before_digits {
        let lines = digit_lines(*d);
        frame.render_widget(Paragraph::new(lines), cols[col_idx]);
        col_idx += 1;
    }
    frame.render_widget(Paragraph::new(percent_lines()), cols[col_idx]);
    col_idx += 1;
    frame.render_widget(Paragraph::new(arrow_lines()), cols[col_idx]);
    col_idx += 1;
    for d in &after_digits {
        let lines = digit_lines(*d);
        frame.render_widget(Paragraph::new(lines), cols[col_idx]);
        col_idx += 1;
    }
    frame.render_widget(Paragraph::new(percent_lines()), cols[col_idx]);
}

fn split_digits(value: u8) -> Vec<u8> {
    if value >= 100 {
        vec![1, 0, 0]
    } else if value >= 10 {
        vec![value / 10, value % 10]
    } else {
        vec![value]
    }
}
