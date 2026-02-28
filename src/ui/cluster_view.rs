use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use super::constants::UiStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentState {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodStatus {
    Running,
    Pending,
    CrashLoop,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Pod {
    pub name: String,
    pub status: PodStatus,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub pods: Vec<Pod>,
    pub status: ComponentState,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub enum ExtraResource {
    Ingress { host: String },
    NetworkPolicy { name: String },
    PersistentVolume { name: String, capacity: String },
    Secret { name: String },
    ConfigMap { name: String },
}

#[derive(Debug, Clone)]
pub struct ControlPlane {
    pub api_server: ComponentState,
    pub etcd: ComponentState,
    pub scheduler: ComponentState,
    pub controller: ComponentState,
}

impl Default for ControlPlane {
    fn default() -> Self {
        Self {
            api_server: ComponentState::Healthy,
            etcd: ComponentState::Healthy,
            scheduler: ComponentState::Healthy,
            controller: ComponentState::Healthy,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClusterScene {
    pub control_plane: ControlPlane,
    pub nodes: Vec<Node>,
    pub services: Vec<Service>,
    pub extras: Vec<ExtraResource>,
    pub tick: usize,
    pub domain: String,
    pub flash_remaining: usize,
}


impl ClusterScene {
    pub fn for_domain(domain: &str, commands: &[String], completed_count: usize) -> Self {
        let lower = domain.to_lowercase();
        let pod_names = extract_pod_names(commands);

        if lower.contains("network") {
            Self::networking_scene(&pod_names, completed_count)
        } else if lower.contains("workload") || lower.contains("scheduling") {
            Self::workloads_scene(&pod_names, completed_count)
        } else if lower.contains("storage") {
            Self::storage_scene(&pod_names, completed_count)
        } else if lower.contains("security") || lower.contains("rbac") {
            Self::security_scene(&pod_names, completed_count)
        } else if lower.contains("troubleshoot") {
            Self::troubleshooting_scene(&pod_names)
        } else {
            Self::cluster_overview_scene(&pod_names, completed_count)
        }
    }

    fn cluster_overview_scene(pod_names: &[String], completed: usize) -> Self {
        let pods1: Vec<Pod> = pod_names
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, name)| Pod {
                name: name.clone(),
                status: if i < completed.min(3) {
                    PodStatus::Running
                } else {
                    PodStatus::Pending
                },
            })
            .collect();
        let pods2 = vec![
            Pod { name: "etcd-0".into(), status: PodStatus::Running },
            Pod { name: "coredns".into(), status: PodStatus::Running },
        ];

        Self {
            control_plane: ControlPlane::default(),
            nodes: vec![
                Node { name: "node-1".into(), pods: if pods1.is_empty() { vec![Pod { name: "app".into(), status: PodStatus::Running }] } else { pods1 }, status: ComponentState::Healthy },
                Node { name: "node-2".into(), pods: pods2, status: ComponentState::Healthy },
            ],
            services: vec![Service { name: "kubernetes".into(), port: 443 }],
            extras: vec![],
            domain: "cluster".into(),
            ..Default::default()
        }
    }

    fn workloads_scene(pod_names: &[String], completed: usize) -> Self {
        let make_pods = |names: &[String], base: usize| -> Vec<Pod> {
            names.iter().enumerate().map(|(i, n)| Pod {
                name: n.clone(),
                status: if i + base < completed { PodStatus::Running } else { PodStatus::Pending },
            }).collect()
        };

        let half = pod_names.len().min(4) / 2;
        let (first, second) = pod_names.split_at(half.max(1).min(pod_names.len()));

        Self {
            control_plane: ControlPlane::default(),
            nodes: vec![
                Node {
                    name: "node-1".into(),
                    pods: if first.is_empty() {
                        vec![Pod { name: "web-0".into(), status: PodStatus::Running }, Pod { name: "web-1".into(), status: PodStatus::Pending }]
                    } else {
                        make_pods(first, 0)
                    },
                    status: ComponentState::Healthy,
                },
                Node {
                    name: "node-2".into(),
                    pods: if second.is_empty() {
                        vec![Pod { name: "worker-0".into(), status: PodStatus::Running }]
                    } else {
                        make_pods(second, half)
                    },
                    status: ComponentState::Healthy,
                },
            ],
            services: vec![Service { name: "web-svc".into(), port: 80 }],
            extras: vec![],
            domain: "workloads".into(),
            ..Default::default()
        }
    }

    fn networking_scene(pod_names: &[String], completed: usize) -> Self {
        let pods: Vec<Pod> = pod_names.iter().take(2).enumerate().map(|(i, n)| Pod {
            name: n.clone(),
            status: if i < completed { PodStatus::Running } else { PodStatus::Pending },
        }).collect();

        Self {
            control_plane: ControlPlane::default(),
            nodes: vec![
                Node {
                    name: "node-1".into(),
                    pods: if pods.is_empty() { vec![Pod { name: "nginx".into(), status: PodStatus::Running }, Pod { name: "redis".into(), status: PodStatus::Running }] } else { pods },
                    status: ComponentState::Healthy,
                },
                Node {
                    name: "node-2".into(),
                    pods: vec![Pod { name: "api".into(), status: PodStatus::Running }, Pod { name: "worker".into(), status: PodStatus::Pending }],
                    status: ComponentState::Healthy,
                },
            ],
            services: vec![
                Service { name: "nginx-svc".into(), port: 80 },
            ],
            extras: vec![ExtraResource::Ingress { host: "app.k8s.io".into() }],
            domain: "networking".into(),
            ..Default::default()
        }
    }

    fn storage_scene(pod_names: &[String], completed: usize) -> Self {
        let pods: Vec<Pod> = pod_names.iter().take(2).enumerate().map(|(i, n)| Pod {
            name: n.clone(),
            status: if i < completed { PodStatus::Running } else { PodStatus::Pending },
        }).collect();

        Self {
            control_plane: ControlPlane::default(),
            nodes: vec![
                Node {
                    name: "node-1".into(),
                    pods: if pods.is_empty() { vec![Pod { name: "db-0".into(), status: PodStatus::Running }] } else { pods },
                    status: ComponentState::Healthy,
                },
            ],
            services: vec![],
            extras: vec![
                ExtraResource::PersistentVolume { name: "data-vol".into(), capacity: "10Gi".into() },
                ExtraResource::PersistentVolume { name: "log-vol".into(), capacity: "5Gi".into() },
            ],
            domain: "storage".into(),
            ..Default::default()
        }
    }

    fn security_scene(pod_names: &[String], _completed: usize) -> Self {
        let pods: Vec<Pod> = pod_names.iter().take(2).map(|n| Pod {
            name: n.clone(),
            status: PodStatus::Running,
        }).collect();

        Self {
            control_plane: ControlPlane::default(),
            nodes: vec![
                Node {
                    name: "node-1".into(),
                    pods: if pods.is_empty() { vec![Pod { name: "app".into(), status: PodStatus::Running }] } else { pods },
                    status: ComponentState::Healthy,
                },
            ],
            services: vec![],
            extras: vec![
                ExtraResource::Secret { name: "tls-cert".into() },
                ExtraResource::NetworkPolicy { name: "deny-all".into() },
            ],
            domain: "security".into(),
            ..Default::default()
        }
    }

    fn troubleshooting_scene(pod_names: &[String]) -> Self {
        let mut pods1: Vec<Pod> = pod_names.iter().take(2).map(|n| Pod {
            name: n.clone(),
            status: PodStatus::CrashLoop,
        }).collect();
        if pods1.is_empty() {
            pods1 = vec![
                Pod { name: "crash-1".into(), status: PodStatus::CrashLoop },
                Pod { name: "pending".into(), status: PodStatus::Pending },
            ];
        }

        Self {
            control_plane: ControlPlane {
                api_server: ComponentState::Healthy,
                etcd: ComponentState::Degraded,
                scheduler: ComponentState::Healthy,
                controller: ComponentState::Degraded,
            },
            nodes: vec![
                Node { name: "node-1".into(), pods: pods1, status: ComponentState::Degraded },
                Node { name: "node-2".into(), pods: vec![Pod { name: "ok-pod".into(), status: PodStatus::Running }], status: ComponentState::Healthy },
            ],
            services: vec![],
            extras: vec![],
            domain: "troubleshooting".into(),
            ..Default::default()
        }
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        if self.flash_remaining > 0 {
            self.flash_remaining -= 1;
        }
    }

    pub fn trigger_flash(&mut self) {
        self.flash_remaining = 20;
    }
}

fn extract_pod_names(commands: &[String]) -> Vec<String> {
    let keywords = ["nginx", "redis", "mysql", "postgres", "app", "web", "api",
                     "worker", "cache", "proxy", "db", "backend", "frontend"];
    let mut found = Vec::new();

    for cmd in commands {
        let lower = cmd.to_lowercase();
        for kw in &keywords {
            if lower.contains(kw) && !found.contains(&kw.to_string()) {
                found.push(kw.to_string());
            }
        }
    }

    if found.is_empty() {
        found.push("app".to_string());
    }

    found.truncate(4);
    found
}

fn ellipsize(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else if max <= 2 {
        text.chars().take(max).collect()
    } else {
        format!("{}..", &text[..max - 2])
    }
}

fn component_icon(state: ComponentState, tick: usize) -> &'static str {
    match state {
        ComponentState::Healthy => {
            if tick % 20 < 10 { "◉" } else { "●" }
        }
        ComponentState::Degraded => "◎",
        ComponentState::Down => "○",
    }
}

fn component_style(state: ComponentState) -> Style {
    match state {
        ComponentState::Healthy => UiStyle::OK,
        ComponentState::Degraded => UiStyle::WARNING,
        ComponentState::Down => UiStyle::ERROR,
    }
}

fn pod_icon(status: PodStatus, tick: usize) -> &'static str {
    match status {
        PodStatus::Running => "☸",
        PodStatus::Pending => {
            let spinner = ["◌", "◔", "◑", "◕"];
            spinner[(tick / 5) % spinner.len()]
        }
        PodStatus::CrashLoop => {
            if tick % 10 < 5 { "✖" } else { " " }
        }
        PodStatus::Completed => "✓",
    }
}

fn pod_style(status: PodStatus, flash: bool) -> Style {
    if flash {
        return UiStyle::OK;
    }
    match status {
        PodStatus::Running => UiStyle::OK,
        PodStatus::Pending => UiStyle::WARNING,
        PodStatus::CrashLoop => UiStyle::ERROR,
        PodStatus::Completed => UiStyle::OK.add_modifier(Modifier::DIM),
    }
}

fn domain_accent(domain: &str) -> Style {
    let lower = domain.to_lowercase();
    if lower.contains("network") {
        UiStyle::DOMAIN_NETWORKING
    } else if lower.contains("workload") || lower.contains("scheduling") {
        UiStyle::DOMAIN_WORKLOADS
    } else if lower.contains("storage") {
        UiStyle::DOMAIN_STORAGE
    } else if lower.contains("security") || lower.contains("rbac") {
        UiStyle::DOMAIN_SECURITY
    } else if lower.contains("troubleshoot") {
        UiStyle::DOMAIN_TROUBLESHOOTING
    } else {
        UiStyle::DOMAIN_CLUSTER
    }
}

impl Widget for &ClusterScene {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 6 || area.width < 12 {
            return;
        }

        let flash = self.flash_remaining > 0;
        let w = area.width as usize;
        let accent = domain_accent(&self.domain);

        if area.height < 12 {
            render_micro(self, area, buf, flash);
            return;
        }

        let mut y = area.y;
        let budget = area.height as usize;
        let mut used: usize = 0;

        let cp_h = if budget >= 20 { 4 } else { 2 };
        if used + cp_h <= budget {
            render_control_plane(self, area.x, y, w, cp_h, buf, accent);
            y += cp_h as u16;
            used += cp_h;
        }

        if used < budget {
            let mid = area.x + (w as u16) / 2;
            buf.set_string(mid, y, "│", UiStyle::BORDER);
            y += 1;
            used += 1;
        }

        let max_nodes = if budget >= 20 { 2 } else { 1 };
        for node in self.nodes.iter().take(max_nodes) {
            let max_pods = node.pods.len().min(4);
            let node_h = 2 + max_pods;
            if used + node_h > budget {
                break;
            }
            render_node(node, area.x, y, w, self.tick, flash, buf);
            y += node_h as u16;
            used += node_h;
        }

        for svc in self.services.iter().take(2) {
            if used + 3 > budget {
                break;
            }
            render_service(svc, area.x, y, w, buf, accent);
            y += 3;
            used += 3;
        }

        for extra in self.extras.iter().take(2) {
            if used + 2 > budget {
                break;
            }
            render_extra(extra, area.x, y, w, buf, accent);
            y += 2;
            used += 2;
        }
    }
}

fn render_micro(scene: &ClusterScene, area: Rect, buf: &mut Buffer, flash: bool) {
    let running = scene.nodes.iter().flat_map(|n| &n.pods).filter(|p| p.status == PodStatus::Running).count();
    let total = scene.nodes.iter().flat_map(|n| &n.pods).count();
    let healthy = scene.control_plane.api_server == ComponentState::Healthy;

    let status_icon = if healthy { "◉" } else { "◎" };
    let status_style = if healthy { UiStyle::OK } else { UiStyle::WARNING };
    let pod_style_val = if flash { UiStyle::OK } else { UiStyle::TEXT_PRIMARY };

    let line = Line::from(vec![
        Span::styled(" ☸ ", pod_style_val),
        Span::styled(format!("{running}/{total} pods"), pod_style_val),
        Span::styled(" │ ", UiStyle::BORDER),
        Span::styled(status_icon, status_style),
        Span::styled(if healthy { " healthy" } else { " degraded" }, status_style),
    ]);

    buf.set_line(area.x, area.y + area.height / 2, &line, area.width);
}

fn render_control_plane(
    scene: &ClusterScene,
    x: u16,
    y: u16,
    w: usize,
    height: usize,
    buf: &mut Buffer,
    accent: Style,
) {
    let cp = &scene.control_plane;
    let tick = scene.tick;
    let box_w = w.min(28);
    let offset = x + ((w - box_w) / 2) as u16;

    let title = " CONTROL PLANE ";
    let left_pad = (box_w.saturating_sub(title.len() + 2)) / 2;
    let right_pad = box_w.saturating_sub(left_pad + title.len() + 2);
    let top = format!(
        "┌{}{}{}┐",
        "─".repeat(left_pad),
        title,
        "─".repeat(right_pad),
    );
    buf.set_string(offset, y, &top[..top.len().min(box_w + 2)], accent);

    if height >= 4 {
        let row1 = format!(
            "│ API {} ETCD {}{}│",
            component_icon(cp.api_server, tick),
            component_icon(cp.etcd, tick),
            " ".repeat(box_w.saturating_sub(18))
        );
        let row1_trunc = &row1[..row1.chars().count().min(box_w + 2)];
        buf.set_string(offset, y + 1, row1_trunc, UiStyle::TEXT_SECONDARY);
        buf.set_string(offset + 6, y + 1, component_icon(cp.api_server, tick), component_style(cp.api_server));
        buf.set_string(offset + 13, y + 1, component_icon(cp.etcd, tick), component_style(cp.etcd));

        let row2 = format!(
            "│ SCHED {} CTRL {}{}│",
            component_icon(cp.scheduler, tick),
            component_icon(cp.controller, tick),
            " ".repeat(box_w.saturating_sub(20))
        );
        let row2_trunc = &row2[..row2.chars().count().min(box_w + 2)];
        buf.set_string(offset, y + 2, row2_trunc, UiStyle::TEXT_SECONDARY);
        buf.set_string(offset + 8, y + 2, component_icon(cp.scheduler, tick), component_style(cp.scheduler));
        buf.set_string(offset + 15, y + 2, component_icon(cp.controller, tick), component_style(cp.controller));

        let bottom = format!("└{}┘", "─".repeat(box_w.saturating_sub(2)));
        buf.set_string(offset, y + 3, &bottom, accent);
    } else {
        let compact = format!(
            "│ API {} ETD {} SCH {} CTL {} │",
            component_icon(cp.api_server, tick),
            component_icon(cp.etcd, tick),
            component_icon(cp.scheduler, tick),
            component_icon(cp.controller, tick),
        );
        buf.set_string(offset, y + 1, &compact[..compact.chars().count().min(w)], UiStyle::TEXT_SECONDARY);
    }
}

fn render_node(
    node: &Node,
    x: u16,
    y: u16,
    w: usize,
    tick: usize,
    flash: bool,
    buf: &mut Buffer,
) {
    let box_w = w.min(24);
    let offset = x + ((w - box_w) / 2) as u16;
    let inner_w = box_w.saturating_sub(4);

    let name = ellipsize(&node.name.to_uppercase(), box_w.saturating_sub(4));
    let top = format!("┌─{}─{}┐", name, "─".repeat(box_w.saturating_sub(name.len() + 4)));
    let node_style = component_style(node.status);
    buf.set_string(offset, y, &top[..top.len().min(box_w + 1)], node_style);

    for (i, pod) in node.pods.iter().take(4).enumerate() {
        let icon = pod_icon(pod.status, tick);
        let pname = ellipsize(&pod.name, inner_w.saturating_sub(2));
        let padding = inner_w.saturating_sub(pname.len() + 2);
        let row = format!("│ {} {}{}│", icon, pname, " ".repeat(padding));
        let row_y = y + 1 + i as u16;
        buf.set_string(offset, row_y, &row[..row.len().min(box_w + 1)], UiStyle::TEXT_SECONDARY);
        buf.set_string(offset + 2, row_y, icon, pod_style(pod.status, flash));
    }

    let pod_count = node.pods.len().min(4);
    let bottom = format!("└{}┘", "─".repeat(box_w.saturating_sub(2)));
    buf.set_string(offset, y + 1 + pod_count as u16, &bottom, node_style);
}

fn render_service(
    svc: &Service,
    x: u16,
    y: u16,
    w: usize,
    buf: &mut Buffer,
    accent: Style,
) {
    let mid = x + (w as u16) / 2;
    buf.set_string(mid, y, "│", UiStyle::BORDER);

    let label = format!("SVC {} → :{}", ellipsize(&svc.name, 10), svc.port);
    let box_w = label.len() + 4;
    let offset = x + ((w.saturating_sub(box_w)) / 2) as u16;
    let top = format!("┌{}┐", "─".repeat(box_w.saturating_sub(2)));
    let mid_row = format!("│ {} │", label);
    let bottom = format!("└{}┘", "─".repeat(box_w.saturating_sub(2)));
    buf.set_string(offset, y + 1, &top, accent);
    buf.set_string(offset, y + 2, &mid_row, UiStyle::TEXT_PRIMARY);
    buf.set_string(offset, y + 3 - 1, &bottom, accent);
}

fn render_extra(
    extra: &ExtraResource,
    x: u16,
    y: u16,
    w: usize,
    buf: &mut Buffer,
    accent: Style,
) {
    match extra {
        ExtraResource::Ingress { host } => {
            let label = format!("ING {}", ellipsize(host, w.saturating_sub(8)));
            let box_w = label.len() + 4;
            let offset = x + ((w.saturating_sub(box_w)) / 2) as u16;
            let mid_x = x + (w as u16) / 2;
            buf.set_string(mid_x, y, "│", UiStyle::BORDER);
            let row = format!("  {} ", label);
            buf.set_string(offset, y + 1, &row, accent);
        }
        ExtraResource::PersistentVolume { name, capacity } => {
            let label = format!("PV {} [{}]", ellipsize(name, 8), capacity);
            let bar_w = w.saturating_sub(4).min(20);
            let filled = bar_w * 7 / 10;
            let empty = bar_w - filled;
            let offset = x + 1;
            buf.set_string(offset, y, format!("  {}", label), UiStyle::TEXT_PRIMARY);
            let bar = format!("  {}{}", "▰".repeat(filled), "▱".repeat(empty));
            buf.set_string(offset, y + 1, bar, accent);
        }
        ExtraResource::NetworkPolicy { name } => {
            let label = format!("⊠ NetPol: {}", ellipsize(name, w.saturating_sub(14)));
            buf.set_string(x + 1, y, format!("  {}", label), accent);
        }
        ExtraResource::Secret { name } => {
            let label = format!("⊠ Secret: {}", ellipsize(name, w.saturating_sub(14)));
            buf.set_string(x + 1, y, format!("  {}", label), UiStyle::WARNING);
        }
        ExtraResource::ConfigMap { name } => {
            let label = format!("  CM: {}", ellipsize(name, w.saturating_sub(10)));
            buf.set_string(x + 1, y, &label, UiStyle::TEXT_SECONDARY);
        }
    }
}
