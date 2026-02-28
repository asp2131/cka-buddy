# Plan: Interactive Learning UX — Rebels Pattern Upgrade

## Task Description
Upgrade the CKA Buddy learning interface from a keyboard-only, static-rendering TUI to a fully interactive, mouse-enabled, callback-driven experience using the design patterns from `rebels-in-the-sky`. The learning screen should feel alive — hoverable commands, clickable action buttons with hotkey underlines, a context-sensitive footer bar, hover help text, mouse-aware scroll/navigation, and a **cartoon ASCII art Kubernetes cluster visualization** in the activity rail that reacts to the current step's domain. This transforms the experience from "typing slash commands into a text box" to "clicking and hovering through a rich interactive dashboard with a living cluster diagram."

## Objective
When complete, the CKA Buddy learning screen will support:
- Mouse hover, click, and scroll on interactive elements
- Clickable runbook commands that auto-fill the command bar
- Action buttons (Verify, Hint, Next, Prev) with visible hotkey underlines
- A persistent footer bar showing available keyboard shortcuts
- A hover help text row showing contextual micro-help for hovered elements
- Interaction layers so popups block background clicks
- A callback registry routing all UI intents through a single dispatcher
- **A cartoony ASCII art cluster visualization** in the activity rail showing nodes, pods, services, ingresses, and control plane components — styled per CKA domain and animated with a tick-based heartbeat

## Problem Statement
The current learning interface requires memorizing slash commands (`/verify`, `/hint`, `/next`, `/suggest`) with no visual affordance beyond the help popup. There's no mouse interaction, no hover feedback, no visual discoverability of available actions. The activity rail is a plain text summary that duplicates main panel content. The Rebels-in-the-Sky reference project solves the interaction problems with an interactive widget system, callback registry, and layered frame wrapper. For the visual gap, we'll build a cartoon cluster widget inspired by Rebels' spatial layout patterns (tournament brackets, bar gauges, box-drawing connectors).

## Solution Approach
Port the Rebels interaction infrastructure (UiFrame wrapper, CallbackRegistry, InteractiveWidget trait, Button with hotkey underlines) into CKA Buddy, then rebuild the learning screen's render pipeline to use interactive widgets. Replace the activity rail with a cartoon cluster visualization that changes based on the current step's CKA domain. This is an incremental refactor — existing keyboard input still works, mouse support is additive.

## Relevant Files

### Existing Files to Modify
- `src/ui/mod.rs` — Add new module declarations (callback, ui_frame, interactive traits)
- `src/ui/traits.rs` — Extend Screen trait to accept `UiFrame` instead of raw `Frame`
- `src/ui/button.rs` — Upgrade to interactive button with hotkey, hover, click callbacks
- `src/ui/widgets.rs` — Add footer hint rendering helpers
- `src/ui/ui_action.rs` — Expand with new callback-driven actions (SetCommandInput, SetPanelIndex, etc.)
- `src/ui/ui_screen.rs` — Integrate UiFrame, footer bar, hover text row, mouse event routing
- `src/ui/learning_screen.rs` — Rebuild render pipeline with interactive widgets
- `src/ui/popup.rs` — Add layer awareness to popup rendering
- `src/ui/constants.rs` — Add new style constants (HOVER, FOOTER_KEY, FOOTER_DESC, BUTTON_ACTIVE)
- `src/app/mod.rs` — Enable mouse capture, route mouse events through callback registry

### New Files to Create
- `src/ui/callback_registry.rs` — CallbackRegistry struct with mouse/keyboard callback storage and layer filtering
- `src/ui/ui_frame.rs` — UiFrame wrapper around ratatui::Frame with hover tracking, callback registry, and interactive render methods
- `src/ui/clickable_list.rs` — Interactive list widget for runbook commands with per-item click/hover
- `src/ui/cluster_view.rs` — Cartoon ASCII art Kubernetes cluster visualization widget with domain-aware scenes

### Reference Files (read-only)
- `rebels-in-the-sky/src/ui/ui_frame.rs` — Frame wrapper pattern
- `rebels-in-the-sky/src/ui/ui_callback.rs` — Callback enum and routing
- `rebels-in-the-sky/src/ui/traits.rs` — InteractiveWidget, SplitPanel traits
- `rebels-in-the-sky/src/ui/button.rs` — Button with hotkey underline and hover
- `rebels-in-the-sky/src/ui/clickable_list.rs` — List with click/scroll/hover
- `rebels-in-the-sky/src/ui/constants.rs` — UiStyle semantic palette

