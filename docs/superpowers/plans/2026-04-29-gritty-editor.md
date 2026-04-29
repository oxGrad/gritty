# Gritty ANSI Block Art Editor — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a client-side Dioxus/WASM web app where users paint glyphs onto a grid, pick colors, manage animation frames, and export ANSI escape sequences.

**Architecture:** Global `Signal<AppState>` provided at root via `use_context_provider`; child components read/write it via `use_context::<Signal<AppState>>()`. The Canvas component uses `use_effect` that reads the project signal — Dioxus automatically reruns the effect whenever state changes, triggering a `web_sys::CanvasRenderingContext2d` redraw. Business logic (state mutations, export, import, color math) lives in pure functions unit-tested with `cargo test`.

**Tech Stack:** Rust 1.95, Dioxus 0.7 (`web` feature), web-sys, wasm-bindgen, wasm-bindgen-futures, js-sys, serde/serde_json, gloo-timers, Tailwind CSS (standalone CLI), Monokai Pro CSS custom properties.

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies + build profile |
| `Dioxus.toml` | dx config: web target, asset dir |
| `tailwind.config.js` | Tailwind content paths |
| `input.css` | Tailwind directives + Monokai Pro CSS vars |
| `assets/tailwind.css` | Built Tailwind output (gitignored) |
| `src/main.rs` | `main()` + `App` component + top-level layout |
| `src/state.rs` | All types: `AppState`, `Project`, `Frame`, `Cell`, `Tool`, `ColorTarget`, `PlaybackState` |
| `src/color.rs` | Pure color math: `hsv_to_rgb`, `rgb_to_hsv`, `parse_hex`, `to_hex` |
| `src/export.rs` | `export_ansi_frame`, `export_ansi_all`, `export_json` |
| `src/import.rs` | `import_json`, `import_ansi` |
| `src/canvas.rs` | `Canvas` Dioxus component: draw loop + mouse events |
| `src/components/mod.rs` | Re-exports |
| `src/components/left_panel.rs` | `LeftPanel`: tool selector + glyph grid |
| `src/components/right_panel.rs` | `RightPanel`: HSV color wheel canvas + hex input + swatches |
| `src/components/top_bar.rs` | `TopBar`: size display + export modal + import file trigger |
| `src/components/timeline.rs` | `Timeline`: frame list + add/dup/del/reorder + playback |

**Constants (in `state.rs`):**
```
CELL_W: f64 = 8.0   // canvas pixels per cell, x
CELL_H: f64 = 16.0  // canvas pixels per cell, y (1:2 terminal ratio)
```

---

## Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `Dioxus.toml`
- Create: `tailwind.config.js`
- Create: `input.css`
- Create: `assets/.gitkeep`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize Cargo project**

```bash
cd /path/to/gritty
cargo init .
```

Expected: `src/main.rs` and `Cargo.toml` created (overwrite the stub `main.rs`).

- [ ] **Step 2: Write `Cargo.toml`**

```toml
[package]
name = "gritty"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "gritty"
path = "src/main.rs"

[dependencies]
dioxus = { version = "0.7", features = ["web"] }
web-sys = { version = "0.3", features = [
    "CanvasRenderingContext2d", "HtmlCanvasElement",
    "HtmlInputElement", "HtmlAnchorElement",
    "MouseEvent", "Window", "Document",
    "Element", "HtmlElement", "EventTarget",
    "ImageData", "FileList", "File",
    "Blob", "BlobPropertyBag", "Url",
] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
gloo-timers = { version = "0.3", features = ["futures"] }

[profile.release]
opt-level = "s"
lto = true
```

- [ ] **Step 3: Write `Dioxus.toml`**

```toml
[application]
name = "gritty"
default_platform = "web"
out_dir = "dist"
asset_dir = "assets"

[web.app]
title = "Gritty — ANSI Block Art"

[web.watcher]
reload_html = true
watch_path = ["src", "assets"]
```

- [ ] **Step 4: Write `tailwind.config.js`**

```js
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./index.html"],
  theme: { extend: {} },
  plugins: [],
}
```

- [ ] **Step 5: Write `input.css`**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  --mk-bg:      #2d2a2e;
  --mk-bg-dark: #221f22;
  --mk-bg-mid:  #403e41;
  --mk-fg:      #fcfcfa;
  --mk-muted:   #9ca0a4;
  --mk-pink:    #ff6188;
  --mk-orange:  #fc9867;
  --mk-yellow:  #ffd866;
  --mk-green:   #a9dc76;
  --mk-cyan:    #78dce8;
  --mk-purple:  #ab9df2;
}

body {
  background-color: var(--mk-bg);
  color: var(--mk-fg);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  margin: 0;
  padding: 0;
  height: 100vh;
  overflow: hidden;
}
```

- [ ] **Step 6: Write stub `src/main.rs`**

```rust
#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/tailwind.css") }
        div { class: "flex flex-col h-screen",
            p { "Gritty loading..." }
        }
    }
}
```

- [ ] **Step 7: Add `.gitignore` entries**

Append to `.gitignore`:
```
/dist
/assets/tailwind.css
node_modules/
```

- [ ] **Step 8: Build Tailwind CSS (one-time)**

Install Tailwind standalone CLI (no Node project needed):
```bash
# macOS
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64
chmod +x tailwindcss-macos-arm64
mv tailwindcss-macos-arm64 tailwindcss
./tailwindcss -i input.css -o assets/tailwind.css
```

Expected: `assets/tailwind.css` created.

- [ ] **Step 9: Verify the project builds**

```bash
dx serve --platform web
```

Expected: browser opens, shows "Gritty loading..." with dark background (no errors in terminal).
Stop with Ctrl+C.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Dioxus.toml tailwind.config.js input.css assets/.gitkeep src/main.rs .gitignore
git commit -m "feat: scaffold Dioxus web project with Tailwind and Monokai Pro"
```

---

## Task 2: State Types

**Files:**
- Create: `src/state.rs`
- Modify: `src/main.rs` (add `mod state;`)

- [ ] **Step 1: Write `src/state.rs`**

