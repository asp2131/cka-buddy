# Rebels in the Sky Ratatui Patterns

Concrete patterns extracted from `rebels-in-the-sky/src/ui` and adjacent runtime modules.

## 1) UI Shell and State Machine

- `src/ui/ui_screen.rs`
  - Global `UiState` (`Splash`, `NewTeam`, `Main`, `SpaceAdventure`) and `UiTab` routing.
  - Maintains panel instances and popup queue.
  - Central render flow:
    1. create `UiFrame`,
    2. set active interaction layer,
    3. render active screen,
    4. render footer,
    5. render popup,
    6. persist callback registry.

## 2) Frame Wrapper + Interaction Layers

- `src/ui/ui_frame.rs`
  - Wraps `ratatui::Frame` with hover region and callback registry.
  - Centers all content into canonical screen size (`UI_SCREEN_SIZE`).
  - Provides interaction-aware render methods.
  - Shows hover help text row when interactive widget is hovered.

## 3) Callback Bus

- `src/ui/ui_callback.rs`
  - Large `UiCallback` enum captures all UI intents.
  - `UiCallback::call(&mut App)` is the single state transition/action router.
  - `CallbackRegistry` stores mouse and keyboard callbacks for currently rendered frame.
  - Supports scoped callbacks by `Rect` and global callbacks for events like scroll.

## 4) Widget Contracts

- `src/ui/traits.rs`
  - `Screen` trait standardizes update/render/input and footer hints.
  - `SplitPanel` trait abstracts index navigation.
  - `InteractiveWidget` and `InteractiveStatefulWidget` define hover + callback registration hooks.

## 5) Reusable Interactive Components

- `src/ui/button.rs`
  - Button supports hotkeys, hover style, disabled reasoning text, and layer assignment.
  - Hotkey underline rendering on first text line.
  - Registers click and keyboard callbacks in `before_rendering`.

- `src/ui/clickable_list.rs`
  - Custom list with selection/hover state and viewport offset logic.
  - Scroll wheel mapped to next/previous index callbacks.

- `src/ui/clickable_table.rs`
  - Custom table equivalent with selectable/hoverable rows.
  - Supports header, column constraints, and stateful row windowing.

- `src/ui/hover_text_span.rs`, `src/ui/hover_text_line.rs`
  - Fine-grained hover help at span-level inside a line.

## 6) Styling and Semantics

- `src/ui/constants.rs`
  - UI constants (`UI_SCREEN_SIZE`, panel widths, bars length).
  - Semantic `UiStyle` palette: ownership, status, network, selection, highlight.

## 7) Screen Composition Patterns

- Left filter/actions + right details:
  - `src/ui/team_panel.rs`
  - `src/ui/player_panel.rs`
  - `src/ui/my_team_panel.rs`
  - `src/ui/tournament_panel.rs`
  - `src/ui/swarm_panel.rs`

- Specialized worlds:
  - `src/ui/galaxy_panel.rs` (zoomable map + clickable planet bounding boxes)
  - `src/ui/space_screen.rs` (real-time simulation UI with compact HUD)

## 8) Modal/Popup Subsystem

- `src/ui/popup_message.rs`
  - Popup enum variants for confirmations, errors, tutorials, and dynamic content.
  - Single renderer computes popup rect by variant and terminal bounds.
  - Dedicated input consumption for modal-first handling.
  - Layered interactions prevent background widget activation.

## 9) Dense Data Visualization Utilities

- `src/ui/widgets.rs`
  - Shared blocks, buttons, bars, stat spans, upgrade views, and rich entity cards.
  - Uses semantic coloring and compact text+glyph visual encoding.

- `src/ui/utils.rs`
  - Image-to-terminal line conversion with half-block glyphs.
  - Big text builder and input validation helpers.

## 10) Runtime Event Loop Integration

- `src/tui.rs`
  - Alternate-screen lifecycle, mouse capture, draw throttling.

- `src/crossterm_event_handler.rs`
  - Poll/read loop with event draining and scroll-collapse optimization.

- `src/tick_event_handler.rs`
  - Separate fast/slow tick streams for UI/game cadence.

## Practical Takeaways

- Treat UI as a deterministic projection of app state + callback registrations.
- Keep interactivity declarative during render (register intent, execute later).
- Use layered callback registries to simplify modal correctness.
- Build custom stateful widgets where default ratatui widgets are not enough.
- Preserve discoverability through footer key hints and hover descriptions.