## Implementation Phases

### Phase 1: Interaction Infrastructure
Build the callback registry, UiFrame wrapper, and InteractiveWidget trait. These are the foundation — no visible changes yet, but the plumbing is in place.

### Phase 2: Interactive Widgets
Upgrade Button to support hotkeys, hover, and click callbacks. Build ClickableList for runbook commands. Wire the learning screen's render pipeline to use `UiFrame` and interactive widgets.

### Phase 3: Footer, Hover Help & Mouse Events
Add the persistent footer bar with alternating key hints, the hover help text row, and enable mouse capture in the main event loop. Route mouse events through the callback registry.

### Phase 4: Cartoon Cluster Visualization
Build the ASCII art cluster widget that renders in the activity rail. Domain-aware scenes show different K8s components based on the current step. Animated heartbeat on the control plane. Pods appear/disappear when steps are completed.

### Phase 5: Polish & Popups
Add layer-aware popup rendering so modals block background clicks. Add celebration enhancements. Verify keyboard-only flow still works perfectly.

## Step by Step Tasks

### 1. Add Style Constants for Interactive Elements
- In `src/ui/constants.rs`, add these styles:
  ```rust
  pub const HOVER: Style = Self::DEFAULT.bg(Color::Rgb(45, 50, 70));
  pub const FOOTER_KEY: Style = Self::DEFAULT.fg(Color::Rgb(14, 18, 28)).bg(Color::Rgb(140, 150, 170));
  pub const FOOTER_DESC: Style = Self::DEFAULT.fg(Color::Rgb(200, 205, 215)).bg(Color::Rgb(35, 40, 55));
  pub const BUTTON_ACTIVE: Style = Self::DEFAULT.fg(Color::Rgb(118, 213, 192));
  pub const HOTKEY_UNDERLINE: Style = Self::DEFAULT.fg(Color::Rgb(255, 220, 100));
  ```

### 2. Create CallbackRegistry (`src/ui/callback_registry.rs`)
- Define `CallbackRegistry` struct storing:
  - `mouse_callbacks: Vec<(MouseEventKind, Option<Rect>, UiAction)>` — scoped or global mouse bindings
  - `keyboard_callbacks: Vec<(KeyCode, UiAction)>` — hotkey bindings
  - `mouse_position: Option<(u16, u16)>` — current cursor position
  - `active_layer: usize` — which interaction layer is active (0=main, 1=popup)
- Implement methods:
  - `register_mouse_callback(kind, area, action)` — register a scoped or global mouse callback
  - `register_keyboard_callback(key, action)` — register a hotkey
  - `is_hovering(area: Rect) -> bool` — check if mouse is within area
  - `set_mouse_position(x, y)` — update from mouse move events
  - `set_active_layer(layer)` / `get_active_layer() -> usize`
  - `resolve_mouse_event(kind, x, y) -> Option<UiAction>` — find matching callback for mouse event
  - `resolve_key_event(key) -> Option<UiAction>` — find matching hotkey callback
  - `clear()` — reset all callbacks for next frame

### 3. Create UiFrame Wrapper (`src/ui/ui_frame.rs`)
- Define `UiFrame<'a, 'b>` wrapping `&'a mut Frame<'b>` with:
  - `callback_registry: CallbackRegistry`
  - `hover_text_area: Rect` — where to render micro-help
- Methods:
  - `render_widget(widget, area)` — pass-through to inner frame
  - `render_stateful_widget(widget, area, state)` — pass-through
  - `render_interactive_widget(widget, area)` — call `before_rendering`, check hover, render hover text, render widget
  - `render_stateful_interactive_widget(widget, area, state)` — same for stateful
  - `set_active_layer(layer)` — delegate to registry
  - `is_hovered(area, layer) -> bool` — delegate to registry with layer check
  - `area() -> Rect` — delegate to inner frame
  - `registry_mut() -> &mut CallbackRegistry` — direct access for custom widgets
  - `into_registry(self) -> CallbackRegistry` — consume frame, return registry for event processing