```rust
use serde::{Deserialize, Serialize};

pub const CELL_W: f64 = 8.0;
pub const CELL_H: f64 = 16.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub ch: char,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { fg: [252, 252, 250], bg: [34, 31, 34], ch: ' ' }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub cells: Vec<Cell>,
}

impl Frame {
    pub fn new(width: u32, height: u32) -> Self {
        Frame {
            cells: vec![Cell::default(); (width * height) as usize],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Frame>,
    pub active_frame: usize,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            width: 40,
            height: 20,
            frames: vec![Frame::new(40, 20)],
            active_frame: 0,
        }
    }
}

impl Project {
    pub fn paint_cell(&mut self, col: u32, row: u32, fg: [u8; 3], bg: [u8; 3], ch: char) {
        if col >= self.width || row >= self.height { return; }
        let idx = (row * self.width + col) as usize;
        if let Some(frame) = self.frames.get_mut(self.active_frame) {
            if let Some(cell) = frame.cells.get_mut(idx) {
                cell.fg = fg;
                cell.bg = bg;
                cell.ch = ch;
            }
        }
    }

    pub fn erase_cell(&mut self, col: u32, row: u32) {
        self.paint_cell(col, row, Cell::default().fg, Cell::default().bg, ' ');
    }

    pub fn add_frame(&mut self) {
        self.frames.push(Frame::new(self.width, self.height));
    }

    pub fn duplicate_frame(&mut self, index: usize) {
        if index < self.frames.len() {
            let frame = self.frames[index].clone();
            self.frames.insert(index + 1, frame);
        }
    }

    pub fn delete_frame(&mut self, index: usize) {
        if self.frames.len() <= 1 { return; }
        self.frames.remove(index);
        if self.active_frame >= self.frames.len() {
            self.active_frame = self.frames.len() - 1;
        }
    }

    pub fn move_frame_up(&mut self, index: usize) {
        if index == 0 { return; }
        self.frames.swap(index, index - 1);
        if self.active_frame == index { self.active_frame -= 1; }
        else if self.active_frame == index - 1 { self.active_frame += 1; }
    }

    pub fn move_frame_down(&mut self, index: usize) {
        if index + 1 >= self.frames.len() { return; }
        self.frames.swap(index, index + 1);
        if self.active_frame == index { self.active_frame += 1; }
        else if self.active_frame == index + 1 { self.active_frame -= 1; }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Tool {
    Brush,
    Eraser,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColorTarget {
    Fg,
    Bg,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaybackState {
    pub playing: bool,
    pub delay_ms: u32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState { playing: false, delay_ms: 100 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    pub project: Project,
    pub tool: Tool,
    pub active_glyph: char,
    pub fg_color: [u8; 3],
    pub bg_color: [u8; 3],
    pub color_target: ColorTarget,
    pub playback: PlaybackState,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            project: Project::default(),
            tool: Tool::Brush,
            active_glyph: '█',
            fg_color: [255, 97, 136],    // Monokai pink
            bg_color: [34, 31, 34],      // Monokai dark bg
            color_target: ColorTarget::Fg,
            playback: PlaybackState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_project_has_one_frame() {
        let p = Project::default();
        assert_eq!(p.frames.len(), 1);
        assert_eq!(p.width, 40);
        assert_eq!(p.height, 20);
    }

    #[test]
    fn paint_cell_updates_correct_index() {
        let mut p = Project::default();
        p.paint_cell(1, 0, [255, 0, 0], [0, 0, 0], '█');
        assert_eq!(p.frames[0].cells[1].ch, '█');
        assert_eq!(p.frames[0].cells[0].ch, ' ');
    }

    #[test]
    fn paint_cell_out_of_bounds_does_not_panic() {
        let mut p = Project::default();
        p.paint_cell(999, 999, [255, 0, 0], [0, 0, 0], '█');
    }

    #[test]
    fn erase_cell_resets_to_default() {
        let mut p = Project::default();
        p.paint_cell(0, 0, [255, 0, 0], [255, 0, 0], '█');
        p.erase_cell(0, 0);
        assert_eq!(p.frames[0].cells[0], Cell::default());
    }

    #[test]
    fn add_frame_increments_count() {
        let mut p = Project::default();
        p.add_frame();
        assert_eq!(p.frames.len(), 2);
    }

    #[test]
    fn duplicate_frame_inserts_after() {
        let mut p = Project::default();
        p.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '█');
        p.duplicate_frame(0);
        assert_eq!(p.frames.len(), 2);
        assert_eq!(p.frames[1].cells[0].ch, '█');
    }

    #[test]
    fn delete_frame_refuses_last() {
        let mut p = Project::default();
        p.delete_frame(0);
        assert_eq!(p.frames.len(), 1);
    }

    #[test]
    fn delete_frame_removes_and_clamps_active() {
        let mut p = Project::default();
        p.add_frame();
        p.active_frame = 1;
        p.delete_frame(1);
        assert_eq!(p.active_frame, 0);
    }

    #[test]
    fn move_frame_up_swaps_correctly() {
        let mut p = Project::default();
        p.add_frame();
        p.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '█');
        p.active_frame = 0;
        let ch_before = p.frames[0].cells[0].ch;
        p.move_frame_down(0);
        assert_eq!(p.frames[1].cells[0].ch, ch_before);
        assert_eq!(p.active_frame, 1);
    }
}
```

- [ ] **Step 2: Add `mod state;` to `src/main.rs`**

```rust
#![allow(non_snake_case)]
mod state;
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/tailwind.css") }
        div { class: "flex flex-col h-screen",
            p { "Gritty loading..." }
        }
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: all 9 tests pass. Zero compile errors.

- [ ] **Step 4: Commit**

```bash
git add src/state.rs src/main.rs
git commit -m "feat: add AppState, Project, Frame, Cell types with unit tests"
```

---

## Task 3: Color Utilities

**Files:**
- Create: `src/color.rs`
- Modify: `src/main.rs` (add `mod color;`)

- [ ] **Step 1: Write `src/color.rs`**

```rust
/// Convert HSV (h: 0-360, s: 0-1, v: 0-1) to RGB [u8; 3].
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
    let h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else              { (c, 0.0, x) };
    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}

