use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Block,
};

use crate::content::models::StepType;

use super::constants::UiStyle;

pub fn default_block() -> Block<'static> {
    Block::bordered().border_style(UiStyle::BORDER)
}

pub fn titled_block(title: &str) -> Block<'static> {
    Block::bordered()
        .border_style(UiStyle::BORDER)
        .title(Span::styled(
            format!(" {title} "),
            UiStyle::TEXT_SECONDARY,
        ))
}

pub fn progress_bar(percentage: u8, width: usize) -> Line<'static> {
    let slots = width.saturating_sub(2);
    let filled = (usize::from(percentage) * slots) / 100;
    let empty = slots.saturating_sub(filled);

    let fill_char = "█";
    let empty_char = "░";

    let fill_style = if percentage >= 80 {
        UiStyle::OK
    } else if percentage >= 40 {
        UiStyle::WARNING
    } else {
        UiStyle::ERROR
    };

    Line::from(vec![
        Span::styled(fill_char.repeat(filled), fill_style),
        Span::styled(empty_char.repeat(empty), UiStyle::MUTED),
    ])
}

pub fn step_type_badge(step_type: &StepType) -> Span<'static> {
    match step_type {
        StepType::Exam => Span::styled(
            "[EXAM]",
            UiStyle::WARNING.add_modifier(Modifier::BOLD),
        ),
        StepType::Project => Span::styled(
            "[PROJECT]",
            UiStyle::HEADER.add_modifier(Modifier::BOLD),
        ),
        StepType::Bug => Span::styled(
            "[BUG]",
            UiStyle::ERROR.add_modifier(Modifier::BOLD),
        ),
    }
}

pub fn difficulty_badge(difficulty: &str) -> Span<'static> {
    let style = match difficulty.to_lowercase().as_str() {
        "easy" => UiStyle::DIFF_EASY,
        "medium" => UiStyle::DIFF_MEDIUM,
        "hard" => UiStyle::DIFF_HARD,
        _ => UiStyle::TEXT_SECONDARY,
    };
    Span::styled(
        format!("[{}]", difficulty.to_uppercase()),
        style.add_modifier(Modifier::BOLD),
    )
}

pub fn domain_style(domain: &str) -> Style {
    let lower = domain.to_lowercase();
    if lower.contains("storage") {
        UiStyle::DOMAIN_STORAGE
    } else if lower.contains("network") {
        UiStyle::DOMAIN_NETWORKING
    } else if lower.contains("workload") || lower.contains("scheduling") {
        UiStyle::DOMAIN_WORKLOADS
    } else if lower.contains("cluster") {
        UiStyle::DOMAIN_CLUSTER
    } else if lower.contains("security") || lower.contains("rbac") {
        UiStyle::DOMAIN_SECURITY
    } else if lower.contains("troubleshoot") {
        UiStyle::DOMAIN_TROUBLESHOOTING
    } else {
        UiStyle::TEXT_SECONDARY
    }
}

pub fn domain_tag(domain: &str) -> Span<'static> {
    Span::styled(format!("[{domain}]"), domain_style(domain))
}

pub fn mode_badge(status: &str, command_input: &str) -> (String, Style) {
    if status.starts_with("Blocked:") {
        (
            "BLOCKED".to_string(),
            UiStyle::ERROR.add_modifier(Modifier::BOLD),
        )
    } else if status.starts_with("Done:") || status.starts_with("Not yet:") {
        (
            "VERIFY".to_string(),
            UiStyle::OK.add_modifier(Modifier::BOLD),
        )
    } else if status.starts_with("Loaded suggestion") {
        (
            "SUGGEST".to_string(),
            UiStyle::WARNING.add_modifier(Modifier::BOLD),
        )
    } else if !command_input.is_empty() {
        (
            "INPUT".to_string(),
            UiStyle::HIGHLIGHT.add_modifier(Modifier::BOLD),
        )
    } else {
        ("READY".to_string(), UiStyle::MUTED)
    }
}

pub fn readiness_segments(readiness: u8, slots: usize) -> (String, String) {
    let filled = (usize::from(readiness) * slots) / 100;
    let empty = slots.saturating_sub(filled);
    ("█".repeat(filled), "░".repeat(empty))
}

pub fn divider(width: usize) -> String {
    "─".repeat(width.max(8))
}

pub fn ellipsize(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    if max <= 3 {
        return "...".to_string();
    }
    text.chars().take(max - 3).collect::<String>() + "..."
}

pub fn footer_help_text(command_input: &str) -> String {
    if command_input.starts_with("/hint") {
        "/hint — Ask the coach for a contextual hint".to_string()
    } else if command_input.starts_with("/verify") {
        "/verify — Run verification checks for current step".to_string()
    } else if command_input.starts_with("/suggest") {
        "/suggest [n] — Load suggested command".to_string()
    } else if command_input.starts_with('/') {
        "Type a slash command. /help for full list.".to_string()
    } else if command_input.is_empty() {
        "Type kubectl commands or /help for available commands".to_string()
    } else {
        format!("Press Enter to execute: {command_input}")
    }
}
