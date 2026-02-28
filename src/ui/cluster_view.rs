use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::Widget,
};

use super::constants::UiStyle;

#[derive(Debug, Clone)]
pub struct ClusterView<'a> {
    pub title: &'a str,
    pub domain: &'a str,
    pub pods: &'a [String],
    pub tick: usize,
    pub style: Style,
}

impl<'a> ClusterView<'a> {
    pub fn new(title: &'a str, domain: &'a str, pods: &'a [String], tick: usize) -> Self {
        Self {
            title,
            domain,
            pods,
            tick,
            style: UiStyle::TEXT_PRIMARY,
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
}

impl Widget for ClusterView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 4 {
            return;
        }

        if area.height < 8 || area.width < 20 {
            let pulse = if (self.tick / 8).is_multiple_of(2) { "◉" } else { "●" };
            let summary = format!("{pulse} {} | {} pods", self.domain, self.pods.len());
            Self::put_line(buf, area, 0, &summary, UiStyle::HIGHLIGHT);
            return;
        }

        let heartbeat = if (self.tick / 8).is_multiple_of(2) { "◉" } else { "●" };
        let crash = if (self.tick / 5).is_multiple_of(2) { "✖" } else { " " };

        let mut y = 0u16;
        Self::put_line(
            buf,
            area,
            y,
            &format!("┌─ {} ─┐", self.title),
            UiStyle::BORDER,
        );
        y += 1;
        Self::put_line(
            buf,
            area,
            y,
            &format!("│ CP {heartbeat} ETCD {heartbeat} │"),
            UiStyle::TEXT_SECONDARY,
        );
        y += 1;
        Self::put_line(buf, area, y, "└──────┬───────┘", UiStyle::BORDER);
        y += 1;
        Self::put_line(buf, area, y, "       │", UiStyle::MUTED);
        y += 1;

        let remaining = area.height.saturating_sub(y + 3) as usize;
        let max_pods = remaining.min(4).min(self.pods.len());

        Self::put_line(buf, area, y, "┌─NODE-1────────┐", UiStyle::BORDER);
        y += 1;

        for pod in self.pods.iter().take(max_pods) {
            let icon = if pod.to_ascii_lowercase().contains("fail") {
                crash
            } else {
                "☸"
            };
            let row = format!("│ {icon} {}", pod);
            Self::put_line(buf, area, y, &row, self.style);
            y += 1;
            if y >= area.height.saturating_sub(2) {
                break;
            }
        }

        if y < area.height {
            Self::put_line(buf, area, y, "└──────┬────────┘", UiStyle::BORDER);
            y += 1;
        }
        if y < area.height {
            Self::put_line(buf, area, y, "   SVC api → :80", UiStyle::COMMAND);
        }
    }
}