### 4. Define InteractiveWidget Trait (`src/ui/traits.rs`)
- Add to existing traits.rs:
  ```rust
  pub trait InteractiveWidget: Widget {
      fn layer(&self) -> usize { 0 }
      fn before_rendering(&mut self, area: Rect, registry: &mut CallbackRegistry);
      fn hover_text(&self) -> Text<'_> { Text::default() }
  }
  ```
- Update `Screen` trait to accept `UiFrame` instead of `Frame`:
  ```rust
  fn render(&mut self, frame: &mut UiFrame, engine: &Engine, area: Rect) -> Result<()>;
  ```
- Add `footer_spans(&self) -> Vec<(String, String)>` method returning key-description pairs for footer rendering

### 5. Expand UiAction Enum (`src/ui/ui_action.rs`)
- Add new variants for callback-driven interactions:
  ```rust
  SetCommandInput(String),    // Auto-fill command bar from clicked runbook command
  SetPanelIndex { index: usize },  // Select item in a list
  NextPanelIndex,             // Scroll down in list
  PreviousPanelIndex,         // Scroll up in list
  ```

### 6. Upgrade Button Widget (`src/ui/button.rs`)
- Add fields: `hotkey: Option<KeyCode>`, `on_click: UiAction`, `is_hovered: bool`, `hover_text: Option<Text<'a>>`, `layer: usize`
- Implement `InteractiveWidget` for `Button`:
  - `before_rendering`: register mouse click callback on area, register keyboard callback if hotkey set, detect hover state
  - `hover_text`: return configured hover text
- In Widget::render, add hotkey underline rendering:
  - Find first occurrence of hotkey character in button text
  - Render that character with `UiStyle::HOTKEY_UNDERLINE.underlined()`
- Add hover visual feedback: when `is_hovered`, use `UiStyle::HOVER` background
- Builder methods: `.hotkey(KeyCode)`, `.on_click(UiAction)`, `.hover_text(text)`, `.layer(n)`

### 7. Build ClickableList Widget (`src/ui/clickable_list.rs`)
- Struct `ClickableList<'a>` with items, selected index, hover index, scroll offset, selection_offset
- Implement `InteractiveWidget`:
  - `before_rendering`: register scroll up/down callbacks (global), register click callbacks per visible row (scoped to row Rect)
  - `hover_text`: show description of hovered item
- Implement `Widget`:
  - Render items with selection highlight and hover highlight
  - Show scroll indicators (↑/↓) when content overflows
- Use for runbook commands in step panel — each item is a command, clicking sets it as input

### 8. Rebuild Learning Screen Render Pipeline
- Change `LearningScreen::render` to accept `UiFrame` instead of `Frame`
- Restructure layout to include footer and hover help rows:
  ```rust
  Layout::vertical([
      Constraint::Length(3),     // header
      Constraint::Min(0),        // main content
      Constraint::Length(1),     // footer (key hints)
      Constraint::Length(1),     // hover help text
  ])
  ```
- Replace static step panel commands with `ClickableList`:
  - Each command item: clicking triggers `UiAction::SetCommandInput(cmd.clone())`
  - Hover shows the command blurb as hover text
- Add action buttons row between step panel and terminal:
  ```
  [V Verify] [H Hint] [S Suggest] [→ Next] [← Prev]
  ```
  - Each is an interactive `Button` with hotkey and on_click
- Update `footer_spans()` to return context-sensitive key hints:
  ```rust
  vec![
      ("V", "Verify"), ("H", "Hint"), ("S", "Suggest"),
      ("→", "Next"), ("←", "Prev"), ("?", "Help"),
      ("Esc", "Clear"), ("Enter", "Run"),
  ]
  ```

### 9. Render Footer Bar with Alternating Styles
- In `UiScreen::render`, after screen render, render footer bar:
  - Iterate `footer_spans()` pairs
  - Alternate between `UiStyle::FOOTER_KEY` (inverted bg for key) and `UiStyle::FOOTER_DESC` (muted bg for description)
  - This creates the classic `" V " " Verify " " H " " Hint "` look

### 10. Add Hover Help Text Row
- Reserve bottom row of viewport for hover text
- `UiFrame` tracks `hover_text_area` rect
- When `render_interactive_widget` detects hover, it renders the widget's `hover_text()` into this area
- If nothing is hovered, show default help text ("Type commands or click actions")

