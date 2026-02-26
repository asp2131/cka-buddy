# Plan: CKA-Buddy TUI Redesign — Rebels-in-the-Sky Quality

## Task Description
Redesign the CKA-Buddy TUI to match the creative, intuitive, and engaging quality of the rebels-in-the-sky terminal game. This involves a full architectural refactor (Screen trait system, modular panels, centralized theming), a rich K8s-themed visual overhaul (splash screen, ASCII art, big numbers, popups, hover text), and polished widget system — all while preserving the existing slash-command UX and PTY-based shell interaction. The goal is to make CKA-Buddy feel as polished and delightful as Claude Code's terminal experience.

## Objective
Transform the monolithic `layout.rs` (488 lines, single `draw()` function) into a modular, visually rich TUI with:
- Screen trait architecture (splash, learning screens)
- K8s-branded splash screen with ASCII art and rotating DevOps quotes
- Rich color palette with semantic K8s theming
- Popup system for celebrations, help, tutorials
- Big-number readiness display
- Contextual help footer
- All existing functionality preserved (slash commands, PTY, verification, progress)

## Problem Statement
The current TUI is functional but clinical — flat text, ~8 basic colors, no visual hierarchy, no splash screen, no popups, no animations. All rendering lives in a single 488-line `layout.rs`. The rebels-in-the-sky project (same Ratatui 0.29 + Crossterm stack) demonstrates what's possible: rich panels, ASCII art, animated elements, popup modals, hover text, and a professional widget system. CKA-Buddy needs to match that quality to be engaging as an educational tool.

## Solution Approach
Adopt rebels-in-the-sky's architectural patterns (Screen trait, UiFrame wrapper, centralized constants) adapted for cka-buddy's educational context. Replace the monolithic renderer with modular screens. Add visual flair appropriate for a K8s learning tool (not a game): ASCII splash, domain-colored progress, celebration popups, contextual help. Keep slash-command-only UX (no mouse/hotkeys).

## Relevant Files

### Existing Files to Modify
- **`src/ui/layout.rs`** (488 lines) — Current monolithic renderer. Will be decomposed into learning_screen.rs and widgets.rs, then deleted
- **`src/ui/mod.rs`** (1 line) — Currently only exports layout. Must expand to export all new UI modules
- **`src/app/mod.rs`** (327 lines) — Event loop and slash command routing. Must be refactored to use UiScreen instead of direct draw() calls. UI state variables (command_input, status, output_log, hint_message, tab_index, completion_card) move into LearningScreen
- **`src/app/state.rs`** (7 lines) — CompletionCard struct stays here, used by popup system
- **`src/lib.rs`** (7 lines) — No changes needed, already exports ui module

### Reference Files (rebels-in-the-sky patterns to adapt)
- **`rebels-in-the-sky/src/ui/constants.rs`** (43 lines) — Pattern for UiStyle struct with const Style definitions
- **`rebels-in-the-sky/src/ui/traits.rs`** (164 lines) — Screen trait, InteractiveWidget trait definitions
- **`rebels-in-the-sky/src/ui/big_numbers.rs`** (153 lines) — ASCII digit font, BigNumberFont trait
- **`rebels-in-the-sky/src/ui/splash_screen.rs`** (374 lines) — Splash screen with ASCII title, quotes, menu
- **`rebels-in-the-sky/src/ui/popup_message.rs`** (1,312 lines) — Centered modal overlay system
- **`rebels-in-the-sky/src/ui/button.rs`** (264 lines) — Button widget with selected/disabled states
- **`rebels-in-the-sky/src/ui/hover_text_line.rs`** (236 lines) — Bottom-line contextual help