/// Convert RGB [u8; 3] to HSV (h: 0-360, s: 0-1, v: 0-1).
pub fn rgb_to_hsv(rgb: [u8; 3]) -> (f64, f64, f64) {
    let r = rgb[0] as f64 / 255.0;
    let g = rgb[1] as f64 / 255.0;
    let b = rgb[2] as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

/// Parse "#RRGGBB" or "RRGGBB" hex string into [u8; 3].
pub fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Format [u8; 3] as "#RRGGBB".
pub fn to_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_hsv_to_rgb() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), [255, 0, 0]);
    }

    #[test]
    fn green_hsv_to_rgb() {
        assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), [0, 255, 0]);
    }

    #[test]
    fn blue_hsv_to_rgb() {
        assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), [0, 0, 255]);
    }

    #[test]
    fn black_hsv_to_rgb() {
        assert_eq!(hsv_to_rgb(0.0, 0.0, 0.0), [0, 0, 0]);
    }

    #[test]
    fn white_hsv_to_rgb() {
        assert_eq!(hsv_to_rgb(0.0, 0.0, 1.0), [255, 255, 255]);
    }

    #[test]
    fn rgb_to_hsv_red() {
        let (h, s, v) = rgb_to_hsv([255, 0, 0]);
        assert!((h - 0.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
    }

    #[test]
    fn roundtrip_rgb_hsv() {
        let original = [169, 220, 118]; // Monokai green
        let (h, s, v) = rgb_to_hsv(original);
        let back = hsv_to_rgb(h, s, v);
        // allow ±2 due to floating point rounding
        for i in 0..3 {
            let diff = (original[i] as i16 - back[i] as i16).abs();
            assert!(diff <= 2, "channel {i}: original={} back={}", original[i], back[i]);
        }
    }

    #[test]
    fn parse_hex_with_hash() {
        assert_eq!(parse_hex("#FF6188"), Some([255, 97, 136]));
    }

    #[test]
    fn parse_hex_without_hash() {
        assert_eq!(parse_hex("a9dc76"), Some([169, 220, 118]));
    }

    #[test]
    fn parse_hex_invalid() {
        assert_eq!(parse_hex("#GGGGGG"), None);
        assert_eq!(parse_hex("#FFF"), None);
    }

    #[test]
    fn to_hex_roundtrip() {
        let color = [255, 97, 136];
        assert_eq!(parse_hex(&to_hex(color)), Some(color));
    }
}
```

- [ ] **Step 2: Add `mod color;` to `src/main.rs`**

Add `mod color;` below `mod state;`.

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: all color tests pass (13 tests total now).

- [ ] **Step 4: Commit**

```bash
git add src/color.rs src/main.rs
git commit -m "feat: add color utilities (HSV/RGB, hex) with unit tests"
```

---

## Task 4: Export Module

**Files:**
- Create: `src/export.rs`
- Modify: `src/main.rs` (add `mod export;`)

- [ ] **Step 1: Write `src/export.rs`**

```rust
use crate::state::{Frame, Project};

/// Generate an ANSI escape string for one frame.
/// Uses truecolor: ESC[38;2;R;G;Bm (fg) and ESC[48;2;R;G;Bm (bg).
pub fn export_ansi_frame(frame: &Frame, width: u32, height: u32) -> String {
    let mut out = String::new();
    for row in 0..height as usize {
        for col in 0..width as usize {
            let cell = &frame.cells[row * width as usize + col];
            out.push_str(&format!(
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m{}",
                cell.fg[0], cell.fg[1], cell.fg[2],
                cell.bg[0], cell.bg[1], cell.bg[2],
                cell.ch
            ));
        }
        out.push_str("\x1b[0m\n");
    }
    out.push_str("\x1b[0m");
    out
}

/// Generate ANSI for all frames, separated by ESC[2J ESC[H (clear + cursor home).
pub fn export_ansi_all(project: &Project) -> String {
    project.frames
        .iter()
        .map(|f| export_ansi_frame(f, project.width, project.height))
        .collect::<Vec<_>>()
        .join("\x1b[2J\x1b[H")
}

/// Serialize the full project to a JSON string.
pub fn export_json(project: &Project) -> String {
    serde_json::to_string_pretty(project).expect("project serialization never fails")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Cell, Frame, Project};

    fn make_frame_with_cell(ch: char, fg: [u8; 3], bg: [u8; 3]) -> Frame {
        let mut frame = Frame::new(2, 1);
        frame.cells[0] = Cell { fg, bg, ch };
        frame
    }

    #[test]
    fn ansi_frame_contains_escape_sequences() {
        let frame = make_frame_with_cell('█', [255, 97, 136], [34, 31, 34]);
        let out = export_ansi_frame(&frame, 2, 1);
        assert!(out.contains("\x1b[38;2;255;97;136m"));
        assert!(out.contains("\x1b[48;2;34;31;34m"));
        assert!(out.contains('█'));
        assert!(out.ends_with("\x1b[0m"));
    }

    #[test]
    fn ansi_frame_row_count_matches_height() {
        let frame = Frame::new(5, 3);
        let out = export_ansi_frame(&frame, 5, 3);
        // 3 rows → 3 newlines
        assert_eq!(out.matches('\n').count(), 3);
    }

    #[test]
    fn ansi_all_inserts_clear_between_frames() {
        let mut project = Project::default();
        project.add_frame();
        let out = export_ansi_all(&project);
        assert!(out.contains("\x1b[2J\x1b[H"));
    }

    #[test]
    fn json_export_is_valid_json() {
        let project = Project::default();
        let json = export_json(&project);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["width"], 40);
        assert_eq!(parsed["height"], 20);
        assert!(parsed["frames"].is_array());
    }

    #[test]
    fn json_export_roundtrips_cell_data() {
        let mut project = Project::default();
        project.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '▄');
        let json = export_json(&project);
        let parsed: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.frames[0].cells[0].ch, '▄');
        assert_eq!(parsed.frames[0].cells[0].fg, [255, 0, 0]);
    }
}
```

- [ ] **Step 2: Add `mod export;` to `src/main.rs`**

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: all export tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/export.rs src/main.rs
git commit -m "feat: add export module (ANSI + JSON) with unit tests"
```

---

## Task 5: Import Module

**Files:**
- Create: `src/import.rs`
- Modify: `src/main.rs` (add `mod import;`)

- [ ] **Step 1: Write `src/import.rs`**

```rust
use crate::state::{Cell, Frame, Project};

/// Deserialize a JSON string into a Project.
pub fn import_json(json: &str) -> Result<Project, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Parse an ANSI escape string into a Project.
/// Splits on ESC[2J to detect multiple frames.
/// Best-effort: cells that can't be parsed get Cell::default().
pub fn import_ansi(text: &str, width: u32, height: u32) -> Project {
    let frame_texts: Vec<&str> = text.split("\x1b[2J").collect();
    let frames: Vec<Frame> = frame_texts
        .iter()
        .map(|t| parse_ansi_frame(t, width, height))
        .collect();
    let frames = if frames.is_empty() {
        vec![Frame::new(width, height)]
    } else {
        frames
    };
    Project { width, height, frames, active_frame: 0 }
}

fn parse_ansi_frame(text: &str, width: u32, height: u32) -> Frame {
    let mut frame = Frame::new(width, height);
    let mut col: u32 = 0;
    let mut row: u32 = 0;
    let mut fg: [u8; 3] = Cell::default().fg;
    let mut bg: [u8; 3] = Cell::default().bg;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            // parse ESC[ sequence
            i += 2;
            let start = i;
            while i < bytes.len() && bytes[i] != b'm' && bytes[i] != b'H' && bytes[i] != b'J' {
                i += 1;
            }
            if i >= bytes.len() { break; }
            let terminator = bytes[i] as char;
            let seq = std::str::from_utf8(&bytes[start..i]).unwrap_or("");
            match terminator {
                'm' => parse_sgr(seq, &mut fg, &mut bg),
                'H' => {
                    let parts: Vec<&str> = seq.split(';').collect();
                    row = parts.first().and_then(|s| s.parse::<u32>().ok()).unwrap_or(1).saturating_sub(1);
                    col = parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1).saturating_sub(1);
                }
                _ => {}
            }
            i += 1;
        } else if bytes[i] == b'\n' {
            row += 1;
            col = 0;
            i += 1;
        } else {
            // printable character
            let ch = text[i..].chars().next().unwrap_or(' ');
            if col < width && row < height {
                let idx = (row * width + col) as usize;
                if let Some(cell) = frame.cells.get_mut(idx) {
                    cell.fg = fg;
                    cell.bg = bg;
                    cell.ch = ch;
                }
            }
            i += ch.len_utf8();
            col += 1;
        }
    }
    frame
}

fn parse_sgr(seq: &str, fg: &mut [u8; 3], bg: &mut [u8; 3]) {
    if seq == "0" || seq.is_empty() {
        *fg = Cell::default().fg;
        *bg = Cell::default().bg;
        return;
    }
    let parts: Vec<u8> = seq.split(';').filter_map(|s| s.parse().ok()).collect();
    let mut j = 0;
    while j < parts.len() {
        match parts[j] {
            38 if parts.get(j + 1) == Some(&2) => {
                if let (Some(&r), Some(&g), Some(&b)) =
                    (parts.get(j + 2), parts.get(j + 3), parts.get(j + 4))
                {
                    *fg = [r, g, b];
                    j += 5;
                    continue;
                }
            }
            48 if parts.get(j + 1) == Some(&2) => {
                if let (Some(&r), Some(&g), Some(&b)) =
                    (parts.get(j + 2), parts.get(j + 3), parts.get(j + 4))
                {
                    *bg = [r, g, b];
                    j += 5;
                    continue;
                }
            }
            0 => {
                *fg = Cell::default().fg;
                *bg = Cell::default().bg;
            }
            _ => {}
        }
        j += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{export_ansi_frame, export_json};
    use crate::state::Project;

    #[test]
    fn json_roundtrip() {
        let mut project = Project::default();
        project.paint_cell(2, 1, [255, 97, 136], [34, 31, 34], '▀');
        let json = export_json(&project);
        let imported = import_json(&json).unwrap();
        assert_eq!(imported.frames[0].cells[2 + 40].ch, '▀');
        assert_eq!(imported.frames[0].cells[2 + 40].fg, [255, 97, 136]);
    }

    #[test]
    fn json_import_invalid_returns_err() {
        assert!(import_json("not json").is_err());
    }

    #[test]
    fn ansi_roundtrip_single_frame() {
        let mut project = Project::default();
        project.paint_cell(0, 0, [255, 0, 0], [0, 0, 0], '█');
        let ansi = export_ansi_frame(&project.frames[0], project.width, project.height);
        let imported = import_ansi(&ansi, project.width, project.height);
        assert_eq!(imported.frames.len(), 1);
        assert_eq!(imported.frames[0].cells[0].ch, '█');
        assert_eq!(imported.frames[0].cells[0].fg, [255, 0, 0]);
    }

    #[test]
    fn ansi_import_multi_frame() {
        let mut project = Project::default();
        project.add_frame();
        let text = format!(
            "{}\x1b[2J{}",
            export_ansi_frame(&project.frames[0], project.width, project.height),
            export_ansi_frame(&project.frames[1], project.width, project.height),
        );
        let imported = import_ansi(&text, project.width, project.height);
        assert_eq!(imported.frames.len(), 2);
    }

    #[test]
    fn ansi_import_empty_string_gives_blank_frame() {
        let imported = import_ansi("", 10, 5);
        assert_eq!(imported.frames.len(), 1);
        assert_eq!(imported.frames[0].cells.len(), 50);
    }
}
```