### 11. Enable Mouse Capture in Main Event Loop (`src/app/mod.rs`)
- In `setup_terminal()`, add `crossterm::event::EnableMouseCapture` to the execute macro
- In `restore_terminal()`, add `crossterm::event::DisableMouseCapture`
- In `run_loop`, change event reading to handle `Event::Mouse`:
  ```rust
  match event::read()? {
      Event::Key(key) if key.kind == KeyEventKind::Press => {
          // First check callback registry for hotkey matches
          if let Some(action) = registry.resolve_key_event(key.code) {
              // handle action
          } else if let Some(action) = ui.handle_key_events(key, engine) {
              // existing fallback
          }
      }
      Event::Mouse(mouse) => {
          match mouse.kind {
              MouseEventKind::Moved => {
                  // Update registry mouse position for next frame
                  mouse_position = Some((mouse.column, mouse.row));
              }
              MouseEventKind::Down(_) | MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                  if let Some(action) = registry.resolve_mouse_event(mouse.kind, mouse.column, mouse.row) {
                      // handle action
                  }
              }
              _ => {}
          }
      }
      _ => continue,
  }
  ```
- Pass `mouse_position` into UiFrame construction each frame
- After each `terminal.draw()`, extract the callback registry from UiFrame for event processing

### 12. Integrate UiFrame into UiScreen
- Modify `UiScreen::render` to create `UiFrame` wrapping `Frame`:
  ```rust
  pub fn render(&mut self, frame: &mut Frame, engine: &Engine, mouse_pos: Option<(u16, u16)>) -> CallbackRegistry {
      let area = centered_clamped_viewport(frame.area());
      let (body, footer, hover) = split_shell(area);

      let mut ui_frame = UiFrame::new(frame, hover, mouse_pos);

      if self.has_popup() {
          ui_frame.set_active_layer(1);
      }

      // Render current screen
      match self.state {
          ScreenState::Splash => self.splash.render(&mut ui_frame, engine, body),
          ScreenState::Learning => self.learning.render(&mut ui_frame, engine, body),
      }

      // Render footer
      render_footer(&mut ui_frame, footer, self.current_screen_footer_spans());

      // Render popup on layer 1
      if let Some(popup) = self.popup_stack.last() {
          popup.render(&mut ui_frame, area);
      }

      ui_frame.into_registry()
  }
  ```

### 13. Add Popup Layer Awareness
- Modify `PopupMessage::render` to accept `UiFrame` and use `layer 1`
- Popup dismiss button registers on layer 1
- Background widgets on layer 0 won't respond to clicks when popup is active

### 14. Add Action Buttons to Learning Screen
- Between the step panel and terminal feed, render a row of interactive buttons:
  - `Button::new("Verify").hotkey(KeyCode::Char('v')).on_click(UiAction::Verify).hover_text("Run verification checks")`
  - `Button::new("Hint").hotkey(KeyCode::Char('h')).on_click(UiAction::Hint).hover_text("Get a contextual hint")`
  - `Button::new("Suggest").hotkey(KeyCode::Char('s')).on_click(UiAction::Suggest(None)).hover_text("Load next suggested command")`
  - `Button::new("Next").hotkey(KeyCode::Right).on_click(UiAction::NextStep).hover_text("Go to next step")`
  - `Button::new("Prev").hotkey(KeyCode::Left).on_click(UiAction::PrevStep).hover_text("Go to previous step")`
- Only register hotkeys when command_input is empty (avoid conflicts while typing)

### 15. Handle New UiAction Variants in Main Loop
- In `run_loop`, add handlers:
  ```rust
  UiAction::SetCommandInput(cmd) => {
      ui.learning.command_input = cmd;
      ui.learning.status = "Command loaded. Press Enter to run.".to_string();
  }
  UiAction::SetPanelIndex { index } => { /* future use for step browser */ }
  UiAction::NextPanelIndex => { /* scroll list down */ }
  UiAction::PreviousPanelIndex => { /* scroll list up */ }
  ```

### 16. Build Cartoon Cluster Visualization Widget (`src/ui/cluster_view.rs`)