### New Files to Create
- `src/ui/constants.rs` — K8s color palette and style constants
- `src/ui/traits.rs` — Screen trait definition
- `src/ui/ui_action.rs` — UiAction enum (replaces rebels' UiCallback for our simpler case)
- `src/ui/ui_screen.rs` — Top-level screen state machine with popup overlay
- `src/ui/splash_screen.rs` — K8s-themed splash with ASCII art and quotes
- `src/ui/learning_screen.rs` — Main learning interface (replaces layout.rs)
- `src/ui/popup.rs` — Popup message types and centered modal rendering
- `src/ui/big_numbers.rs` — ASCII digit font (adapted from rebels)
- `src/ui/widgets.rs` — Shared widget utilities (progress bars, badges, dividers)
- `src/ui/button.rs` — Simple button widget for splash menu

## Implementation Phases

### Phase 1: Architectural Foundation
Create the Screen trait system, centralized constants, UiAction enum, and UiScreen state machine. This is the skeleton everything builds on. The old `layout.rs::draw()` continues working during migration via a compatibility shim in UiScreen.

### Phase 2: Splash Screen + Buttons
Build the K8s-themed splash screen with ASCII art title, rotating DevOps quotes, and Start/Continue/Quit menu buttons. First real Screen trait implementor — validates the architecture.

### Phase 3: Learning Screen Visual Overhaul
Migrate all rendering from `layout.rs` into the new `LearningScreen` struct. Apply the K8s color palette, add bordered panels, colored badges, richer progress bar, and visual hierarchy. Delete `layout.rs` when complete.

### Phase 4: Widget System (Popups, Big Numbers, Progress)
Build the popup modal system for step completion celebrations (with big-number readiness delta), help overlay, verify fail feedback, and tutorial pages. Add the big-number ASCII font.

### Phase 5: Contextual Help Footer + Tutorial
Add state-driven contextual help in the footer bar. Build the first-launch tutorial popup (3-page walkthrough). Polish and integrate everything.

## Team Orchestration

- You operate as the team lead and orchestrate the team to execute the plan.
- You're responsible for deploying the right team members with the right context to execute the plan.
- IMPORTANT: You NEVER operate directly on the codebase. You use `Task` and `Task*` tools to deploy team members to do the building, validating, testing, deploying, and other tasks.
- Communication is paramount. You'll use the Task* Tools to communicate with the team members and ensure they're on track to complete the plan.

### Team Members

- Builder
  - Name: foundation-builder
  - Role: Create the architectural foundation — Screen trait, constants, UiAction, UiScreen, ui/mod.rs expansion
  - Agent Type: general-purpose
  - Resume: true

- Builder
  - Name: splash-builder
  - Role: Build the splash screen with ASCII art, quotes, button widget, and menu navigation
  - Agent Type: general-purpose
  - Resume: true

- Builder
  - Name: learning-screen-builder
  - Role: Migrate layout.rs into LearningScreen with full K8s visual overhaul and refactor app/mod.rs event loop
  - Agent Type: general-purpose
  - Resume: true

- Builder
  - Name: widget-builder
  - Role: Build popup system, big numbers, celebration cards, help overlay, tutorial, and contextual help footer
  - Agent Type: general-purpose
  - Resume: true

- Validator
  - Name: final-validator
  - Role: Validate the complete TUI compiles, runs, and all screens/popups work correctly
  - Agent Type: validator
  - Resume: false

## Step by Step Tasks

### 1. Create Architectural Foundation
- **Task ID**: create-foundation
- **Depends On**: none
- **Assigned To**: foundation-builder
- **Agent Type**: general-purpose
- **Parallel**: false
- Create `src/ui/constants.rs` with K8s color palette:
  ```rust
  pub struct UiStyle;
  impl UiStyle {
      pub const DEFAULT: Style = Style { fg: None, bg: None, underline_color: None, add_modifier: Modifier::empty(), sub_modifier: Modifier::empty() };
      pub const SELECTED: Style = Self::DEFAULT.bg(Color::Rgb(40, 44, 62));
      pub const HEADER: Style = Self::DEFAULT.fg(Color::Rgb(50, 130, 240));   // Cluster blue
      pub const HIGHLIGHT: Style = Self::DEFAULT.fg(Color::Rgb(0, 200, 210)); // Namespace cyan
      pub const OK: Style = Self::DEFAULT.fg(Color::Rgb(80, 220, 120));       // Pod green
      pub const WARNING: Style = Self::DEFAULT.fg(Color::Rgb(255, 185, 50));  // Warning amber
      pub const ERROR: Style = Self::DEFAULT.fg(Color::Rgb(240, 70, 70));     // Error red
      pub const MUTED: Style = Self::DEFAULT.fg(Color::Rgb(80, 88, 105));     // Text muted
      pub const TEXT_PRIMARY: Style = Self::DEFAULT.fg(Color::Rgb(230, 235, 245));
      pub const TEXT_SECONDARY: Style = Self::DEFAULT.fg(Color::Rgb(140, 150, 170));
      pub const COMMAND: Style = Self::DEFAULT.fg(Color::Rgb(255, 220, 100)); // Command yellow
      pub const PROMPT: Style = Self::DEFAULT.fg(Color::Rgb(0, 220, 230));    // Prompt cyan
      pub const BORDER: Style = Self::DEFAULT.fg(Color::Rgb(55, 62, 80));     // Panel borders
      // Domain-specific colors for CKA exam domains
      pub const DOMAIN_STORAGE: Style = Self::DEFAULT.fg(Color::Rgb(180, 120, 255));
      pub const DOMAIN_NETWORKING: Style = Self::DEFAULT.fg(Color::Rgb(100, 200, 255));
      pub const DOMAIN_WORKLOADS: Style = Self::DEFAULT.fg(Color::Rgb(255, 150, 100));
      pub const DOMAIN_CLUSTER: Style = Self::DEFAULT.fg(Color::Rgb(120, 230, 180));
      pub const DOMAIN_SECURITY: Style = Self::DEFAULT.fg(Color::Rgb(255, 100, 130));
      pub const DOMAIN_TROUBLESHOOTING: Style = Self::DEFAULT.fg(Color::Rgb(255, 210, 80));
      // Difficulty
      pub const DIFF_EASY: Style = Self::DEFAULT.fg(Color::Rgb(80, 220, 120));
      pub const DIFF_MEDIUM: Style = Self::DEFAULT.fg(Color::Rgb(255, 185, 50));
      pub const DIFF_HARD: Style = Self::DEFAULT.fg(Color::Rgb(240, 70, 70));
  }
  ```
- Create `src/ui/traits.rs` with Screen trait (adapted from rebels — no `debug_view`, uses `Engine` instead of `World`):
  ```rust
  pub trait Screen {
      fn update(&mut self, engine: &Engine) -> anyhow::Result<()>;
      fn render(&mut self, frame: &mut Frame, engine: &Engine, area: Rect) -> anyhow::Result<()>;
      fn handle_key_events(&mut self, key_event: KeyEvent, engine: &Engine) -> Option<UiAction>;
      fn footer_help(&self) -> String { String::new() }
  }
  ```
- Create `src/ui/ui_action.rs` with action enum:
  ```rust
  pub enum UiAction {
      None, Quit, StartSession, NextStep, PrevStep, JumpRecommended, JumpBack,
      Verify, Hint, Suggest(Option<usize>), ClearLog, ShowHelp, DismissPopup,
      RunCommand(String), ForceRunCommand(String),
  }
  ```
- Create `src/ui/ui_screen.rs` — top-level screen manager with `ScreenState::Splash | Learning`, popup stack, delegates update/render/handle_key_events to active screen, renders popup overlay on top
- Update `src/ui/mod.rs` to export all new modules
- Ensure existing `layout.rs::draw()` still works during migration (UiScreen initially wraps it)

### 2. Build Splash Screen and Button Widget
- **Task ID**: build-splash
- **Depends On**: create-foundation
- **Assigned To**: splash-builder
- **Agent Type**: general-purpose
- **Parallel**: false
- Create `src/ui/button.rs` — simplified button widget (no mouse callbacks):
  - Fields: text, selected, disabled, style, selected_style, optional block
  - Renders with thick border when selected (`border::THICK`), dim when disabled
  - Used for splash menu items and popup confirmation buttons
- Create `src/ui/splash_screen.rs` implementing Screen trait:
  - Large ASCII art title "CKA BUDDY" in box-drawing characters (adapt rebels' splash_screen.rs pattern), rendered in cluster blue
  - Subtitle: "Kubernetes Exam Preparation" in muted text
  - 15+ rotating DevOps/K8s quotes (rotate every ~8 seconds via tick counter):
    - "Pods are cattle, not pets."
    - "If it's not in version control, it doesn't exist."
    - "There is no cloud, it's just someone else's computer."
    - "kubectl get pods --all-namespaces - the most typed command in history"
    - "etcd: because your cluster state shouldn't be a mystery."
    - "In containers we trust, in orchestrators we believe."
    - "The best incident response is prevention."
    - "Infrastructure as code, or it didn't happen."
    - "Namespaces: because sharing is overrated."
    - "A rolling update gathers no downtime."
    - "Your cluster is only as strong as its weakest RBAC policy."
    - "Probes don't lie — your app's health is what it is."
    - "The control plane is always watching."
    - "Scheduling is an art. Resource requests are science."
    - "Every outage is a learning opportunity."
  - Menu: Up/Down arrow to select, Enter to confirm
    - "Continue" (shown when saved progress exists, selected by default)
    - "New Session" (start fresh)
    - "Quit"
  - Show progress summary if saved: "42% ready | 12/25 steps complete"
  - Layout: vertical centering within terminal — margin, title, subtitle, spacer, menu, quote
  - ASCII Kubernetes wheel animation (simple 4-frame text spinner using tick counter)

### 3. Migrate Layout to LearningScreen with K8s Visual Overhaul
- **Task ID**: build-learning-screen
- **Depends On**: create-foundation
- **Assigned To**: learning-screen-builder
- **Agent Type**: general-purpose
- **Parallel**: true (can run alongside build-splash)
- Create `src/ui/learning_screen.rs` implementing Screen trait
- Move UI state from app/mod.rs into LearningScreen struct:
  ```rust
  pub struct LearningScreen {
      pub command_input: String,
      pub status: String,
      pub output_log: Vec<String>,
      pub hint_message: Option<String>,
      pub completion_card: Option<CompletionCard>,
      pub tab_index: usize,
  }
  ```
- Migrate rendering from layout.rs, applying K8s visual overhaul:
  - **Header**: Replace flat text with bordered header block. Use `UiStyle::HEADER` for "CKA BUDDY". Colored badges for step type: `[EXAM]` in amber, `[PROJECT]` in blue, `[BUG]` in red. Difficulty colored: easy=green, medium=amber, hard=red. Progress bar uses block characters `████░░░░` with gradient coloring.
  - **Step Panel**: Wrap in `Block::bordered()` with title "Step". Objective in bold primary text. Domain tags each colored by domain type (map domain string to UiStyle::DOMAIN_*). Runbook commands with syntax-like coloring (command in yellow, indices in muted).
  - **Terminal Feed**: Wrap in `Block::bordered()` with title "Terminal". Output in secondary text color. Preserve auto-scroll behavior.
  - **Activity Rail** (wide terminals): Bordered panel with title "Activity". Status, hints, and next actions with proper section headers.
  - **Command Bar**: Keep `>` prompt in cyan bold. Mode badges with background color (`UiStyle::SELECTED` bg). Contextual help line above the prompt.
- Refactor `src/app/mod.rs` event loop:
  - Replace `draw()` call with `ui_screen.update()` + `ui_screen.render()`
  - Route key events through `ui_screen.handle_key_events()` first
  - Process returned UiAction for slash commands, PTY execution, navigation
  - Move slash command handling to process UiAction results
  - Handle `UiAction::StartSession` transition from splash to learning
- Delete `src/ui/layout.rs` after full migration
- Create `src/ui/widgets.rs` with shared utilities extracted from layout.rs:
  - `default_block() -> Block` (bordered with BORDER color)
  - `titled_block(title: &str) -> Block`
  - `progress_bar(percentage: u8, width: usize) -> Line`
  - `step_type_badge(step_type: &StepType) -> Span`
  - `difficulty_badge(difficulty: &str) -> Span`
  - `domain_tag(domain: &str) -> Span` (maps domain name to color)
  - `mode_badge(status: &str, command_input: &str) -> (String, Style)`
  - `readiness_segments(readiness: u8, slots: usize) -> (String, String)`
  - `divider(width: usize) -> String`
  - `ellipsize(text: &str, max: usize) -> String`

### 4. Build Popup System and Big Numbers
- **Task ID**: build-widgets
- **Depends On**: build-learning-screen
- **Assigned To**: widget-builder
- **Agent Type**: general-purpose
- **Parallel**: false
- Create `src/ui/big_numbers.rs` — adapt directly from rebels-in-the-sky/src/ui/big_numbers.rs:
  - Copy all digit definitions (zero through nine, dots, hyphen)
  - Add `percent()` symbol for `%` display
  - Add `render_big_number(frame, area, value: u8)` helper that splits a number into digits and renders horizontally
  - Adapt `big_text()` helper (from rebels' utils.rs) or inline it
- Create `src/ui/popup.rs` with popup types:
  ```rust
  pub enum PopupMessage {
      StepComplete { title: String, what_changed: Vec<String>, next_commands: Vec<String>,
                     readiness_before: u8, readiness_after: u8 },
      Help { commands: Vec<(&'static str, &'static str)> },
      VerifyFail { message: String },
      Tutorial { page: usize, total_pages: usize, content: Vec<(String, Vec<String>)> },
  }
  ```
  - Render as centered overlay: `Clear` background widget, then bordered box in `BORDER` color with thick borders
  - StepComplete popup: Title "STEP COMPLETE" in pod green bold, big-number readiness display (before -> after), what_changed list, next_commands, "Press Enter to continue"
  - Help popup: Title "COMMAND REFERENCE", two-column layout of all slash commands with descriptions
  - VerifyFail popup: Title "NOT YET" in warning amber, failure message, "Press Enter to close"
  - Tutorial popup: Multi-page with page indicator `[1/3]`, Left/Right to navigate, Enter on last page to dismiss
  - All popups dismiss on Enter key, consumed by UiScreen before reaching active screen
- Wire popups into UiScreen:
  - `ui_screen.push_popup(msg)` — adds to stack
  - Render topmost popup as overlay after rendering active screen
  - Key events: Enter/Esc dismiss, arrow keys navigate tutorial pages
- Wire StepComplete popup into verify flow in app/mod.rs:
  - On `UiAction::Verify` -> `VerifyOutcome::Pass`: capture readiness_before, complete_current, capture readiness_after, push StepComplete popup
  - On `VerifyOutcome::Fail`: push VerifyFail popup
- Wire Help popup: On `/help` slash command, push Help popup instead of writing to status

### 5. Add Contextual Help Footer and Tutorial Flow
- **Task ID**: build-help-tutorial
- **Depends On**: build-widgets
- **Assigned To**: widget-builder
- **Agent Type**: general-purpose
- **Parallel**: false
- Add contextual help to LearningScreen footer:
  - 1-line footer between command bar and prompt showing state-driven help text
  - When popup active: "Enter: dismiss | Esc: close"
  - When input starts with `/hint`: "/hint — Ask the coach for a contextual hint"
  - When input starts with `/verify`: "/verify — Run verification checks for current step"
  - When input starts with `/suggest`: "/suggest [n] — Load suggested command"
  - When input starts with `/`: "Type a slash command. /help for full list."
  - When input empty: "Type kubectl commands or /help for available commands"
  - When typing: "Press Enter to execute: {command_input}"
- Build tutorial content (3 pages):
  - Page 1 — "Welcome to CKA Buddy": Brief intro, what the tool does, how it helps with CKA prep
  - Page 2 — "Commands": All slash commands with descriptions, note about direct kubectl execution
  - Page 3 — "Interface Guide": Header (readiness, step info), Step Panel (objective, runbook), Terminal (command output), Command Bar (prompt)
- Add first-launch detection:
  - Check for `tutorial_seen: bool` in ProgressState (add field with `#[serde(default)]`)
  - On first launch (tutorial_seen == false), push Tutorial popup after transitioning to Learning screen
  - Set `tutorial_seen = true` after dismissing tutorial, save progress
- Final polish pass:
  - Ensure all borders use consistent `UiStyle::BORDER` color
  - Ensure all text uses the correct palette colors
  - Test narrow terminal (<120 chars) and wide terminal (>=120 chars) layouts
  - Verify splash -> learning transition works with both Continue and New Session

### 6. Final Validation
- **Task ID**: validate-all
- **Depends On**: create-foundation, build-splash, build-learning-screen, build-widgets, build-help-tutorial
- **Assigned To**: final-validator
- **Agent Type**: validator
- **Parallel**: false
- Run `cargo build` — must compile with no errors
- Run `cargo clippy` — must pass with no warnings
- Verify file structure: all new files exist in `src/ui/`
- Verify `src/ui/layout.rs` has been deleted
- Verify `src/ui/mod.rs` exports all new modules
- Read through each new file to verify:
  - Screen trait is properly implemented by SplashScreen and LearningScreen
  - UiScreen correctly delegates to active screen and renders popups
  - app/mod.rs event loop uses UiScreen instead of direct draw()
  - All slash commands still work via UiAction routing
  - K8s color palette is consistently applied
  - Big numbers render correctly
  - Popups render as centered overlays
  - Contextual help footer changes based on state

## Acceptance Criteria
- [ ] `cargo build` succeeds with no errors
- [ ] `cargo clippy` passes with no warnings
- [ ] Splash screen displays on launch with ASCII title, quote, and menu
- [ ] "Continue" menu option appears when saved progress exists
- [ ] Pressing Enter on menu transitions to learning screen
- [ ] Learning screen shows bordered panels with K8s color palette
- [ ] Step type, difficulty, and domain tags are color-coded
- [ ] Progress bar uses block characters with gradient
- [ ] All slash commands work: /next, /prev, /back, /recommended, /verify, /hint, /suggest, /clear, /help, /quit
- [ ] PTY shell execution works (typing kubectl commands executes in embedded zsh)
- [ ] /verify success shows StepComplete popup with big-number readiness
- [ ] /verify fail shows VerifyFail popup
- [ ] /help shows Help popup overlay (not inline text)
- [ ] Tutorial popup shows on first launch
- [ ] Contextual help footer updates based on input state
- [ ] Wide terminal (>=120 chars) shows activity rail
- [ ] Narrow terminal (<120 chars) shows single-column layout
- [ ] layout.rs is deleted and no longer referenced

## Validation Commands
- `cargo build 2>&1` — Must compile cleanly
- `cargo clippy 2>&1` — Must pass with no warnings
- `cargo test 2>&1` — Must pass (if tests exist)
- `ls src/ui/` — Verify all new files: constants.rs, traits.rs, ui_action.rs, ui_screen.rs, splash_screen.rs, learning_screen.rs, popup.rs, big_numbers.rs, widgets.rs, button.rs, mod.rs
- `grep -r "layout::draw" src/` — Must return empty (no references to old draw function)
- `grep -r "UiStyle::" src/ui/` — Verify K8s palette is used throughout

## Notes
- No new dependencies needed — Ratatui 0.29 and Crossterm 0.28 already provide everything
- The `rand` crate is NOT in Cargo.toml — quote rotation should use a simple tick-based index, not random selection
- The `tutorial_seen` field added to ProgressState needs `#[serde(default)]` for backward compatibility with existing progress files
- CompletionCard struct stays in `src/app/state.rs` — it's a data model, not a UI widget
- The popup rendering must use ratatui's `Clear` widget to blank the area behind the modal before rendering the bordered box
- Reference rebels' `popup_message.rs` for the exact centered-overlay rendering technique