- [ ] **Step 2: Add `mod import;` to `src/main.rs`**

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: all tests pass (22+ tests now).

- [ ] **Step 4: Commit**

```bash
git add src/import.rs src/main.rs
git commit -m "feat: add import module (JSON + ANSI parsing) with unit tests"
```

---

## Task 6: Canvas Component

**Files:**
- Create: `src/canvas.rs`
- Modify: `src/main.rs` (add `mod canvas;` + wire into App)

- [ ] **Step 1: Write `src/canvas.rs`**

```rust
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::state::{AppState, ColorTarget, Project, Tool, CELL_H, CELL_W};

pub const CANVAS_ID: &str = "main-canvas";

fn get_canvas_ctx() -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let canvas = web_sys::window()?
        .document()?
        .get_element_by_id(CANVAS_ID)?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d").ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    Some((canvas, ctx))
}

fn draw_project(project: &Project) {
    let Some((canvas, ctx)) = get_canvas_ctx() else { return };

    let px_w = (project.width as f64 * CELL_W) as u32;
    let px_h = (project.height as f64 * CELL_H) as u32;
    canvas.set_width(px_w);
    canvas.set_height(px_h);

    ctx.set_font(&format!("{}px monospace", CELL_H as u32));
    ctx.set_text_baseline("top");
    ctx.clear_rect(0.0, 0.0, px_w as f64, px_h as f64);

    let frame = &project.frames[project.active_frame];
    for row in 0..project.height {
        for col in 0..project.width {
            let idx = (row * project.width + col) as usize;
            let cell = &frame.cells[idx];
            let x = col as f64 * CELL_W;
            let y = row as f64 * CELL_H;

            // background
            ctx.set_fill_style_str(&format!(
                "rgb({},{},{})", cell.bg[0], cell.bg[1], cell.bg[2]
            ));
            ctx.fill_rect(x, y, CELL_W, CELL_H);

            // glyph
            if cell.ch != ' ' {
                ctx.set_fill_style_str(&format!(
                    "rgb({},{},{})", cell.fg[0], cell.fg[1], cell.fg[2]
                ));
                let _ = ctx.fill_text(&cell.ch.to_string(), x, y);
            }
        }
    }
}

fn coords_to_cell(x: f64, y: f64) -> (u32, u32) {
    ((x / CELL_W) as u32, (y / CELL_H) as u32)
}

fn apply_tool(state: &mut AppState, col: u32, row: u32) {
    match state.tool {
        Tool::Brush => {
            let fg = state.fg_color;
            let bg = state.bg_color;
            let ch = state.active_glyph;
            state.project.paint_cell(col, row, fg, bg, ch);
        }
        Tool::Eraser => {
            state.project.erase_cell(col, row);
        }
    }
}

#[component]
pub fn Canvas() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut is_painting = use_signal(|| false);

    // Reactive redraw: runs whenever app_state changes
    use_effect(move || {
        let state = app_state.read();
        draw_project(&state.project);
    });

    rsx! {
        div {
            class: "flex-1 overflow-auto bg-[#19181a] flex items-start justify-start p-2",
            canvas {
                id: CANVAS_ID,
                style: "cursor: crosshair; image-rendering: pixelated;",
                onmousedown: move |evt| {
                    is_painting.set(true);
                    let coords = evt.element_coordinates();
                    let (col, row) = coords_to_cell(coords.x, coords.y);
                    app_state.with_mut(|s| apply_tool(s, col, row));
                },
                onmousemove: move |evt| {
                    if *is_painting.read() {
                        let coords = evt.element_coordinates();
                        let (col, row) = coords_to_cell(coords.x, coords.y);
                        app_state.with_mut(|s| apply_tool(s, col, row));
                    }
                },
                onmouseup: move |_| is_painting.set(false),
                onmouseleave: move |_| is_painting.set(false),
            }
        }
    }
}
```

- [ ] **Step 2: Add `mod canvas;` to `src/main.rs` and use Canvas in App**

```rust
#![allow(non_snake_case)]
mod canvas;
mod color;
mod components;
mod export;
mod import;
mod state;

use canvas::Canvas;
use dioxus::prelude::*;
use state::AppState;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        document::Stylesheet { href: asset!("/tailwind.css") }
        div {
            class: "flex flex-col h-screen bg-[#2d2a2e] text-[#fcfcfa]",
            // TopBar placeholder
            div { class: "h-10 bg-[#221f22] border-b border-[#403e41] flex items-center px-3",
                span { class: "text-sm font-bold tracking-widest", "GRITTY" }
            }
            // Main area
            div { class: "flex flex-1 overflow-hidden",
                // LeftPanel placeholder
                div { class: "w-14 bg-[#221f22] border-r border-[#403e41]" }
                // Canvas
                Canvas {}
                // RightPanel placeholder
                div { class: "w-16 bg-[#221f22] border-l border-[#403e41]" }
            }
            // Timeline placeholder
            div { class: "h-12 bg-[#221f22] border-t border-[#403e41]" }
        }
    }
}
```

- [ ] **Step 3: Create stub `src/components/mod.rs`**

```rust
pub mod left_panel;
pub mod right_panel;
pub mod timeline;
pub mod top_bar;
```

- [ ] **Step 4: Create stub files for components**

Create `src/components/left_panel.rs`:
```rust
use dioxus::prelude::*;
#[component]
pub fn LeftPanel() -> Element { rsx! { div {} } }
```

Create `src/components/right_panel.rs`:
```rust
use dioxus::prelude::*;
#[component]
pub fn RightPanel() -> Element { rsx! { div {} } }
```

Create `src/components/timeline.rs`:
```rust
use dioxus::prelude::*;
#[component]
pub fn Timeline() -> Element { rsx! { div {} } }
```

Create `src/components/top_bar.rs`:
```rust
use dioxus::prelude::*;
#[component]
pub fn TopBar() -> Element { rsx! { div {} } }
```

- [ ] **Step 5: Serve and verify canvas paints**

```bash
dx serve --platform web
```

Open browser. You should see a dark layout with a canvas in the center. Click and drag — cells should fill with the default pink color and `█` glyph. Stop with Ctrl+C.

- [ ] **Step 6: Commit**

```bash
git add src/canvas.rs src/components/ src/main.rs
git commit -m "feat: add Canvas component with web-sys drawing and mouse paint"
```

---

## Task 7: Left Panel