This is the fun part. The activity rail (right 30% on wide terminals) gets replaced with a living ASCII art Kubernetes cluster that changes based on the current step's CKA domain.

**Data Model — `ClusterScene`:**
```rust
pub struct ClusterScene {
    pub control_plane: ControlPlane,
    pub nodes: Vec<Node>,
    pub services: Vec<Service>,
    pub extras: Vec<ExtraResource>,  // Ingress, NetworkPolicy, PV, etc.
    pub tick: usize,                 // For animations
}

pub struct ControlPlane {
    pub api_server: ComponentState,  // Healthy, Degraded, Down
    pub etcd: ComponentState,
    pub scheduler: ComponentState,
    pub controller: ComponentState,
}

pub struct Node {
    pub name: String,
    pub pods: Vec<Pod>,
    pub status: ComponentState,
}

pub struct Pod {
    pub name: String,
    pub status: PodStatus,  // Running, Pending, CrashLoop, Completed
}

pub struct Service {
    pub name: String,
    pub port: u16,
    pub target_node: Option<usize>,  // Arrow points to this node
}

pub enum ExtraResource {
    Ingress { host: String },
    NetworkPolicy { name: String },
    PersistentVolume { name: String, capacity: String },
    Secret { name: String },
    ConfigMap { name: String },
}
```

**Domain-Specific Scenes — each CKA domain renders a different cluster layout:**

| Domain | Scene | Visual Elements |
|--------|-------|-----------------|
| **Cluster Management** | Full cluster overview | Control plane box + 2 nodes + etcd gauge |
| **Workloads** | Pod-heavy view | 2 nodes packed with pods, deployment arrows |
| **Networking** | Service mesh view | Services with arrows to pods, ingress box |
| **Storage** | Volume view | PV/PVC bars, pod→volume connections |
| **Security** | RBAC/policy view | Lock icons, NetworkPolicy borders, secrets |
| **Troubleshooting** | Broken cluster | Nodes with warning symbols, crashloop pods |

**ASCII Art Rendering — Example "Networking" scene (~30 cols wide, ~25 rows tall):**
```
┌─ CONTROL PLANE ─────┐
│ API ◉  ETCD ◉       │
│ SCHED ◉  CTRL ◉     │
└──────────┬──────────┘
           │
    ┌──────┴──────┐
    │             │
┌─NODE-1──┐ ┌─NODE-2──┐
│ ☸ nginx │ │ ☸ api   │
│ ☸ redis │ │ ☸ worker│
│   ◌ ... │ │ ☸ cache │
└────┬────┘ └────┬────┘
     │           │
  ┌──┴───────────┴──┐
  │ SVC nginx → :80 │
  └────────┬────────┘
           │
  ┌────────┴────────┐
  │ ING app.k8s.io  │
  └─────────────────┘
```

**Visual vocabulary — cartoony icons using Unicode:**
- `☸` — Pod (running, green)
- `◌` — Pod (pending, yellow)
- `✖` — Pod (crashloop, red, blinks on tick)
- `◉` — Component healthy (green)
- `◎` — Component degraded (yellow)
- `○` — Component down (red)
- `▰▱` — Capacity/storage bars
- `→` — Service routing arrows
- `🔒` (or `⊠`) — Security/RBAC elements
- `│ ┌ └ ┘ ┐ ├ ┤ ─ ┬ ┴` — Box drawing for structure

**Animation — tick-based heartbeat:**
- Control plane components pulse: `◉` ↔ `●` every 10 ticks (simulates heartbeat)
- CrashLoop pods blink: `✖` ↔ ` ` every 5 ticks
- Pending pods cycle: `◌` → `◔` → `◑` → `◕` (spinner)
- On step completion: brief "flash" where all pods turn green for 20 ticks

**Rendering Safety — Preventing Layout Overflow:**

The cluster view must never stretch the app or cause rendering issues regardless of terminal size or scene complexity. Safeguards:

1. **Fixed viewport, not content-driven sizing** — The activity rail always gets `Constraint::Percentage(30)`. The cluster widget renders *within* that fixed Rect. It never pushes other panels.

2. **Canonical scene with clamping** — Each scene is designed for a canonical 28×20 area. Components are rendered top-to-bottom with a running line counter. Once `lines_used >= area.height`, remaining components are skipped.

