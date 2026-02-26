# CKA Buddy UI Redesign Plan
> **Note:** Save a copy of this plan to `docs/ui-redesign-plan.md` in the project repo.

## Context

The current 5-panel layout (Progress, Objective, Run/Next+Terminal side-by-side, Success Card, Command Footer) creates visual overload. Everything competes for attention at once. The goal is a 3-zone layout that feels like an AI coding agent — a focused content feed with a shell-style command bar at the bottom, à la FreecodeChamp × Khan Academy × LLM TUI.

**User choices confirmed:**
- Suggested commands: numbered list in the main feed ([1] cmd, [2] cmd...) with Tab cycling through them
- Navigation: j/k and arrows stay as step navigation; terminal auto-scrolls to bottom

## Workflow Revision (Slash-First UX)

Updated UX direction: remove single-key action bindings for app workflow, and use explicit slash commands (OpenCode/Claude-style) entered in the command bar.

- No direct `j/k`, arrows, `h`, `v`, `b`, `r`, `Tab`, or `q` for workflow control.
- Workflow controls become slash commands:
  - `/help`, `/next`, `/prev`, `/back`, `/recommended`
  - `/verify`, `/hint`, `/suggest [n]`, `/clear`, `/quit`
- Non-slash input remains shell command execution (with existing guard behavior).
- This aligns interaction with modern command-driven TUIs and avoids hidden keymap cognitive load.

## Reevaluation (Ratatui-Rebels Lens)

This redesign direction is strong and should proceed, but with a few adjustments to match proven ratatui interaction patterns:

1. **Keep the 3-zone shell** (header, feed, command bar) exactly as proposed.
2. **Keep inline hints and inline success** in the feed (good reduction of panel churn).
3. **Adjust feed composition slightly:** keep objective + suggested commands pinned near the top and auto-scroll only the activity section (terminal/hint/success), so key context does not disappear on long sessions.
4. **Preserve actionable success metadata:** include `next_commands` and `verify_optional` inline in dim style after the success line (do not drop this from the existing `CompletionCard` behavior).
5. **Use semantic style constants in `layout.rs`** (local const palette) instead of scattered inline color literals for maintainability and consistency.

These changes keep your visual simplification while preserving discoverability and long-session usability.

---

## Files Changed

| File | Change |
|------|--------|
| `src/ui/layout.rs` | Complete rewrite of `draw()` — 5-panel → 3-zone |
| `src/app/mod.rs` | Add `hint_message` + `tab_index` state; update draw() call; fix Tab cycling |
| `src/app/state.rs` | Remove unused `VerifyMode` enum |

**Files NOT changed:** engine.rs, content/, progress/, coach/, verify/, terminal/

---

## New Layout

```
┌──────────────────────────────────────────┐  <- 2 lines, no border (styled bg)
│ CKA Buddy  ████░░░ 45%  │ Step 5/20 │ Exam │ Hard │
│ Deploy nginx Pod in default namespace    │
├──────────────────────────────────────────┤  <- Min(0), one scrollable area
│                                          │
│  Deploy an nginx Pod in default ns...   │  <- objective (bold)
│  10 min · Hard · Workloads, RBAC        │  <- metadata (dim)
│                                          │
│  Steps to run:                           │
│  [1] kubectl create namespace test      │  <- cyan/yellow
│  [2] kubectl run nginx --image=nginx    │
│                                          │
│  ─── Terminal ────────────────────────  │  <- dim separator
│  $ kubectl create namespace test        │  <- shell output (gray)
│  namespace/test created                 │
│  $ kubectl run nginx --image=nginx      │
│  pod/nginx created                      │
│                                          │
│  ✓ Deploy nginx complete!               │  <- green (inline success)
│    Pod running in default namespace     │
│                                          │
│  [h] hint  [v] verify  [j/k] nav  [TAB] suggest  │
├──────────────────────────────────────────┤  <- 3 lines
│  step 6/20 unlocked                     │  <- status (dim), or empty
│  ❯ _                                    │  <- command input
└──────────────────────────────────────────┘
```

---

## Implementation Steps

### Step 1 — `src/app/state.rs`

Remove `VerifyMode` enum (never rendered, only passed around). Keep `CompletionCard` unchanged.

### Step 2 — `src/app/mod.rs`

**Add two new local state variables in `run_loop`:**
```rust
let mut hint_message: Option<String> = None;
let mut tab_index: usize = 0;
```