**Files:**
- Modify: `src/components/left_panel.rs`
- Modify: `src/main.rs` (replace LeftPanel placeholder)

- [ ] **Step 1: Define glyph categories (add to `src/state.rs`)**

Add this constant at the bottom of `state.rs`:
```rust
pub const GLYPH_GROUPS: &[(&str, &[char])] = &[
    ("Full",    &['█']),
    ("H-Half",  &['▀', '▄']),
    ("V-Half",  &['▌', '▐']),
    ("Shade",   &['░', '▒', '▓']),
    ("Quad",    &['▖', '▗', '▘', '▙', '▚', '▛', '▜', '▝', '▞', '▟']),
    ("Space",   &[' ']),
];
```

- [ ] **Step 2: Write `src/components/left_panel.rs`**

```rust
use dioxus::prelude::*;
use crate::state::{AppState, Tool, GLYPH_GROUPS};

#[component]
pub fn LeftPanel() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    rsx! {
        div {
            class: "flex flex-col gap-2 p-2 bg-[#221f22] border-r border-[#403e41] w-14 overflow-y-auto",

            // Tool selector
            div { class: "flex flex-col gap-1",
                span { class: "text-[9px] text-[#9ca0a4] tracking-widest uppercase", "Tool" }
                button {
                    class: {
                        let is_brush = app_state.read().tool == Tool::Brush;
                        if is_brush {
                            "w-full h-7 rounded text-xs font-bold bg-[#ff6188] text-[#2d2a2e]"
                        } else {
                            "w-full h-7 rounded text-xs bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]"
                        }
                    },
                    onclick: move |_| app_state.with_mut(|s| s.tool = Tool::Brush),
                    "✏"
                }
                button {
                    class: {
                        let is_eraser = app_state.read().tool == Tool::Eraser;
                        if is_eraser {
                            "w-full h-7 rounded text-xs font-bold bg-[#78dce8] text-[#2d2a2e]"
                        } else {
                            "w-full h-7 rounded text-xs bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]"
                        }
                    },
                    onclick: move |_| app_state.with_mut(|s| s.tool = Tool::Eraser),
                    "◻"
                }
            }

            // Divider
            div { class: "border-t border-[#403e41]" }

            // Glyph grid
            div { class: "flex flex-col gap-1",
                span { class: "text-[9px] text-[#9ca0a4] tracking-widest uppercase", "Glyph" }
                for (label, glyphs) in GLYPH_GROUPS.iter() {
                    div { class: "flex flex-col gap-0.5",
                        span { class: "text-[8px] text-[#5b595c]", "{label}" }
                        div { class: "flex flex-wrap gap-0.5",
                            for ch in glyphs.iter() {
                                {
                                    let ch = *ch;
                                    let is_active = app_state.read().active_glyph == ch;
                                    rsx! {
                                        button {
                                            class: if is_active {
                                                "w-6 h-6 text-sm rounded bg-[#ffd866] text-[#2d2a2e] font-bold"
                                            } else {
                                                "w-6 h-6 text-sm rounded bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]"
                                            },
                                            onclick: move |_| app_state.with_mut(|s| s.active_glyph = ch),
                                            "{ch}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 3: Wire LeftPanel into App in `src/main.rs`**

Replace the LeftPanel placeholder div:
```rust
use components::left_panel::LeftPanel;
// ...
// Inside the main area div, replace the placeholder:
LeftPanel {}
```

- [ ] **Step 4: Serve and verify**

```bash
dx serve --platform web
```

Left panel shows Tool buttons and glyph grid. Clicking a glyph highlights it in yellow. Switching tools changes the button highlight. Painting on the canvas uses the selected glyph.

- [ ] **Step 5: Commit**

```bash
git add src/components/left_panel.rs src/main.rs src/state.rs
git commit -m "feat: add LeftPanel with tool selector and categorized glyph grid"
```

---

## Task 8: Right Panel — Color Wheel

**Files:**
- Modify: `src/components/right_panel.rs`
- Modify: `src/main.rs` (wire RightPanel)

- [ ] **Step 1: Write `src/components/right_panel.rs`**

```rust
use dioxus::prelude::*;
use js_sys::Uint8ClampedArray;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use crate::color::{hsv_to_rgb, parse_hex, rgb_to_hsv, to_hex};
use crate::state::{AppState, ColorTarget};

const WHEEL_ID: &str = "color-wheel";
const WHEEL_SIZE: u32 = 120;
const RING_W: f64 = 16.0;

fn get_wheel_ctx() -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let canvas = web_sys::window()?
        .document()?
        .get_element_by_id(WHEEL_ID)?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas
        .get_context("2d").ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    Some((canvas, ctx))
}