3. **Tiered degradation by available height:**
   | Available Height | What Renders |
   |---|---|
   | >= 20 rows | Full scene: control plane + 2 nodes + services + extras |
   | 12-19 rows | Compact: control plane bar + 1 node + service |
   | < 12 rows | Micro: single status line `☸ 3/5 pods │ ◉ healthy` |
   | < 6 rows | Nothing (area too small, skip entirely) |

4. **Capped element counts** — Max 2 nodes, max 4 pods per node, max 2 services, max 1 ingress. These are hard caps regardless of step data.

5. **Width clamping** — All box-drawing is computed relative to `area.width`. Node boxes use `area.width - 4` for their border width. Pod names are truncated via `ellipsize(name, node_inner_width - 4)`.

6. **Pre-render line budget:**
   ```rust
   fn render(&self, area: Rect) {
       let budget = area.height as usize;
       let mut used = 0;

       // Control plane: 3 lines (or 1 if compact)
       let cp_height = if budget >= 20 { 3 } else { 1 };
       render_control_plane(cp_height); used += cp_height;

       // Connector: 1 line
       if used + 1 < budget { render_connector(); used += 1; }

       // Nodes: variable, but capped
       for node in self.nodes.iter().take(max_nodes) {
           let node_h = 2 + node.pods.len().min(4);
           if used + node_h >= budget { break; }
           render_node(node); used += node_h;
       }

       // Services: only if space remains
       for svc in self.services.iter().take(2) {
           if used + 2 >= budget { break; }
           render_service(svc); used += 2;
       }
   }
   ```

**Implementation:**
- `ClusterScene::for_domain(domain: &str, step: &Step, progress: &ProgressState, tick: usize) -> Self`
  - Factory that builds the right scene based on domain
  - Uses step metadata (commands, difficulty) to populate pod names
  - Uses progress to decide how many pods are "running" vs "pending"
- `ClusterScene::render(&self, frame: &mut UiFrame, area: Rect)`
  - Render using box-drawing characters and semantic styles
  - Uses line budget system (described above) to prevent overflow
  - Each component is an interactive widget (hoverable for description)
- Scene updates every `update()` call (tick increments)

### 17. Integrate Cluster View into Learning Screen Activity Rail
- In `LearningScreen`, replace `render_activity_rail()` with `render_cluster_rail()`
- The rail now has two sections:
  ```rust
  Layout::vertical([
      Constraint::Min(0),        // Cluster visualization (takes most space)
      Constraint::Length(6),     // Status summary (compact: status + hint + completion)
  ])
  ```
- The cluster visualization fills the top portion
- Below it, a compact status summary shows:
  - Current status badge
  - Hint (if present, truncated to 2 lines)
  - Completion card (if present, 1 line)
- `LearningScreen` stores a `ClusterScene` that updates on `update()`:
  ```rust
  pub struct LearningScreen {
      // ... existing fields ...
      pub cluster_scene: ClusterScene,
  }
  ```
- On step change (next/prev/jump), rebuild the scene: `self.cluster_scene = ClusterScene::for_domain(...)`
- On completion, trigger the flash animation

### 18. Make Cluster Components Hoverable
- Each cluster component (node, pod, service, etc.) renders as an interactive element
- Hovering a pod shows: "Pod nginx-7b4f8c | Running | Node 1"
- Hovering a service shows: "Service nginx-svc | Port 80 → Pods in Node 1, Node 2"
- Hovering the control plane shows: "Control Plane | API Server, etcd, Scheduler, Controller Manager"
- Hovering a PV shows: "PersistentVolume data-vol | 10Gi | Bound"
- This teaches K8s concepts passively while the user works

### 19. Validate Full Interaction Flow
- Test keyboard-only flow: all existing slash commands still work
- Test mouse flow: hover highlights, click commands auto-fill, click buttons trigger actions
- Test popup layer: clicking through popup should NOT trigger background actions
- Test small terminal: graceful degradation when terminal < 120 cols (hide action buttons, footer wraps)
- Run `cargo build` and `cargo clippy` to ensure clean compilation

