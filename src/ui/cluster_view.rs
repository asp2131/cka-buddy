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

    #[allow(dead_code)]
    fn put_centered(buf: &mut Buffer, area: Rect, y_off: u16, line: &str, style: Style) {
        if y_off >= area.height {
            return;
        }
        let maxw = area.width as usize;
        if maxw == 0 {
            return;
        }
        let char_count = line.chars().count();
        let pad = maxw.saturating_sub(char_count) / 2;
        let padded = format!("{}{}", " ".repeat(pad), line);
        let clipped: String = padded.chars().take(maxw).collect();
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
            let summary = format!("{pulse} {} │ {} pods", self.domain, self.pods.len());
            Self::put_line(buf, area, 0, &summary, UiStyle::HIGHLIGHT);
            return;
        }

        let heartbeat = if (self.tick / 8).is_multiple_of(2) { "◉" } else { "●" };
        let heartbeat2 = if (self.tick / 10).is_multiple_of(2) { "◉" } else { "●" };
        let crash = if (self.tick / 5).is_multiple_of(2) { "✖" } else { " " };
        let pending_frames = ["◌", "◔", "◑", "◕"];
        let pending = pending_frames[(self.tick / 4) % pending_frames.len()];

        let w = area.width as usize;
        let inner_w = w.saturating_sub(4);

        let mut y = 0u16;

        let cp_border = format!("┌─ CONTROL PLANE {}─┐", "─".repeat(inner_w.saturating_sub(18)));
        Self::put_line(buf, area, y, &cp_border, UiStyle::BORDER_ACTIVE);
        y += 1;

        let cp_status = format!(
            "│ API {heartbeat}  ETCD {heartbeat2}  SCHED {heartbeat}{}│",
            " ".repeat(inner_w.saturating_sub(22).max(0))
        );
        Self::put_line(buf, area, y, &cp_status, UiStyle::GLOW_GREEN);
        y += 1;

        let cp_bottom = format!("└{}┬{}┘", "─".repeat(inner_w / 2), "─".repeat(inner_w.saturating_sub(inner_w / 2 + 1)));
        Self::put_line(buf, area, y, &cp_bottom, UiStyle::BORDER_ACTIVE);
        y += 1;

        let connector_pad = inner_w / 2;
        let connector = format!("{}│", " ".repeat(connector_pad));
        Self::put_line(buf, area, y, &connector, UiStyle::MUTED);
        y += 1;

        let split_pad = inner_w / 4;
        let split_line = format!(
            "{}┌{}┴{}┐",
            " ".repeat(split_pad),
            "─".repeat(inner_w / 4),
            "─".repeat(inner_w / 4)
        );
        Self::put_line(buf, area, y, &split_line, UiStyle::MUTED);
        y += 1;

        let remaining = area.height.saturating_sub(y + 3) as usize;
        let max_pods = remaining.min(4).min(self.pods.len());

        let node_w = w.saturating_sub(2);
        let node_top = format!(
            " ┌─NODE-1{}┐",
            "─".repeat(node_w.saturating_sub(10))
        );
        Self::put_line(buf, area, y, &node_top, UiStyle::BORDER);
        y += 1;

        for pod in self.pods.iter().take(max_pods) {
            let icon = if pod.to_ascii_lowercase().contains("fail") || pod.to_ascii_lowercase().contains("crash") {
                crash
            } else if pod.to_ascii_lowercase().contains("pending") || pod.to_ascii_lowercase().contains("init") {
                pending
            } else {
                "☸"
            };

            let pod_name: String = pod.chars().take(node_w.saturating_sub(8)).collect();
            let pad = node_w.saturating_sub(pod_name.chars().count() + 6);
            let row = format!(" │ {icon} {pod_name}{}│", " ".repeat(pad));

            let pod_style = if icon == crash {
                UiStyle::ERROR
            } else if icon == pending {
                UiStyle::WARNING
            } else {
                UiStyle::GLOW_GREEN
            };
            Self::put_line(buf, area, y, &row, pod_style);
            y += 1;
            if y >= area.height.saturating_sub(3) {
                break;
            }
        }

        if max_pods == 0 {
            let empty_row = format!(
                " │ {} no pods{}│",
                pending,
                " ".repeat(node_w.saturating_sub(12))
            );
            Self::put_line(buf, area, y, &empty_row, UiStyle::MUTED);
            y += 1;
        }

        if y < area.height {
            let node_bot = format!(
                " └{}┬{}┘",
                "─".repeat(node_w.saturating_sub(3) / 2),
                "─".repeat(node_w.saturating_sub(3).saturating_sub(node_w.saturating_sub(3) / 2))
            );
            Self::put_line(buf, area, y, &node_bot, UiStyle::BORDER);
            y += 1;
        }

        if y < area.height {
            let svc_line = format!(
                "   SVC → :80  │  {} endpoints",
                self.pods.len()
            );
            Self::put_line(buf, area, y, &svc_line, UiStyle::COMMAND);
            y += 1;
        }

        if y < area.height {
            let ing_line = "   ING ▸ app.k8s.local";
            Self::put_line(buf, area, y, ing_line, UiStyle::DOMAIN_NETWORKING);
        }
    }
}