fn draw_wheel(hue: f64, sat: f64, val: f64) {
    let Some((canvas, ctx)) = get_wheel_ctx() else { return };
    canvas.set_width(WHEEL_SIZE);
    canvas.set_height(WHEEL_SIZE);

    let w = WHEEL_SIZE;
    let h = WHEEL_SIZE;
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let outer_r = cx - 2.0;
    let inner_r = outer_r - RING_W;
    let sq_half = inner_r * 0.68; // largest square fitting the inner circle

    let mut pixels = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let x = col as f64 - cx;
            let y = row as f64 - cy;
            let dist = (x * x + y * y).sqrt();
            let idx = ((row * w + col) * 4) as usize;

            if dist >= inner_r && dist <= outer_r {
                // Hue ring
                let angle = y.atan2(x).to_degrees();
                let h_deg = (angle + 360.0) % 360.0;
                let [r, g, b] = hsv_to_rgb(h_deg, 1.0, 1.0);
                pixels[idx]     = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            } else if x.abs() <= sq_half && y.abs() <= sq_half {
                // Saturation/Value square: s on x-axis, v on y-axis (flipped)
                let s = ((x + sq_half) / (2.0 * sq_half)).clamp(0.0, 1.0);
                let v = ((sq_half - y) / (2.0 * sq_half)).clamp(0.0, 1.0);
                let [r, g, b] = hsv_to_rgb(hue, s, v);
                pixels[idx]     = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            }
            // else: transparent (stays 0,0,0,0)
        }
    }

    let clamped = Uint8ClampedArray::from(pixels.as_slice());
    if let Ok(image_data) = ImageData::new_with_js_uint8_clamped_array_and_sw(&clamped, w) {
        let _ = ctx.put_image_data(&image_data, 0.0, 0.0);
    }

    // Draw hue indicator on ring
    let hue_rad = hue.to_radians();
    let ind_r = (inner_r + outer_r) / 2.0;
    let ix = cx + ind_r * hue_rad.cos();
    let iy = cy + ind_r * hue_rad.sin();
    ctx.begin_path();
    let _ = ctx.arc(ix, iy, 4.0, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style_str("#fcfcfa");
    ctx.set_line_width(1.5);
    ctx.stroke();

    // Draw SV indicator
    let sx = cx - sq_half + sat * 2.0 * sq_half;
    let sy = cy + sq_half - val * 2.0 * sq_half;
    ctx.begin_path();
    let _ = ctx.arc(sx, sy, 4.0, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style_str("#fcfcfa");
    ctx.stroke();
}

#[component]
pub fn RightPanel() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    // Reactive redraw of color wheel
    use_effect(move || {
        let state = app_state.read();
        let active_color = match state.color_target {
            ColorTarget::Fg => state.fg_color,
            ColorTarget::Bg => state.bg_color,
        };
        let (h, s, v) = rgb_to_hsv(active_color);
        draw_wheel(h, s, v);
    });

    let active_color = {
        let s = app_state.read();
        match s.color_target { ColorTarget::Fg => s.fg_color, ColorTarget::Bg => s.bg_color }
    };
    let hex_value = to_hex(active_color);
    let fg_hex = to_hex(app_state.read().fg_color);
    let bg_hex = to_hex(app_state.read().bg_color);
    let color_target = app_state.read().color_target.clone();

    rsx! {
        div {
            class: "flex flex-col items-center gap-2 p-2 bg-[#221f22] border-l border-[#403e41] w-[136px] overflow-y-auto",

            span { class: "text-[9px] text-[#9ca0a4] tracking-widest uppercase self-start", "Color" }

            // Color wheel canvas
            canvas {
                id: WHEEL_ID,
                width: WHEEL_SIZE,
                height: WHEEL_SIZE,
                style: "cursor: crosshair; border-radius: 50%;",
                onmousedown: move |evt| {
                    let coords = evt.element_coordinates();
                    let cx = WHEEL_SIZE as f64 / 2.0;
                    let cy = WHEEL_SIZE as f64 / 2.0;
                    let dx = coords.x - cx;
                    let dy = coords.y - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let outer_r = cx - 2.0;
                    let inner_r = outer_r - RING_W;
                    let sq_half = inner_r * 0.68;

                    app_state.with_mut(|s| {
                        let active_color = match s.color_target {
                            ColorTarget::Fg => s.fg_color,
                            ColorTarget::Bg => s.bg_color,
                        };
                        let (cur_h, cur_s, cur_v) = rgb_to_hsv(active_color);

                        let new_rgb = if dist >= inner_r && dist <= outer_r {
                            // Clicked hue ring
                            let angle = dy.atan2(dx).to_degrees();
                            let new_h = (angle + 360.0) % 360.0;
                            hsv_to_rgb(new_h, cur_s.max(0.1), cur_v.max(0.1))
                        } else if dx.abs() <= sq_half && dy.abs() <= sq_half {
                            // Clicked SV square
                            let new_s = ((dx + sq_half) / (2.0 * sq_half)).clamp(0.0, 1.0);
                            let new_v = ((sq_half - dy) / (2.0 * sq_half)).clamp(0.0, 1.0);
                            hsv_to_rgb(cur_h, new_s, new_v)
                        } else {
                            return;
                        };
                        match s.color_target {
                            ColorTarget::Fg => s.fg_color = new_rgb,
                            ColorTarget::Bg => s.bg_color = new_rgb,
                        }
                    });
                },
            }

            // Hex input
            input {
                r#type: "text",
                value: "{hex_value}",
                maxlength: 7,
                class: "w-full bg-[#403e41] text-[#fcfcfa] text-xs text-center rounded px-1 py-0.5 font-mono border border-[#5b595c] focus:outline-none focus:border-[#ff6188]",
                onchange: move |evt| {
                    if let Some(rgb) = parse_hex(&evt.value()) {
                        app_state.with_mut(|s| {
                            match s.color_target {
                                ColorTarget::Fg => s.fg_color = rgb,
                                ColorTarget::Bg => s.bg_color = rgb,
                            }
                        });
                    }
                },
            }

            // FG / BG swatches
            div { class: "flex flex-col gap-1 w-full",
                span { class: "text-[9px] text-[#9ca0a4] tracking-widest uppercase", "FG / BG" }
                div { class: "flex gap-1",
                    button {
                        class: if matches!(color_target, ColorTarget::Fg) {
                            "flex-1 h-7 rounded border-2 border-[#ffd866]"
                        } else {
                            "flex-1 h-7 rounded border-2 border-transparent hover:border-[#5b595c]"
                        },
                        style: "background-color: {fg_hex};",
                        onclick: move |_| app_state.with_mut(|s| s.color_target = ColorTarget::Fg),
                        title: "Foreground"
                    }
                    button {
                        class: if matches!(color_target, ColorTarget::Bg) {
                            "flex-1 h-7 rounded border-2 border-[#ffd866]"
                        } else {
                            "flex-1 h-7 rounded border-2 border-transparent hover:border-[#5b595c]"
                        },
                        style: "background-color: {bg_hex};",
                        onclick: move |_| app_state.with_mut(|s| s.color_target = ColorTarget::Bg),
                        title: "Background"
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Wire RightPanel into App in `src/main.rs`**

Replace the RightPanel placeholder div:
```rust
use components::right_panel::RightPanel;
// ...
RightPanel {}
```

- [ ] **Step 3: Serve and verify**

```bash
dx serve --platform web
```

Right panel shows HSV color wheel. Click the outer hue ring to change hue. Click the inner square to change saturation/value. Hex input updates to reflect selected color. FG/BG swatches toggle which color is being edited. Painting on canvas uses the selected color.

- [ ] **Step 4: Commit**

```bash
git add src/components/right_panel.rs src/main.rs
git commit -m "feat: add RightPanel with HSV color wheel, hex input, FG/BG swatches"
```

---

## Task 9: Top Bar

**Files:**
- Modify: `src/components/top_bar.rs`
- Modify: `src/main.rs` (wire TopBar)

- [ ] **Step 1: Write `src/components/top_bar.rs`**

```rust
use dioxus::prelude::*;
use js_sys::Array;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlInputElement, Url};

use crate::export::{export_ansi_all, export_ansi_frame, export_json};
use crate::import::{import_ansi, import_json};
use crate::state::AppState;

#[component]
pub fn TopBar() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut export_open = use_signal(|| false);
    let mut export_tab = use_signal(|| 0usize); // 0=ANSI, 1=PlainANSI, 2=JSON
    let mut all_frames = use_signal(|| false);

    let (w, h) = {
        let s = app_state.read();
        (s.project.width, s.project.height)
    };

    // Build export content string reactively
    let export_text = {
        let s = app_state.read();
        let tab = *export_tab.read();
        let all = *all_frames.read();
        match tab {
            0 | 1 => {
                if all { export_ansi_all(&s.project) }
                else { export_ansi_frame(&s.project.frames[s.project.active_frame], s.project.width, s.project.height) }
            }
            _ => String::new() // JSON is downloaded, not shown
        }
    };

    rsx! {
        div {
            class: "h-10 bg-[#221f22] border-b border-[#403e41] flex items-center px-3 gap-4 shrink-0",

            // Logo
            span { class: "text-sm font-bold tracking-widest text-[#fcfcfa]", "GRITTY" }

            // Canvas size
            span {
                class: "text-xs text-[#9ca0a4] font-mono",
                "{w}×{h}"
            }

            div { class: "flex-1" }

            // Import button
            label {
                class: "text-xs bg-[#403e41] hover:bg-[#5b595c] text-[#a9dc76] px-2 py-1 rounded cursor-pointer",
                r#for: "file-import",
                "Import"
            }
            input {
                id: "file-import",
                r#type: "file",
                accept: ".json,.ans,.txt",
                class: "hidden",
                onchange: move |_| {
                    // Read the file via web-sys
                    let document = web_sys::window().unwrap().document().unwrap();
                    let input = document.get_element_by_id("file-import").unwrap()
                        .dyn_into::<HtmlInputElement>().unwrap();
                    if let Some(files) = input.files() {
                        if let Some(file) = files.item(0) {
                            let name = file.name();
                            let is_json = name.ends_with(".json");
                            let w = app_state.read().project.width;
                            let h = app_state.read().project.height;
                            let text_promise = file.text();
                            let mut state_ref = app_state;
                            wasm_bindgen_futures::spawn_local(async move {
                                if let Ok(text_js) = wasm_bindgen_futures::JsFuture::from(text_promise).await {
                                    if let Some(text) = text_js.as_string() {
                                        if is_json {
                                            if let Ok(project) = import_json(&text) {
                                                state_ref.with_mut(|s| s.project = project);
                                            }
                                        } else {
                                            let project = import_ansi(&text, w, h);
                                            state_ref.with_mut(|s| s.project = project);
                                        }
                                    }
                                }
                            });
                        }
                    }
                },
            }

            // Export button
            button {
                class: "text-xs bg-[#ff6188] hover:bg-[#e05078] text-[#2d2a2e] font-bold px-2 py-1 rounded",
                onclick: move |_| export_open.set(true),
                "Export"
            }

            // Export modal
            if *export_open.read() {
                div {
                    class: "fixed inset-0 bg-black bg-opacity-60 flex items-center justify-center z-50",
                    onclick: move |_| export_open.set(false),
                    div {
                        class: "bg-[#2d2a2e] border border-[#403e41] rounded-lg w-[560px] max-h-[80vh] flex flex-col overflow-hidden",
                        onclick: move |evt| evt.stop_propagation(),

                        // Modal header + tabs
                        div { class: "flex items-center border-b border-[#403e41]",
                            for (i, label) in ["ANSI", "Plain+ANSI", "JSON"].iter().enumerate() {
                                button {
                                    class: if *export_tab.read() == i {
                                        "px-4 py-2 text-xs font-bold text-[#ff6188] border-b-2 border-[#ff6188]"
                                    } else {
                                        "px-4 py-2 text-xs text-[#9ca0a4] hover:text-[#fcfcfa]"
                                    },
                                    onclick: move |_| export_tab.set(i),
                                    "{label}"
                                }
                            }
                            div { class: "flex-1" }
                            button {
                                class: "px-3 py-2 text-[#9ca0a4] hover:text-[#fcfcfa] text-sm",
                                onclick: move |_| export_open.set(false),
                                "✕"
                            }
                        }

                        // Scope toggle (ANSI tabs only)
                        if *export_tab.read() < 2 {
                            div { class: "flex items-center gap-2 px-4 py-2 border-b border-[#403e41]",
                                span { class: "text-xs text-[#9ca0a4]", "Scope:" }
                                button {
                                    class: if !*all_frames.read() {
                                        "text-xs px-2 py-0.5 rounded bg-[#ff6188] text-[#2d2a2e] font-bold"
                                    } else {
                                        "text-xs px-2 py-0.5 rounded bg-[#403e41] text-[#fcfcfa]"
                                    },
                                    onclick: move |_| all_frames.set(false),
                                    "Current frame"
                                }
                                button {
                                    class: if *all_frames.read() {
                                        "text-xs px-2 py-0.5 rounded bg-[#ff6188] text-[#2d2a2e] font-bold"
                                    } else {
                                        "text-xs px-2 py-0.5 rounded bg-[#403e41] text-[#fcfcfa]"
                                    },
                                    onclick: move |_| all_frames.set(true),
                                    "All frames"
                                }
                            }
                        }

                        // Content area
                        if *export_tab.read() < 2 {
                            // Text copy area
                            div { class: "flex flex-col flex-1 overflow-hidden p-3 gap-2",
                                pre {
                                    class: "flex-1 overflow-auto text-xs font-mono text-[#fcfcfa] bg-[#19181a] p-3 rounded border border-[#403e41] whitespace-pre",
                                    "{export_text}"
                                }
                                button {
                                    class: "self-end text-xs bg-[#403e41] hover:bg-[#5b595c] text-[#78dce8] px-3 py-1 rounded",
                                    onclick: {
                                        let text = export_text.clone();
                                        move |_| {
                                            if let Some(clipboard) = web_sys::window()
                                                .and_then(|w| w.navigator().clipboard())
                                            {
                                                let _ = clipboard.write_text(&text);
                                            }
                                        }
                                    },
                                    "Copy"
                                }
                            }
                        } else {
                            // JSON download
                            div { class: "flex flex-col items-center justify-center flex-1 gap-4 p-6",
                                p { class: "text-sm text-[#9ca0a4]", "Downloads the full project (all frames) as .json" }
                                button {
                                    class: "bg-[#ff6188] text-[#2d2a2e] font-bold px-6 py-2 rounded hover:bg-[#e05078]",
                                    onclick: move |_| {
                                        let json = export_json(&app_state.read().project);
                                        download_string(&json, "gritty-project.json", "application/json");
                                    },
                                    "Download .json"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn download_string(content: &str, filename: &str, mime: &str) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let array = Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(content));
    let mut opts = BlobPropertyBag::new();
    opts.type_(mime);
    let Ok(blob) = Blob::new_with_str_sequence_and_options(&array, &opts) else { return };
    let Ok(url) = Url::create_object_url_with_blob(&blob) else { return };
    if let Ok(a) = document.create_element("a").and_then(|el| el.dyn_into::<HtmlAnchorElement>()) {
        a.set_href(&url);
        a.set_download(filename);
        if let Some(body) = document.body() {
            let _ = body.append_child(&a);
            a.click();
            let _ = body.remove_child(&a);
        }
    }
    let _ = Url::revoke_object_url(&url);
}
```

- [ ] **Step 2: Wire TopBar into App in `src/main.rs`**

Replace the TopBar placeholder div:
```rust
use components::top_bar::TopBar;
// ...
TopBar {}
```

- [ ] **Step 3: Serve and verify**

```bash
dx serve --platform web
```

Top bar shows "GRITTY" logo and canvas dimensions. Clicking Export opens the modal with ANSI/JSON tabs. Copy button copies the ANSI text. JSON download button works. Import label opens a file picker; selecting a `.json` file loads the project.

- [ ] **Step 4: Commit**

```bash
git add src/components/top_bar.rs src/main.rs
git commit -m "feat: add TopBar with export modal (ANSI/JSON) and import file picker"
```

---

## Task 10: Timeline

**Files:**
- Modify: `src/components/timeline.rs`
- Modify: `src/main.rs` (wire Timeline)

- [ ] **Step 1: Write `src/components/timeline.rs`**

```rust
use dioxus::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::state::AppState;

#[component]
pub fn Timeline() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut interval_id: Signal<Option<i32>> = use_signal(|| None);

    let (frame_count, active_frame, delay_ms, playing) = {
        let s = app_state.read();
        (s.project.frames.len(), s.project.active_frame, s.playback.delay_ms, s.playback.playing)
    };

    rsx! {
        div {
            class: "h-12 bg-[#221f22] border-t border-[#403e41] flex items-center px-3 gap-2 shrink-0 overflow-x-auto",

            span { class: "text-[9px] text-[#9ca0a4] tracking-widest uppercase shrink-0", "FRAMES" }

            // Frame thumbnails
            for i in 0..frame_count {
                button {
                    class: if i == active_frame {
                        "w-8 h-8 rounded text-xs font-bold bg-[#ff6188] text-[#2d2a2e] shrink-0"
                    } else {
                        "w-8 h-8 rounded text-xs bg-[#403e41] text-[#9ca0a4] hover:bg-[#5b595c] shrink-0"
                    },
                    onclick: move |_| app_state.with_mut(|s| s.project.active_frame = i),
                    "{i + 1}"
                }
            }

            // Frame controls
            div { class: "flex gap-1 shrink-0",
                // Add frame
                button {
                    class: "w-7 h-7 rounded bg-[#403e41] text-[#a9dc76] hover:bg-[#5b595c] text-sm font-bold",
                    title: "Add frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        s.project.add_frame();
                        s.project.active_frame = s.project.frames.len() - 1;
                    }),
                    "+"
                }
                // Duplicate frame
                button {
                    class: "w-7 h-7 rounded bg-[#403e41] text-[#78dce8] hover:bg-[#5b595c] text-xs",
                    title: "Duplicate frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.duplicate_frame(idx);
                        s.project.active_frame = idx + 1;
                    }),
                    "⧉"
                }
                // Move frame left
                button {
                    class: "w-7 h-7 rounded bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c] text-xs",
                    title: "Move frame left",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_up(idx);
                    }),
                    "←"
                }
                // Move frame right
                button {
                    class: "w-7 h-7 rounded bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c] text-xs",
                    title: "Move frame right",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_down(idx);
                    }),
                    "→"
                }
                // Delete frame
                button {
                    class: "w-7 h-7 rounded bg-[#403e41] text-[#ff6188] hover:bg-[#5b595c] text-xs font-bold",
                    title: "Delete frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.delete_frame(idx);
                    }),
                    "✕"
                }
            }

            div { class: "flex-1" }

            // Playback controls
            div { class: "flex items-center gap-2 shrink-0",
                // Delay input
                input {
                    r#type: "number",
                    value: "{delay_ms}",
                    min: "50",
                    max: "5000",
                    step: "50",
                    class: "w-16 bg-[#403e41] text-[#fcfcfa] text-xs text-center rounded px-1 py-0.5 font-mono border border-[#5b595c] focus:outline-none focus:border-[#78dce8]",
                    title: "Frame delay (ms)",
                    onchange: move |evt| {
                        if let Ok(ms) = evt.value().parse::<u32>() {
                            app_state.with_mut(|s| s.playback.delay_ms = ms.max(50));
                        }
                    },
                }
                span { class: "text-[9px] text-[#9ca0a4]", "ms" }

                // Play/Stop button
                button {
                    class: if playing {
                        "px-3 h-7 rounded bg-[#fc9867] text-[#2d2a2e] text-xs font-bold hover:bg-[#e08856] shrink-0"
                    } else {
                        "px-3 h-7 rounded bg-[#403e41] text-[#78dce8] text-xs font-bold hover:bg-[#5b595c] shrink-0"
                    },
                    onclick: move |_| {
                        let currently_playing = app_state.read().playback.playing;
                        if currently_playing {
                            // Stop
                            app_state.with_mut(|s| s.playback.playing = false);
                            if let Some(id) = *interval_id.read() {
                                if let Some(window) = web_sys::window() {
                                    window.clear_interval_with_handle(id);
                                }
                            }
                            interval_id.set(None);
                        } else {
                            // Start
                            app_state.with_mut(|s| s.playback.playing = true);
                            let delay = app_state.read().playback.delay_ms as i32;
                            let mut state_ref = app_state;
                            let cb = Closure::wrap(Box::new(move || {
                                let mut s = state_ref.write();
                                if !s.playback.playing { return; }
                                let nframes = s.project.frames.len();
                                s.project.active_frame = (s.project.active_frame + 1) % nframes;
                            }) as Box<dyn FnMut()>);
                            if let Some(window) = web_sys::window() {
                                if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                                    cb.as_ref().unchecked_ref(),
                                    delay,
                                ) {
                                    interval_id.set(Some(id));
                                }
                            }
                            cb.forget(); // JS owns the closure
                        }
                    },
                    if playing { "■ Stop" } else { "▶ Play" }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Wire Timeline into App in `src/main.rs`**

Replace the Timeline placeholder div:
```rust
use components::timeline::Timeline;
// ...
Timeline {}
```

- [ ] **Step 3: Serve and verify**

```bash
dx serve --platform web
```

Timeline shows Frame 1 button highlighted. "+" adds a frame. "⧉" duplicates. "✕" deletes (can't delete last). "←"/"→" reorder. Click Play — canvas advances through frames at the set delay. Click Stop to pause. Delay input changes playback speed.

- [ ] **Step 4: Commit**

```bash
git add src/components/timeline.rs src/main.rs
git commit -m "feat: add Timeline with frame management and interval-based playback"
```

---

## Task 11: Final Wiring and `src/main.rs` Cleanup

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write final `src/main.rs`**

```rust
#![allow(non_snake_case)]
mod canvas;
mod color;
mod components;
mod export;
mod import;
mod state;

use canvas::Canvas;
use components::{
    left_panel::LeftPanel,
    right_panel::RightPanel,
    timeline::Timeline,
    top_bar::TopBar,
};
use dioxus::prelude::*;
use state::AppState;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        document::Stylesheet { href: asset!("/tailwind.css") }
        div {
            class: "flex flex-col h-screen bg-[#2d2a2e] text-[#fcfcfa] select-none",
            TopBar {}
            div {
                class: "flex flex-1 overflow-hidden",
                LeftPanel {}
                Canvas {}
                RightPanel {}
            }
            Timeline {}
        }
    }
}
```

- [ ] **Step 2: Run all tests**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 3: Serve and do a full feature walkthrough**

```bash
dx serve --platform web
```

Walk through:
1. Paint cells with different glyphs from the left panel
2. Switch between Brush and Eraser tools
3. Change FG color via the color wheel, paint, then change BG color
4. Edit hex input to set a specific color
5. Add a second frame, paint something different, add a third
6. Reorder frames with ← → buttons
7. Play animation — confirm frames advance in the canvas
8. Export ANSI (current frame) — copy the text
9. Paste it in a terminal: `printf '<pasted text>'` — confirm it renders
10. Export JSON — confirm the file downloads
11. Import the JSON back — confirm the project restores correctly

- [ ] **Step 4: Rebuild Tailwind to pick up all classes**

```bash
./tailwindcss -i input.css -o assets/tailwind.css
```

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire all components into App layout — MVP complete"
```

---

## Task 12: Release Build Verification

**Files:** None new.

- [ ] **Step 1: Release build**

```bash
dx build --platform web --release
```

Expected: `dist/` created with no errors.

- [ ] **Step 2: Smoke test the dist**

```bash
npx serve dist
```

Open `http://localhost:3000`. Verify full functionality matches dev build.

- [ ] **Step 3: Commit dist config if needed**

If any path issues arise in the release build, fix `Dioxus.toml` asset paths, then:
```bash
git add Dioxus.toml
git commit -m "fix: correct asset paths for release build"
```

---

## Self-Review Checklist

- [x] Spec §2 Data Model → Task 2 (`state.rs`)
- [x] Spec §3 Glyph Palette → Task 7 (`GLYPH_GROUPS` constant + `LeftPanel`)
- [x] Spec §4 Canvas Rendering (8×16px, use_effect, mouse) → Task 6 (`canvas.rs`)
- [x] Spec §5 TopBar → Task 9
- [x] Spec §5 LeftPanel → Task 7
- [x] Spec §5 RightPanel (HSV wheel, hex, swatches) → Task 8
- [x] Spec §5 Timeline (add/dup/del/reorder/play) → Task 10
- [x] Spec §6 Export (ANSI single/all, JSON download) → Task 4 + Task 9
- [x] Spec §6 Import (JSON, ANSI multi-frame) → Task 5 + Task 9
- [x] Spec §7 File Structure → matches exactly
- [x] Spec §8 Dependencies → all in Task 1 `Cargo.toml`
- [x] Monokai Pro theme → Task 1 `input.css` CSS vars used throughout all components
- [x] Terminal cell ratio (8×16) → `CELL_W`/`CELL_H` constants in `state.rs`, used in `canvas.rs`
- [x] `ColorTarget` in AppState → Task 2 + Task 8