## Testing Strategy
- **Keyboard regression**: All existing slash commands (`/verify`, `/hint`, `/next`, `/prev`, `/suggest`, `/help`, `/clear`, `/quit`) continue to work identically
- **Mouse interactions**: Hover runbook command → highlight + hover text; click → auto-fill command bar; click action button → triggers action
- **Layer isolation**: Open help popup → click background → nothing happens; dismiss popup → clicks work again
- **Responsive layout**: Terminal < 120 cols → action buttons row hidden, footer shows compact hints; >= 120 cols → full layout
- **Edge cases**: Empty runbook (no commands) → no clickable list rendered; rapid clicking → no double-fire; scroll past bounds → clamped
- **Cluster rendering safety** (critical for preventing layout issues):
  - **Unit test: line budget never exceeds area height** — For every domain scene, assert `rendered_lines <= area.height` across a range of Rect sizes (6×10, 28×20, 40×30, 100×48)
  - **Unit test: width never exceeds area width** — For every rendered Line, assert `line.width() <= area.width`
  - **Fuzz test terminal sizes** — Render each domain scene at sizes from 5×5 to 160×48 in increments of 5, assert no panic and no overflow
  - **Snapshot tests** — Capture expected ASCII output for each domain scene at canonical size (28×20) to catch regressions
  - **Stress test: rapid step changes** — Navigate next/prev rapidly through all steps, assert cluster scene rebuilds cleanly without accumulating state

## Acceptance Criteria
- [ ] Mouse hover on runbook commands shows highlight and hover text
- [ ] Clicking a runbook command fills the command bar with that command
- [ ] Action buttons (Verify, Hint, Suggest, Next, Prev) render with visible hotkey underlines
- [ ] Clicking an action button triggers the corresponding UiAction
- [ ] Pressing the hotkey character (when not typing) triggers the same action
- [ ] Footer bar shows alternating key/description segments for all available actions
- [ ] Hover help text row at bottom shows contextual help for hovered element
- [ ] Popups block all background mouse interactions
- [ ] All existing keyboard slash commands still work
- [ ] **Cartoon cluster renders in activity rail on wide terminals (120+ cols)**
- [ ] **Cluster scene changes based on current step's CKA domain**
- [ ] **Control plane heartbeat animation pulses every ~1 second**
- [ ] **CrashLoop pods blink in troubleshooting scenes**
- [ ] **Hovering cluster components shows K8s educational hover text**
- [ ] **Step completion triggers brief green flash on cluster**
- [ ] Narrow terminals (<120 cols) gracefully hide cluster view
- [ ] `cargo build` and `cargo clippy` pass cleanly

## Validation Commands
- `cargo build 2>&1` — Ensure clean compilation
- `cargo clippy -- -W clippy::all 2>&1` — No warnings
- `cargo test 2>&1` — All existing tests pass
- `cargo run` — Manual smoke test: hover commands, click buttons, use slash commands, open/close popup

## Notes
- The `crossterm` crate (already at 0.28) supports mouse capture natively — no new dependencies needed
- The UiFrame pattern requires passing the callback registry back from render to the event loop. This changes the render signature to return `CallbackRegistry` instead of `Result<()>`. An alternative is storing it on `UiScreen`, but returning it is cleaner (no stale state).
- Hotkeys should only be active when the command bar is empty to avoid conflicts with typing. The `before_rendering` on buttons should check this condition.
- The `ClickableList` for runbook commands is simpler than Rebels' version — no selection persistence needed, just click-to-fill behavior.
- Phase 1-2 can be implemented without enabling mouse capture (keyboard hotkeys alone are valuable). Phase 3 enables mouse support additively.
- **Cluster visualization is pure ASCII art** — no image dependencies needed. Uses box-drawing characters (`┌└┐┘─│`), Unicode symbols (`☸◉◌✖▰▱→`), and semantic colors from UiStyle.
- The cluster scene is **deterministic per step** — same step always shows the same base layout. Animation (heartbeat, blink) is tick-driven from the existing `update()` cycle at ~10fps (100ms poll interval).
- Pod/node names in the cluster are **decorative** — they're derived from the step's commands (e.g., if the step mentions `nginx`, the cluster shows nginx pods). This creates a visual connection between what you're typing and what you see.
- The cluster view **does not query actual cluster state** — it's a pedagogical illustration, not a live dashboard. This keeps it simple and avoids kubectl dependencies in the UI layer.
