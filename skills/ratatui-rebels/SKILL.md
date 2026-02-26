---
name: ratatui-rebels
description: Design and implement production-grade ratatui interfaces inspired by Rebels in the Sky patterns, including screen state machines, interactive widgets, callback routing, hover layers, popups, and dense information layouts.
---

# Ratatui Rebels

Build feature-rich terminal UIs with the same design and interaction patterns used in `rebels-in-the-sky`.

## Use This Skill When

- You are building or refactoring a Rust TUI with `ratatui`.
- You need consistent keyboard + mouse interactions (hover, click, scroll, hotkeys).
- You need multi-panel screens, tabs, popups, and callback-driven actions.
- You need dense, game-like dashboards with bars, tables, lists, and animated image frames.

## Core Design System (Rebels Pattern)

1. **Stateful shell, pluggable screens**
   - Keep a top-level UI controller that owns app state (`Splash`, `NewTeam`, `Main`, etc.) and active tab.
   - Define a shared `Screen` trait (`update`, `render`, optional input handling).
   - Keep panel index behavior in a small `SplitPanel` trait for list/table navigation.

2. **Central callback routing**
   - Route all UI actions through one callback enum (`UiCallback`) and a central dispatcher (`call`).
   - Widgets emit callbacks; business logic happens in callback handlers.
   - Keep callbacks serializable/inspectable and avoid ad-hoc closures in widgets.

3. **Interactive rendering pipeline**
   - Wrap `ratatui::Frame` in a custom frame object that tracks:
     - active interaction layer,
     - hover area,
     - callback registry.
   - Render with explicit helpers:
     - `render_widget`
     - `render_stateful_widget`
     - `render_interactive_widget`
     - `render_stateful_interactive_widget`

4. **Layered input model**
   - Layer 0: main screen widgets.
   - Layer 1: modal/popup widgets.
   - Only widgets in active layer should respond to hover/click/hotkeys.

5. **Stable visual language**
   - Define semantic styles in one place (`UiStyle`): `DEFAULT`, `HEADER`, `OK`, `ERROR`, `HIGHLIGHT`, `OWN_TEAM`, etc.
   - Style by meaning (state/severity/ownership), not by per-widget one-off colors.

## Interaction Contracts

### Interactive widgets

Build widgets with a trait contract similar to:

```rust
trait InteractiveWidget: Widget {
    fn layer(&self) -> usize;
    fn before_rendering(&mut self, area: Rect, registry: &mut CallbackRegistry);
    fn hover_text(&self) -> Text<'_>;
}
```

In `before_rendering`:
- compute hover state,
- register mouse callbacks (click, scroll),
- register keyboard callbacks (hotkeys),
- avoid mutating app state directly.

### Stateful collections (lists/tables)

- Use custom state structs (`offset`, `selected`, `hovered`).
- Keep selected row visible by recomputing visible bounds.
- Register scroll callbacks for next/previous index.
- Register click callbacks for hovered row -> `SetPanelIndex`.

### Footer and discoverability

- Always render context-sensitive controls in the footer.
- Show alternating style segments for key + meaning (`" T "`, `" Travel "`).
- Reserve one dedicated hover-text row for micro-help.

## Layout Recipes

### Fixed design resolution + centering

- Pick a canonical screen size (Rebels uses `160x48`).
- If terminal is larger, center the UI in a fixed viewport.
- If smaller, clamp to available area with `saturating_*` logic.

### Standard shell split

- Vertical split:
  - body (`Min`),
  - footer (`Length(1)`),
  - hover help (`Length(1)`).

### Left filter + right detail pattern

- Left: filter buttons + selectable list.
- Right: rich detail panel with image/metrics/actions.
- This pattern is reused heavily and scales across domains.

## Data-Dense Presentation Patterns

- Use segmented bars (`▰` / `▱`) for capacities and status.
- Use semantic colors for each resource/metric.
- Use compact comparison rows (`old -> new`) for upgrades.
- For pixel-art images, convert two RGBA rows into one terminal row using half-block glyphs (`▀`, `▄`).

## Popup/Modal Pattern

- Keep popup messages as an enum with data payloads.
- One popup renderer handles all variants.
- Popups consume input first; underlying panels should not process keys while modal is active.
- Use active layer switching to prevent click-through bugs.

## Implementation Workflow

1. Define/extend domain callbacks in a single enum.
2. Implement callback handling in one dispatcher function.
3. Add or extend a screen struct implementing `Screen`.
4. Build layout with clear split hierarchy.
5. Use interactive widgets that only emit callbacks.
6. Add footer key hints + hover text for discoverability.
7. Add popup variant if action requires confirmation.
8. Validate both keyboard-only and mouse-enabled flows.

## Quality Checklist

- Input works with keyboard only.
- Mouse hover/click/scroll works and respects active layer.
- No direct domain mutation in widget render code.
- Styles are semantic and centralized.
- Footer hints and hover help are present for critical actions.
- Selected list/table row stays visible while navigating.
- Modal blocks underlying interactions.

## Anti-Patterns to Avoid

- Binding business logic directly inside widget render methods.
- Scattering hotkey handling across unrelated files with no registry.
- One-off ad-hoc colors instead of semantic style constants.
- Rendering popups without interaction layering.
- Letting selected indices drift out of bounds when list data changes.

## Reference

See `references/rebels-ratatui-patterns.md` for concrete source-derived patterns and file map.
