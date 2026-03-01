use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
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
    Eating,
}

#[derive(Debug, Clone)]
pub struct Tamagotchi {
    pub stage: EvolutionStage,
    pub mood: Mood,
    pub tick: usize,
    pub name: String,
    pub pod_count: usize,
    pub healthy_pods: usize,
    pub readiness: u8,
}

impl Tamagotchi {
    pub fn new(readiness: u8, pods: &[String], tick: usize) -> Self {
        let stage = EvolutionStage::from_readiness(readiness);
        let pod_count = pods.len();
        let fail_count = pods
            .iter()
            .filter(|p| p.to_ascii_lowercase().contains("fail"))
            .count();
        let healthy_pods = pod_count.saturating_sub(fail_count);

        let mood = if fail_count > 0 {
            Mood::Sick
        } else if pod_count > 0 && (tick / 20).is_multiple_of(8) {
            Mood::Eating
        } else if readiness > 50 {
            Mood::Happy
        } else {
            Mood::Idle
        };

        Self {
            stage,
            mood,
            tick,
            name: "Kube-chan".to_string(),
            pod_count,
            healthy_pods,
            readiness,
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

    fn creature_art(&self) -> (&[&str], Style) {
        let frame = (self.tick / 12) % 2;
        match self.stage {
            EvolutionStage::Egg => {
                let art = if frame == 0 {
                    EGG_FRAME_0
                } else {
                    EGG_FRAME_1
                };
                (art, UiStyle::TEXT_SECONDARY)
            }
            EvolutionStage::Hatchling => {
                let art = if frame == 0 {
                    HATCHLING_FRAME_0
                } else {
                    HATCHLING_FRAME_1
                };
                let style = match self.mood {
                    Mood::Happy => UiStyle::OK,
                    Mood::Sick => UiStyle::WARNING,
                    Mood::Eating => UiStyle::HIGHLIGHT,
                    Mood::Idle => UiStyle::TEXT_SECONDARY,
                };
                (art, style)
            }
            EvolutionStage::Juvenile => {
                let art = match self.mood {
                    Mood::Happy => {
                        if frame == 0 {
                            JUVENILE_HAPPY_0
                        } else {
                            JUVENILE_HAPPY_1
                        }
                    }
                    Mood::Sick => JUVENILE_SICK,
                    Mood::Eating => JUVENILE_EATING,
                    Mood::Idle => {
                        if frame == 0 {
                            JUVENILE_IDLE_0
                        } else {
                            JUVENILE_IDLE_1
                        }
                    }
                };
                let style = match self.mood {
                    Mood::Happy => UiStyle::OK,
                    Mood::Sick => UiStyle::ERROR,
                    Mood::Eating => UiStyle::HIGHLIGHT,
                    Mood::Idle => UiStyle::TEXT_PRIMARY,
                };
                (art, style)
            }
            EvolutionStage::Adult => {
                let art = match self.mood {
                    Mood::Happy => {
                        if frame == 0 {
                            ADULT_HAPPY_0
                        } else {
                            ADULT_HAPPY_1
                        }
                    }
                    Mood::Sick => ADULT_SICK,
                    Mood::Eating => ADULT_EATING,
                    Mood::Idle => {
                        if frame == 0 {
                            ADULT_IDLE_0
                        } else {
                            ADULT_IDLE_1
                        }
                    }
                };
                let style = match self.mood {
                    Mood::Happy => UiStyle::OK,
                    Mood::Sick => UiStyle::ERROR,
                    Mood::Eating => UiStyle::HIGHLIGHT,
                    Mood::Idle => UiStyle::HEADER,
                };
                (art, style)
            }
        }
    }

    fn status_line(&self) -> String {
        let heartbeat = if (self.tick / 8).is_multiple_of(2) { "◉" } else { "●" };
        let mood_icon = match self.mood {
            Mood::Happy => "♥",
            Mood::Idle => "~",
            Mood::Sick => "✖",
            Mood::Eating => "☸",
        };
        format!(
            "{} {} {}% | {} {}/{}",
            heartbeat, mood_icon, self.readiness, self.name, self.healthy_pods, self.pod_count
        )
    }
}

impl Widget for Tamagotchi {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 3 {
            return;
        }

        if area.height < 8 || area.width < 16 {
            let pulse = if (self.tick / 8).is_multiple_of(2) { "◉" } else { "●" };
            let mood = match self.mood {
                Mood::Happy => "♥",
                Mood::Idle => "~",
                Mood::Sick => "✖",
                Mood::Eating => "☸",
            };
            let summary = format!("{pulse} {mood} {}% | {}", self.readiness, self.name);
            Self::put_line(buf, area, 0, &summary, UiStyle::HIGHLIGHT);
            return;
        }

        let (art, creature_style) = self.creature_art();

        let mut y = 0u16;

        let stage_label = match self.stage {
            EvolutionStage::Egg => "EGG",
            EvolutionStage::Hatchling => "HATCHLING",
            EvolutionStage::Juvenile => "JUVENILE",
            EvolutionStage::Adult => "MASTER",
        };
        let header = format!("┌─ {} ─┐", stage_label);
        Self::put_line(buf, area, y, &header, UiStyle::BORDER);
        y += 1;

        for line in art {
            if y >= area.height.saturating_sub(3) {
                break;
            }
            Self::put_centered(buf, area, y, line, creature_style);
            y += 1;
        }

        if y < area.height.saturating_sub(2) {
            let footer = format!("└{}┘", "─".repeat(area.width.saturating_sub(2) as usize));
            Self::put_line(buf, area, y, &footer, UiStyle::BORDER);
            y += 1;
        }

        if y < area.height.saturating_sub(1) {
            Self::put_line(buf, area, y, &self.status_line(), UiStyle::TEXT_SECONDARY);
            y += 1;
        }

        if y < area.height {
            let bar_width = area.width.saturating_sub(4) as usize;
            let filled = (self.readiness as usize * bar_width) / 100;
            let empty = bar_width.saturating_sub(filled);
            let bar = format!(" [{}{}]", "█".repeat(filled), "░".repeat(empty));
            let bar_style = if self.readiness >= 75 {
                UiStyle::OK
            } else if self.readiness >= 40 {
                UiStyle::WARNING
            } else {
                UiStyle::MUTED
            };
            Self::put_line(buf, area, y, &bar, bar_style);
        }
    }
}

const EGG_FRAME_0: &[&str] = &[
    "    ╭───╮    ",
    "   ╱ ◉ ◉ ╲   ",
    "  │  ···  │  ",
    "  │  ☸    │  ",
    "   ╲     ╱   ",
    "    ╰───╯    ",
];

const EGG_FRAME_1: &[&str] = &[
    "    ╭───╮    ",
    "   ╱ ● ● ╲   ",
    "  │  ···  │  ",
    "  │    ☸  │  ",
    "   ╲     ╱   ",
    "    ╰───╯    ",
];

const HATCHLING_FRAME_0: &[&str] = &[
    "     ∧∧      ",
    "   ( ◉‿◉)    ",
    "   /|☸ |\\   ",
    "    | ☸|     ",
    "    /  \\     ",
];

const HATCHLING_FRAME_1: &[&str] = &[
    "     ∧∧      ",
    "   ( ●‿●)    ",
    "   /|  |\\   ",
    "    |☸ |     ",
    "    /  \\     ",
];

const JUVENILE_HAPPY_0: &[&str] = &[
    "    ╱╲╱╲     ",
    "   ( ◉ω◉)    ",
    "  ╱|☸☸☸|╲   ",
    "   |    |    ",
    "   ╱╲  ╱╲    ",
    "  ╱  ╲╱  ╲   ",
];

const JUVENILE_HAPPY_1: &[&str] = &[
    "    ╱╲╱╲     ",
    "   ( ◉ω◉) ♥  ",
    "  ╱|☸☸☸|╲   ",
    "   |    |    ",
    "   ╱╲  ╱╲    ",
    "  ╱  ╲╱  ╲   ",
];

const JUVENILE_SICK: &[&str] = &[
    "    ╱╲╱╲     ",
    "   ( ×_×)    ",
    "  ╱|✖✖✖|╲   ",
    "   | ~~ |    ",
    "   ╱╲  ╱╲    ",
    "  ╱  ╲╱  ╲   ",
];

const JUVENILE_EATING: &[&str] = &[
    "    ╱╲╱╲     ",
    "   ( ◉○◉)    ",
    "  ╱|☸☸ |╲   ",
    "   | ☸☸|    ",
    "   ╱╲  ╱╲    ",
    "  ╱  ╲╱  ╲   ",
];

const JUVENILE_IDLE_0: &[&str] = &[
    "    ╱╲╱╲     ",
    "   ( ◉_◉)    ",
    "  ╱|☸☸ |╲   ",
    "   |    |    ",
    "   ╱╲  ╱╲    ",
    "  ╱  ╲╱  ╲   ",
];

const JUVENILE_IDLE_1: &[&str] = &[
    "    ╱╲╱╲     ",
    "   ( ●_●)    ",
    "  ╱|☸☸ |╲   ",
    "   |    |    ",
    "   ╱╲  ╱╲    ",
    "  ╱  ╲╱  ╲   ",
];

const ADULT_HAPPY_0: &[&str] = &[
    "   ╱☸╲╱☸╲    ",
    "  ╱ ╱╲╱╲ ╲   ",
    "  ( ◉ ω ◉ )  ",
    " ╱|☸☸☸☸☸|╲  ",
    "  | ◉API◉ |  ",
    "  |  ETCD  |  ",
    "  ╱╲ ☸☸ ╱╲   ",
    " ╱  ╲──╱  ╲  ",
];

const ADULT_HAPPY_1: &[&str] = &[
    "   ╱☸╲╱☸╲    ",
    "  ╱ ╱╲╱╲ ╲   ",
    "  ( ◉ ω ◉ )♥ ",
    " ╱|☸☸☸☸☸|╲  ",
    "  | ◉API◉ |  ",
    "  |  ETCD  |  ",
    "  ╱╲ ☸☸ ╱╲   ",
    " ╱  ╲──╱  ╲  ",
];

const ADULT_SICK: &[&str] = &[
    "   ╱✖╲╱✖╲    ",
    "  ╱ ╱╲╱╲ ╲   ",
    "  ( × _ × )  ",
    " ╱|✖✖✖✖✖|╲  ",
    "  |  ~~~~  |  ",
    "  | CRASH! |  ",
    "  ╱╲ ~~ ╱╲   ",
    " ╱  ╲──╱  ╲  ",
];

const ADULT_EATING: &[&str] = &[
    "   ╱☸╲╱☸╲    ",
    "  ╱ ╱╲╱╲ ╲   ",
    "  ( ◉ ○ ◉ )  ",
    " ╱|☸☸☸☸☸|╲  ",
    "  |  ☸☸☸  |  ",
    "  | ☸ETCD☸|  ",
    "  ╱╲ ☸☸ ╱╲   ",
    " ╱  ╲──╱  ╲  ",
];

const ADULT_IDLE_0: &[&str] = &[
    "   ╱☸╲╱☸╲    ",
    "  ╱ ╱╲╱╲ ╲   ",
    "  ( ◉ _ ◉ )  ",
    " ╱|☸☸☸☸☸|╲  ",
    "  | ◉API◉ |  ",
    "  |  ETCD  |  ",
    "  ╱╲ ☸☸ ╱╲   ",
    " ╱  ╲──╱  ╲  ",
];

const ADULT_IDLE_1: &[&str] = &[
    "   ╱☸╲╱☸╲    ",
    "  ╱ ╱╲╱╲ ╲   ",
    "  ( ● _ ● )  ",
    " ╱|☸☸☸☸☸|╲  ",
    "  | ◉API◉ |  ",
    "  |  ETCD  |  ",
    "  ╱╲ ☸☸ ╱╲   ",
    " ╱  ╲──╱  ╲  ",
];
