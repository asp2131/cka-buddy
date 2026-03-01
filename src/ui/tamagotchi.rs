use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::Widget,
};

use super::constants::UiStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionStage {
    Egg,
    Hatchling,
    Juvenile,
    Adult,
}

impl EvolutionStage {
    pub fn from_readiness(readiness: u8) -> Self {
        match readiness {
            0..=10 => Self::Egg,
            11..=35 => Self::Hatchling,
            36..=70 => Self::Juvenile,
            _ => Self::Adult,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Happy,
    Idle,
    Sick,
    Thinking,
}

#[derive(Debug, Clone)]
pub struct KubeChanAssistant {
    pub stage: EvolutionStage,
    pub mood: Mood,
    pub tick: usize,
    pub readiness: u8,
    pub objective: String,
    pub domains: Vec<String>,
    pub difficulty: String,
    pub tip: Option<String>,
    pub hint_message: Option<String>,
}

impl KubeChanAssistant {
    pub fn new(
        readiness: u8,
        tick: usize,
        objective: &str,
        domains: &[String],
        difficulty: &str,
        fallback_hint: Option<&str>,
        run_commands: &[String],
        hint_message: Option<&str>,
        has_completion: bool,
    ) -> Self {
        let stage = EvolutionStage::from_readiness(readiness);

        let mood = if has_completion {
            Mood::Happy
        } else if hint_message.is_some() {
            Mood::Thinking
        } else if readiness > 50 {
            Mood::Happy
        } else {
            Mood::Idle
        };

        let tip = if let Some(hint) = hint_message {
            Some(hint.to_string())
        } else if let Some(fh) = fallback_hint {
            Some(fh.to_string())
        } else {
            run_commands.first().map(|cmd| format!("Start with: {}", cmd))
        };

        Self {
            stage,
            mood,
            tick,
            readiness,
            objective: objective.to_string(),
            domains: domains.to_vec(),
            difficulty: difficulty.to_string(),
            tip,
            hint_message: hint_message.map(|s| s.to_string()),
        }
    }

    fn put_line(buf: &mut Buffer, area: Rect, y_off: u16, line: &str, style: Style) {
        if y_off >= area.height {
            return;
        }
        let maxw = area.width as usize;
        if maxw == 0 {
            return;
        }
        let clipped: String = line.chars().take(maxw).collect();
        Line::styled(clipped, style).render(
            Rect::new(area.x, area.y + y_off, area.width, 1),
            buf,
        );
    }

    fn put_centered(buf: &mut Buffer, area: Rect, y_off: u16, line: &str, style: Style) {
        if y_off >= area.height {
            return;
        }
        let maxw = area.width as usize;
        if maxw == 0 {
            return;
        }
        let char_count = line.chars().count();
        let padding = if char_count < maxw {
            (maxw - char_count) / 2
        } else {
            0
        };
        let padded = format!("{}{}", " ".repeat(padding), line);
        let clipped: String = padded.chars().take(maxw).collect();
        Line::styled(clipped, style).render(
            Rect::new(area.x, area.y + y_off, area.width, 1),
            buf,
        );
    }

    fn creature_face(&self) -> (&str, Style) {
        let frame = (self.tick / 12) % 2;
        match self.stage {
            EvolutionStage::Egg => {
                let face = if frame == 0 { "  ( ◉‿◉ )  " } else { "  ( ●‿● )  " };
                (face, UiStyle::TEXT_SECONDARY)
            }
            EvolutionStage::Hatchling => {
                let (face, style) = match self.mood {
                    Mood::Happy => (if frame == 0 { " ☸( ◉ω◉ )☸" } else { " ☸( ◉ω◉ )♥" }, UiStyle::OK),
                    Mood::Thinking => (" ☸( ◉_◉ )?" , UiStyle::HIGHLIGHT),
                    Mood::Sick => (" ☸( ×_× )!" , UiStyle::WARNING),
                    Mood::Idle => (if frame == 0 { " ☸( ◉_◉ )☸" } else { " ☸( ●_● )☸" }, UiStyle::TEXT_SECONDARY),
                };
                (face, style)
            }
            EvolutionStage::Juvenile => {
                let (face, style) = match self.mood {
                    Mood::Happy => (if frame == 0 { "☸☸( ◉ω◉ )☸☸" } else { "☸☸( ◉ω◉ )♥♥" }, UiStyle::OK),
                    Mood::Thinking => ("☸☸( ◉_◉ )??", UiStyle::HIGHLIGHT),
                    Mood::Sick => ("☸☸( ×_× )!!", UiStyle::ERROR),
                    Mood::Idle => (if frame == 0 { "☸☸( ◉_◉ )☸☸" } else { "☸☸( ●_● )☸☸" }, UiStyle::TEXT_PRIMARY),
                };
                (face, style)
            }
            EvolutionStage::Adult => {
                let (face, style) = match self.mood {
                    Mood::Happy => (if frame == 0 { "☸☸☸( ◉ω◉ )☸☸☸" } else { "☸☸☸( ◉ω◉ )♥♥♥" }, UiStyle::OK),
                    Mood::Thinking => ("☸☸☸( ◉_◉ )???", UiStyle::HIGHLIGHT),
                    Mood::Sick => ("☸☸☸( ×_× )!!!", UiStyle::ERROR),
                    Mood::Idle => (if frame == 0 { "☸☸☸( ◉_◉ )☸☸☸" } else { "☸☸☸( ●_● )☸☸☸" }, UiStyle::HEADER),
                };
                (face, style)
            }
        }
    }

    fn domain_explanation(&self) -> &str {
        if self.domains.is_empty() {
            return "General Kubernetes skills.";
        }
        let first = self.domains[0].to_lowercase();
        if first.contains("storage") {
            "Persistent data matters. Pods\ndie, but volumes survive."
        } else if first.contains("network") {
            "Services connect pods. This is\nhow traffic reaches your apps."
        } else if first.contains("workload") || first.contains("scheduling") {
            "Workloads are the heart of k8s.\nDeployments, pods, containers."
        } else if first.contains("cluster") {
            "Cluster management keeps the\ncontrol plane healthy."
        } else if first.contains("security") || first.contains("rbac") {
            "Security locks things down.\nRBAC, secrets, policies."
        } else if first.contains("troubleshoot") {
            "Something's broken. Logs, events\nand describe are your friends."
        } else {
            "Core Kubernetes concepts for\nthe CKA exam."
        }
    }

    fn wrap_text(text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        for paragraph in text.split('\n') {
            let words: Vec<&str> = paragraph.split_whitespace().collect();
            if words.is_empty() {
                lines.push(String::new());
                continue;
            }
            let mut current = String::new();
            for word in words {
                if current.is_empty() {
                    current = word.to_string();
                } else if current.len() + 1 + word.len() <= width {
                    current.push(' ');
                    current.push_str(word);
                } else {
                    lines.push(current);
                    current = word.to_string();
                }
            }
            if !current.is_empty() {
                lines.push(current);
            }
        }
        lines
    }
}

impl Widget for KubeChanAssistant {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 3 {
            return;
        }

        let inner_width = area.width.saturating_sub(2) as usize;

        if area.height < 6 || area.width < 14 {
            let pulse = if (self.tick / 8).is_multiple_of(2) { "◉" } else { "●" };
            let summary = format!("{pulse} Kube-chan | {}%", self.readiness);
            Self::put_line(buf, area, 0, &summary, UiStyle::HIGHLIGHT);
            return;
        }

        let mut y = 0u16;

        let stage_label = match self.stage {
            EvolutionStage::Egg => "Kube-chan",
            EvolutionStage::Hatchling => "Kube-chan",
            EvolutionStage::Juvenile => "Kube-san",
            EvolutionStage::Adult => "Kube-sensei",
        };

        let header = format!(" ☸ {} ", stage_label);
        Self::put_centered(buf, area, y, &header, UiStyle::HEADER.add_modifier(Modifier::BOLD));
        y += 1;

        let (face, face_style) = self.creature_face();
        Self::put_centered(buf, area, y, face, face_style);
        y += 1;

        let bar_width = inner_width.min(20);
        let filled = (self.readiness as usize * bar_width) / 100;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!(" {}{}  {}%", "█".repeat(filled), "░".repeat(empty), self.readiness);
        let bar_style = if self.readiness >= 75 {
            UiStyle::OK
        } else if self.readiness >= 40 {
            UiStyle::WARNING
        } else {
            UiStyle::MUTED
        };
        Self::put_centered(buf, area, y, &bar, bar_style);
        y += 1;

        if y < area.height {
            let sep = "─".repeat(inner_width.min(area.width as usize));
            Self::put_line(buf, area, y, &format!(" {sep}"), UiStyle::BORDER);
            y += 1;
        }

        let domain_text = self.domain_explanation();
        let wrapped_domain = Self::wrap_text(domain_text, inner_width.saturating_sub(2));
        for line in &wrapped_domain {
            if y >= area.height.saturating_sub(1) {
                break;
            }
            Self::put_line(buf, area, y, &format!(" {line}"), UiStyle::TEXT_SECONDARY);
            y += 1;
        }

        if y < area.height.saturating_sub(1) {
            y += 1;
        }

        if let Some(tip) = &self.tip {
            let prefix = if self.hint_message.is_some() { "● Hint" } else { "● Tip" };
            let tip_text = format!("{}: {}", prefix, tip);
            let wrapped = Self::wrap_text(&tip_text, inner_width.saturating_sub(2));
            let tip_style = if self.hint_message.is_some() {
                UiStyle::WARNING
            } else {
                UiStyle::HIGHLIGHT
            };
            for line in &wrapped {
                if y >= area.height {
                    break;
                }
                Self::put_line(buf, area, y, &format!(" {line}"), tip_style);
                y += 1;
            }
        }
    }
}
