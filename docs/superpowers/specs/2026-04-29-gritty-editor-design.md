# Gritty ANSI Block Art Editor — Design Spec

**Date:** 2026-04-29  
**Status:** Approved

---

## 1. Overview

Gritty is a browser-based ANSI block art editor built with Rust + Dioxus, compiled to WebAssembly. It runs entirely client-side — no server, no backend. Static files produced by `dx build --release`.

**Stack:**
- Language: Rust
- UI framework: Dioxus 0.7 (web target)
- Styling: Tailwind CSS via `dx` built-in integration, Monokai Pro color tokens as CSS variables
- Rendering: HTML Canvas via `web_sys`
- Theme: Monokai Pro

---

## 2. Data Model

```rust
struct AppState {
    project: Project,
    tool: Tool,           // Brush | Eraser
    active_glyph: char,
    fg_color: [u8; 3],
    bg_color: [u8; 3],
    color_target: ColorTarget, // Fg | Bg — which swatch the color wheel edits
    playback: PlaybackState,
}

enum ColorTarget { Fg, Bg }

struct Project {
    width: u32,
    height: u32,
    frames: Vec<Frame>,
    active_frame: usize,
}

struct Frame {
    cells: Vec<Cell>,     // row-major, length = width * height
}

struct Cell {
    fg: [u8; 3],
    bg: [u8; 3],
    ch: char,
}

struct PlaybackState {
    playing: bool,
    delay_ms: u32,
}
```

All state lives in Dioxus `Signal`s. No external state library.

---

## 3. Glyph Palette

Grouped by category in the left panel glyph grid:

| Category      | Glyphs                          |
|---------------|---------------------------------|
| Full          | `█`                             |
| Horiz half    | `▀` `▄`                         |
| Vert half     | `▌` `▐`                         |
| Shade         | `░` `▒` `▓`                     |
| Quadrant      | `▖` `▗` `▘` `▙` `▚` `▛` `▜` `▝` `▞` `▟` |
| Space         | ` `                             |

---

## 4. Canvas Rendering

- Single `<canvas>` element, sized to `width * 8` × `height * 16` pixels
- Cell size: **8px wide × 16px tall** (1:2 terminal aspect ratio)
- `use_effect` watches the project signal; on change, re-draws all cells:
  1. Clear canvas
  2. For each cell: `fill_rect` with `bg` color, then `fill_text` with `ch` in `fg` color
- Font: monospace, 16px, set once on context

**Mouse interaction:**
- `onmousedown` → start stroke, paint cell at cursor
- `onmousemove` → if button held, paint cell at cursor
- `onmouseup` / `onmouseleave` → end stroke
- Cell index: `col = floor(x / 8)`, `row = floor(y / 16)`

---

## 5. UI Layout

Layout A — fixed panels, canvas center:

```
┌─────────────────────────────────────────┐
│              TopBar                     │
├──────────┬──────────────────┬───────────┤
│          │                  │           │
│ Left     │     Canvas       │  Right    │
│ Panel    │   (<canvas>)     │  Panel    │
│          │                  │           │
├──────────┴──────────────────┴───────────┤
│              Timeline                   │
└─────────────────────────────────────────┘
```

### TopBar
- Project name (static "Gritty" for MVP)
- Canvas size display (e.g. `80×24`)
- Import button → file picker (`.json`, `.ans`, `.txt`)
- Export button → modal

### LeftPanel
- Tool selector: Brush / Eraser toggle
- Glyph grid: grouped by category, active glyph highlighted

### Canvas
- `<canvas>` element, centered with scrollable overflow for large grids
- Receives all mouse events

### RightPanel
- HSV color wheel: outer hue ring + inner saturation/value square
- Hex input field (synced with wheel)
- FG swatch — click to set `color_target = Fg`; wheel edits `fg_color`
- BG swatch — click to set `color_target = Bg`; wheel edits `bg_color`
- Active target indicated by a highlight border on the swatch

### Timeline
- Frame thumbnails (mini canvas renders)
- Add / Duplicate / Delete frame buttons
- Drag-to-reorder (MVP: up/down buttons)
- Play/Stop button
- Delay input (ms)

---

## 6. Import / Export

### Export modal (triggered from TopBar)

Three tabs:

| Tab | Scope toggle | Output |
|-----|-------------|--------|
| ANSI escape string | Current frame / All frames | Copyable text in `<pre>` |
| Plain text + ANSI  | Current frame / All frames | Copyable text in `<pre>` |
| JSON | Always all frames | `.json` file download |

Multi-frame ANSI export: frames concatenated with `\x1b[2J\x1b[H` (clear + home) as separator.

### Import (file picker in TopBar)

| Format | Handling |
|--------|----------|
| `.json` | Full project restore (single or multi-frame, detected from `frames.len()`) |
| `.ans` / `.txt` | Best-effort ANSI parse → single frame, or multiple frames if `\x1b[2J` found between sections |

---

## 7. File Structure

```
gritty/
├── Cargo.toml
├── Dioxus.toml
├── tailwind.config.js
├── input.css
├── assets/
│   └── main.css
└── src/
    ├── main.rs
    ├── state.rs
    ├── canvas.rs
    ├── export.rs
    ├── import.rs
    └── components/
        ├── top_bar.rs
        ├── left_panel.rs
        ├── right_panel.rs
        └── timeline.rs
```

---

## 8. Key Dependencies

```toml
[dependencies]
dioxus = { version = "0.7", features = ["web"] }
web-sys = { version = "0.3", features = [
  "CanvasRenderingContext2d", "HtmlCanvasElement",
  "MouseEvent", "Window", "Document", "Element",
  "HtmlElement", "EventTarget"
]}
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## 9. Out of Scope (MVP)

- Undo/redo
- Flood fill
- Collaborative editing
- GPU rendering
- Layer system
- Straight-line modifier (Shift)
- Color pick modifier (Alt)
- Dirty-region canvas optimization

---

## 10. Success Criteria

- User can paint cells with any glyph from the full palette
- FG + BG colors selectable via color wheel or hex
- Multiple frames: add, duplicate, delete, reorder
- Playback loops frames at configurable delay
- Export: ANSI string (single/all frames), JSON
- Import: JSON project, ANSI text (single/multi-frame)
- Exported ANSI renders correctly in a real terminal
