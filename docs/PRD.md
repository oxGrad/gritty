# PRD — Gritty (ANSI Block Art Editor, WebAssembly)

## 1. Overview

Gritty is a browser-based ANSI block art editor built with Rust and WebAssembly. It focuses on creating colored block-based terminal art using a structured grid model that maps directly to ANSI-renderable output.

The product targets developers, terminal enthusiasts, and creators who want to design ANSI visuals and lightweight animations directly in the browser, with export compatibility for real terminal environments.

---

## 2. Goals

### Primary Goals
- Provide an intuitive editor for ANSI block art using a cell-based grid
- Support multiple block glyph types (full, half, etc.)
- Enable frame-based animation
- Allow import/export of ANSI-compatible formats

### Non-Goals (v1)
- No collaborative editing
- No GPU-heavy rendering (no wgpu initially)
- No advanced layering system

---

## 3. Target Users

- CLI/TUI developers
- DevOps engineers and tool builders
- Terminal art hobbyists
- Open-source maintainers building branded CLIs

---

## 4. Core Features

### 4.1 Canvas Configuration
- User can define:
  - Width (columns)
  - Height (rows)
- Grid is fixed-size after creation (resize handled later)

---

### 4.2 Block Drawing System

Each cell consists of:
- Foreground color
- Background color
- Character (glyph)

#### Supported Block Types (v1)
- Full block (█)
- Half block bottom (▄)
- Half block top (▀)
- Space ( )

Future:
- Quarter blocks
- Braille patterns

---

### 4.3 Color System

- Truecolor (RGB)
- Optional constrained palettes:
  - ANSI 16-color
  - ANSI 256-color

---

### 4.4 Drawing Tools

- Brush (click + drag)
- Eraser (resets cell)
- Fill tool (optional v1.1)

---

### 4.5 Frame-Based Animation

- Multiple frames per project
- Each frame = full grid state

#### Frame Controls
- Add frame
- Duplicate frame
- Delete frame
- Reorder frames

#### Playback (basic)
- Preview animation loop
- Adjustable frame delay (ms)

---

### 4.6 Import / Export

#### Export Formats
- ANSI escape string
- Plain text with embedded ANSI codes
- JSON (project format)

#### Import
- JSON project
- ANSI text (best-effort parsing)

---

## 5. Technical Architecture

### 5.1 Stack

- Language: Rust
- UI Framework: Dioxus
- Styling: TailwindCSS (via WASM-compatible setup)
- Rendering: HTML Canvas via web-sys
- WASM Binding: wasm-bindgen

---

### 5.2 Rendering Model

- Single <canvas> element
- Each cell rendered as:
  - Background rectangle
  - Foreground glyph (text draw)

#### Rendering Loop
- Triggered on:
  - State updates
  - Frame change
  - Canvas resize

---

### 5.3 Data Model

#### Cell
- fg: RGB
- bg: RGB
- ch: char

#### Frame
- cells: Vec<Cell>

#### Project
- width: u32
- height: u32
- frames: Vec<Frame>
- active_frame_index: usize

---

### 5.4 State Management

Handled via Dioxus signals/hooks:
- Global project state
- Tool state (selected glyph, color)
- UI state (selected frame, playback)

---

## 6. UI / UX

### 6.1 Layout

- Top bar:
  - Project settings (size)
  - Export / Import
- Left panel:
  - Tools (brush, eraser)
  - Block selector
- Right panel:
  - Color picker (fg/bg)
- Center:
  - Canvas
- Bottom:
  - Timeline (frames)

---

### 6.2 Interaction Model

- Click: paint cell
- Drag: continuous paint
- Modifier keys:
  - Shift: straight line (future)
  - Alt: color pick (future)

---

## 7. Performance Considerations

- Avoid DOM-per-cell rendering
- Batch canvas redraws
- Minimize full redraw (dirty regions later optimization)
- Target grid sizes:
  - Recommended: up to 120x60

---

## 8. Milestones

### MVP (v1)
- Canvas setup
- Block drawing
- Color selection
- Frame system
- Export ANSI string
- Import/export JSON

### v1.1
- Flood fill
- Undo/redo
- Better ANSI parsing

### v1.2
- Animation export (scripted playback)
- Palette presets

---

## 9. Risks

- ANSI export inconsistencies across terminals
- Performance degradation on large grids
- Complexity of accurate ANSI import parsing

---

## 10. Success Criteria

- Users can create ANSI art and render it correctly in a terminal
- Frame animation works reliably in preview
- Exported output is usable in CLI tools

---

## 11. Future Expansion

- CLI companion tool (render/export)
- GitHub integration (share art)
- Terminal preview emulator
- Plugin system for custom glyph sets

---

## 12. Positioning

Gritty is positioned as:
> A developer-first ANSI art tool that treats terminal visuals as structured data, not just pixels.

It bridges:
- Creative tooling
- Terminal-native rendering
- Developer workflows

---