**Update key handlers:**
- `h` key: `hint_message = Some(coach.hint(current_step, &output_log))`
- `j`/`k`/`↑`/`↓`/`b`/`r` navigation: clear `hint_message = None`, `completion_card = None`, `tab_index = 0`
- `Tab` key: cycle through `run_commands` instead of always index 0:
  ```rust
  let cmds = &engine.current_step().run_commands;
  if !cmds.is_empty() {
      command_input = cmds[tab_index % cmds.len()].clone();
      tab_index += 1;
  }
  ```
- Remove `verify_mode` variable (was unused in rendering)
- On successful `v` verify: keep existing `CompletionCard` fields populated (`done`, `what_changed`, `next_commands`, `verify_optional`) so they can be rendered inline in Zone 1.
- On manual typing (`KeyCode::Char(_)`): reset `tab_index = 0` so suggestion cycling remains predictable.

**Update `draw()` call:**
```rust
draw(frame, &engine, &command_input, &status, &output_log,
     hint_message.as_deref(), completion_card.as_ref());
```

### Step 3 — `src/ui/layout.rs`

**New `draw()` signature:**
```rust
pub fn draw(
    frame: &mut Frame,
    engine: &Engine,
    command_input: &str,
    status: &str,
    output_log: &[String],
    hint_message: Option<&str>,
    completion_card: Option<&CompletionCard>,
)
```

**3-zone vertical layout:**
```rust
let zones = Layout::vertical([
    Constraint::Length(2),   // zones[0]: header
    Constraint::Min(0),      // zones[1]: main content
    Constraint::Length(3),   // zones[2]: command bar
]).split(frame.area());
```

**Zone 0 — Header (no border, styled background):**
- Line 1: `" CKA Buddy  [████░░░] 45%  │  Step 5 / 20  │  Exam  │  Hard"`
  - "CKA Buddy" in bold White
  - progress bar built from `█` (filled) and `░` (empty) chars proportional to readiness
  - step info in Cyan
- Line 2: `"  {step.title}"` in bold White
- Render as `Paragraph` with `Style::default().bg(Color::Black)` (or DarkGray)

**Zone 1 — Main content area:**

Recommended structure (more usable than one giant scrolled paragraph):

1. Build a small **pinned prelude** block at the top of Zone 1:
   - objective
   - metadata
   - suggested commands list
2. Build a **scrolling activity** block below it:
   - terminal divider
   - terminal output (tail)
   - inline hint
   - inline success + what_changed + next/verify optional
   - key hints

If you still want a single scrolled feed, keep your current approach, but expect objective/suggestions to scroll out of view during long sessions.

If using the split approach, derive it like:

```rust
let main = Layout::vertical([
    Constraint::Length(pinned_height),
    Constraint::Min(0),
]).split(zones[1]);
```

Then apply bottom scroll only to `main[1]`.

If you keep the single feed, build a `Vec<Line>` in order:

1. One blank line
2. Objective text (bold, wrapped) — use `step.objective`
3. Metadata line: `"  {timebox} min · {difficulty} · {domains}"` — dimmed
4. One blank line
5. If `!run_commands.is_empty()`: label `"  Steps to run:"` in dim White
6. For each command in `run_commands`: `"  [{n}] {cmd}"` — `[n]` in DarkGray, cmd in Yellow
7. One blank line
8. Divider line: `"  ─── Terminal " + "─".repeat(remainder)` in DarkGray
9. Terminal output lines (last N that fit, from `output_log`) — Gray
10. If `hint_message.is_some()`: blank line + `"  ● Hint: {text}"` in Yellow italic
11. If `completion_card.is_some()`: blank line + `"  ✓ {card.done}"` in bold Green, then `"    {what_changed}"` lines in Green
    - Also render `next_commands` and `verify_optional` as dim helper lines to preserve guidance continuity
12. One blank line
13. Key hints: `"  [h] hint  [v] verify  [j/k] nav  [r] next  [b] back  [TAB] suggest"` in DarkGray

For terminal output scroll: calculate `total_lines` in the built Vec, compare to `zones[1].height`. Use `Paragraph::scroll((offset, 0))` where offset ensures the bottom is visible:
```rust
let scroll = total_lines.saturating_sub(zones[1].height as usize) as u16;
```

Render as: `Paragraph::new(Text::from(lines)).block(Block::default().borders(Borders::NONE)).scroll((scroll, 0))`

**Ratatui implementation note:** prefer `Span`-based line building for mixed styles (index, command, icons, dim labels), rather than formatting a single style-uniform string line.

**Zone 2 — Command bar:**
- Line 1: status message in DarkGray (or empty space)
- Line 2: `"  ❯ {command_input}"` — `❯` in Cyan, input in White
- Render with top border only: `Block::default().borders(Borders::TOP)`

---

## Styling Reference

| Element | Color / Style |
|---------|---------------|
| Header background | `Color::Black` bg |
| "CKA Buddy" | Bold White |
| Progress bar `█` | Green |
| Progress bar `░` | DarkGray |
| Step position / type | Cyan |
| Step title (header L2) | Bold White |
| Objective text | Bold White |
| Metadata line | DarkGray |
| "Steps to run:" label | DarkGray |
| `[n]` index | DarkGray |
| Suggested command | Yellow |
| Terminal divider | DarkGray |
| Terminal output | Gray |
| Hint bullet `●` | Yellow |
| Hint text | Yellow italic |
| Success `✓` + done | Bold Green |
| what_changed bullets | Green |
| Key hints bar | DarkGray |
| Command bar border | DarkGray |
| `❯` prompt | Cyan |
| Command input | White |
| Status line | DarkGray |

---

## Creative Layout Directions (Optional Enhancements)

If you want a more distinctive identity than a clean shell, here are low-risk creative layers that still fit terminal ergonomics:

### Option A — Mission Control Header (recommended)

Turn the 2-line header into a compact "status strip" with visual rhythm:

- Left: `CKA BUDDY` mark + tiny pulse glyph (`◉`) that blinks every tick
- Middle: segmented readiness bar (`▰▰▰▱▱`) + numeric percent
- Right: chip-like labels: `[STEP 05/20] [EXAM] [HARD]`
- Line 2 keeps current step title, but truncate with ellipsis for narrow widths

Why this works: you keep a practical shell while giving the product a recognizable signature.

### Option B — Feed Cards Instead of Plain Lines

In Zone 1, render sections as lightweight card blocks separated by subtle spacer lines:

- `Objective` card (bold title + wrapped text)
- `Runbook` card (`[1]`, `[2]`, `[3]` commands)
- `Terminal` card (stream)
- `Assistant` card for hint/success

Implementation note: use borderless inner cards with prefixed labels (`"▸ Objective"`) and dim separators, not heavy full borders.

### Option C — Split Activity Rail

For wider terminals only (`width >= 120`):

- Left 70%: Objective + Runbook + Terminal
- Right 30%: compact live rail (`Hint`, `Verify result`, `Recent status`, `Next actions`)

Fallback to single-feed mode on narrow terminals.

Why this works: adds depth and scanning speed without returning to a full 5-panel overload.

### Option D — Context-Aware Command Bar

Keep the bottom bar, but make it adaptive:

- Left: status text
- Center: prompt (`❯ command_input`)
- Right: mode badge (`[SUGGEST]`, `[VERIFY]`, `[BLOCKED]`) based on latest action/guard outcome

This gives immediate command-state feedback with almost no extra cognitive load.

### Option E — Micro-Animation + Typographic Mood

Subtle dynamics (no noise):

- Animate progress bar fill transitions over 2-3 frames
- Alternate terminal divider glyph pattern on redraw (`─`, `╌`) at low frequency
- Use consistent symbol language: `●` hint, `✓` success, `!` warning, `✕` blocked

Keep all animation optional behind a `reduced_motion` toggle.

---

## Creative Direction Recommendation

Use **Option A + Option D now**, and add **Option C for wide terminals** as a second pass.

That gives a unique visual identity quickly, stays keyboard-first, and avoids reintroducing the clutter that prompted this redesign.

---

## Verification

1. `cargo build` — confirm no compile errors
2. `cargo run` — launch and visually confirm:
   - 3-zone layout renders correctly (no 5-panel)
   - Header shows readiness bar + step title
   - Numbered commands appear in feed
   - Terminal output appears below separator
   - Tab cycles through commands (not always #1)
   - `h` shows hint inline, not in a separate panel
   - `v` pass shows success card inline
   - `j`/`k` navigates steps and clears hint/success
   - Command bar shows `❯` prompt with input
3. Resize terminal window — confirm layout scales gracefully
