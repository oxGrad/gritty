use dioxus::prelude::*;
use dioxus_elements::geometry::WheelDelta;
use dioxus_elements::input_data::MouseButton;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

use crate::color::to_hex;
use crate::state::{AppState, Tool};

pub const CANVAS_ID: &str = "main-canvas";

fn get_canvas_ctx() -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let canvas = web_sys::window()?
        .document()?
        .get_element_by_id(CANVAS_ID)?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    let ctx = canvas.get_context("2d").ok()??.dyn_into::<CanvasRenderingContext2d>().ok()?;
    Some((canvas, ctx))
}

fn draw_project(state: &AppState) {
    let project = &state.project;
    let cell = state.cell_size;
    let Some((canvas, ctx)) = get_canvas_ctx() else { return };

    let px_w = (project.width as f64 * cell) as u32;
    let px_h = (project.height as f64 * cell) as u32;
    let dpr = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0);
    canvas.set_width((px_w as f64 * dpr) as u32);
    canvas.set_height((px_h as f64 * dpr) as u32);
    let _ = ctx.scale(dpr, dpr);

    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", cell as u32));
    ctx.set_text_baseline("top");
    ctx.clear_rect(0.0, 0.0, px_w as f64, px_h as f64);

    let frame = &project.frames[project.active_frame];

    // Onion skin (previous frame, faded)
    if state.onion_skin && project.active_frame > 0 {
        let prev = &project.frames[project.active_frame - 1];
        ctx.set_global_alpha(0.22);
        for row in 0..project.height {
            for col in 0..project.width {
                let c = &prev.cells[(row * project.width + col) as usize];
                if c.ch != ' ' {
                    let x = col as f64 * cell;
                    let y = row as f64 * cell;
                    ctx.set_fill_style_str(&format!("rgb({},{},{})", c.bg[0], c.bg[1], c.bg[2]));
                    ctx.fill_rect(x, y, cell, cell);
                    ctx.set_fill_style_str(&format!("rgb({},{},{})", c.fg[0], c.fg[1], c.fg[2]));
                    let _ = ctx.fill_text(&c.ch.to_string(), x, y);
                }
            }
        }
        ctx.set_global_alpha(1.0);
    }

    // Active frame
    for row in 0..project.height {
        for col in 0..project.width {
            let idx = (row * project.width + col) as usize;
            let c = &frame.cells[idx];
            let x = col as f64 * cell;
            let y = row as f64 * cell;
            ctx.set_fill_style_str(&format!("rgb({},{},{})", c.bg[0], c.bg[1], c.bg[2]));
            ctx.fill_rect(x, y, cell, cell);
            if c.ch != ' ' {
                if state.phosphor {
                    let fg_css = format!("rgb({},{},{})", c.fg[0], c.fg[1], c.fg[2]);
                    ctx.set_shadow_color(&fg_css);
                    ctx.set_shadow_blur(4.0);
                }
                ctx.set_fill_style_str(&format!("rgb({},{},{})", c.fg[0], c.fg[1], c.fg[2]));
                let _ = ctx.fill_text(&c.ch.to_string(), x, y);
                if state.phosphor {
                    ctx.set_shadow_blur(0.0);
                }
            }
        }
    }

    // Grid overlay
    if state.show_grid {
        ctx.begin_path();
        ctx.set_stroke_style_str("rgba(255,255,255,0.06)");
        ctx.set_line_width(0.5);
        for col in 0..=project.width {
            let x = col as f64 * cell;
            ctx.move_to(x, 0.0);
            ctx.line_to(x, px_h as f64);
        }
        for row in 0..=project.height {
            let y = row as f64 * cell;
            ctx.move_to(0.0, y);
            ctx.line_to(px_w as f64, y);
        }
        ctx.stroke();
    }
}

fn viewport_to_cell(vx: f64, vy: f64, pan_x: f64, pan_y: f64, zoom: f64, cell: f64) -> (u32, u32) {
    let cx = (vx - pan_x) / zoom;
    let cy = (vy - pan_y) / zoom;
    ((cx / cell) as u32, (cy / cell) as u32)
}

fn apply_tool(state: &mut AppState, col: u32, row: u32) {
    match state.tool {
        Tool::Brush | Tool::Rect | Tool::Line => {
            let fg = state.fg_color;
            let bg = state.bg_color;
            let ch = state.active_glyph;
            state.project.paint_cell(col, row, fg, bg, ch);
        }
        Tool::Eraser => {
            state.project.erase_cell(col, row);
        }
        Tool::Eyedrop => {
            if col < state.project.width && row < state.project.height {
                let idx = (row * state.project.width + col) as usize;
                if let Some(frame) = state.project.frames.get(state.project.active_frame) {
                    let c = &frame.cells[idx];
                    state.fg_color = c.fg;
                    state.bg_color = c.bg;
                    if c.ch != ' ' { state.active_glyph = c.ch; }
                }
                state.tool = Tool::Brush;
            }
        }
        Tool::Fill => {
            let fg = state.fg_color;
            let bg = state.bg_color;
            let ch = state.active_glyph;
            state.project.flood_fill(col, row, fg, bg, ch);
        }
    }
}

fn window_size() -> (f64, f64) {
    let Some(win) = web_sys::window() else { return (1280.0, 800.0) };
    let w = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1280.0);
    let h = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
    (w, h)
}

#[component]
pub fn Canvas() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut is_painting = use_signal(|| false);
    let mut is_panning  = use_signal(|| false);
    let mut space_held  = use_signal(|| false);
    let mut pan_start   = use_signal(|| (0.0f64, 0.0f64));
    let mut pan_start_offset = use_signal(|| (0.0f64, 0.0f64));
    let mut last_mouse  = use_signal(|| (0.0f64, 0.0f64));
    let mut dpr_signal: Signal<f64> = use_signal(|| {
        web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0)
    });

    // Centre canvas on first render using zoom from state
    let (init_px, init_py) = {
        let s = app_state.read();
        let z = s.zoom;
        let cw = s.project.width as f64 * s.cell_size * z;
        let ch = s.project.height as f64 * s.cell_size * z;
        let (vw, vh) = window_size();
        ((vw - cw) / 2.0, (vh - ch) / 2.0)
    };
    let mut pan_x = use_signal(|| init_px);
    let mut pan_y = use_signal(|| init_py);

    // Spacebar handler
    use_effect(move || {
        let doc = web_sys::window().unwrap().document().unwrap();
        let down = Closure::<dyn FnMut(_)>::wrap(Box::new(move |e: KeyboardEvent| {
            if e.code() == "Space" { e.prevent_default(); space_held.set(true); }
        }));
        let up = Closure::<dyn FnMut(_)>::wrap(Box::new(move |e: KeyboardEvent| {
            if e.code() == "Space" { space_held.set(false); }
        }));
        doc.add_event_listener_with_callback("keydown", down.as_ref().unchecked_ref()).unwrap();
        doc.add_event_listener_with_callback("keyup",   up.as_ref().unchecked_ref()).unwrap();
        down.forget(); up.forget();
    });

    // DPR change detection
    use_effect(move || {
        let win = web_sys::window().unwrap();
        let cb = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let new_dpr = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0);
            if (*dpr_signal.peek() - new_dpr).abs() > 0.01 { dpr_signal.set(new_dpr); }
        }));
        win.add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref()).unwrap();
        cb.forget();
    });

    // Redraw whenever state or DPR changes
    use_effect(move || {
        let _ = *dpr_signal.read();
        let _ = *pan_x.read();
        let _ = *pan_y.read();
        draw_project(&app_state.read());
    });

    let s = app_state.read();
    let z = s.zoom;
    let cell = s.cell_size;
    let css_w = (s.project.width as f64 * cell) as u32;
    let css_h = (s.project.height as f64 * cell) as u32;
    let active_idx = s.project.active_frame;
    let total = s.project.frames.len();
    let pw = s.project.width;
    let ph = s.project.height;
    drop(s);

    let px = *pan_x.read();
    let py = *pan_y.read();
    let panning = *is_panning.read();
    let space = *space_held.read();

    let cursor = if panning { "grabbing" } else if space { "grab" } else { "crosshair" };

    rsx! {
        div { class: "canvas-area",

            // Tabline header
            div { class: "canvas-tabline mono",
                span { style: "color: #39ff14;", "●" }
                " frame "
                span { "{active_idx + 1:02}/{total:02}" }
                span { class: "grow" }
                span { class: "mute", "{pw}×{ph}" }
            }

            // Stage
            div { class: "canvas-frame",
                div {
                    class: "canvas-stage",
                    style: "cursor: {cursor};",

                    onwheel: move |evt| {
                        let (dx, dy) = match evt.delta() {
                            WheelDelta::Pixels(v) => (v.x, v.y),
                            WheelDelta::Lines(v)  => (v.x * 20.0, v.y * 20.0),
                            WheelDelta::Pages(v)  => (v.x * 300.0, v.y * 300.0),
                        };
                        if evt.modifiers().ctrl() {
                            if dy == 0.0 { return; }
                            let factor = if dy > 0.0 { 1.0 / 1.1 } else { 1.1 };
                            let old_z = app_state.read().zoom;
                            let new_z = (old_z * factor).clamp(0.25, 8.0);
                            let (mx, my) = *last_mouse.read();
                            let ratio = new_z / old_z;
                            let old_px = *pan_x.read();
                            let old_py = *pan_y.read();
                            pan_x.set(mx * (1.0 - ratio) + old_px * ratio);
                            pan_y.set(my * (1.0 - ratio) + old_py * ratio);
                            app_state.with_mut(|s| s.zoom = new_z);
                        } else if evt.modifiers().shift() {
                            let scroll = if dx != 0.0 { dx } else { dy };
                            let cur = *pan_x.read();
                            pan_x.set(cur - scroll);
                        } else {
                            let cur_y = *pan_y.read();
                            pan_y.set(cur_y - dy);
                            if dx != 0.0 {
                                let cur_x = *pan_x.read();
                                pan_x.set(cur_x - dx);
                            }
                        }
                    },

                    onmousedown: move |evt| {
                        let coords = evt.element_coordinates();
                        let btn = evt.trigger_button();
                        let mut start_pan = || {
                            is_panning.set(true);
                            pan_start.set((coords.x, coords.y));
                            pan_start_offset.set((*pan_x.read(), *pan_y.read()));
                        };
                        if btn == Some(MouseButton::Auxiliary) {
                            start_pan();
                        } else if btn == Some(MouseButton::Primary) {
                            if *space_held.read() {
                                start_pan();
                            } else {
                                is_painting.set(true);
                                let cell = app_state.read().cell_size;
                                let zoom = app_state.read().zoom;
                                let (col, row) = viewport_to_cell(
                                    coords.x, coords.y, *pan_x.read(), *pan_y.read(), zoom, cell,
                                );
                                app_state.with_mut(|s| apply_tool(s, col, row));
                            }
                        }
                    },

                    onmousemove: move |evt| {
                        let coords = evt.element_coordinates();
                        last_mouse.set((coords.x, coords.y));
                        if *is_painting.read() {
                            let cell = app_state.read().cell_size;
                            let zoom = app_state.read().zoom;
                            let (col, row) = viewport_to_cell(
                                coords.x, coords.y, *pan_x.read(), *pan_y.read(), zoom, cell,
                            );
                            app_state.with_mut(|s| apply_tool(s, col, row));
                        } else if *is_panning.read() {
                            let (sx, sy) = *pan_start.read();
                            let (ox, oy) = *pan_start_offset.read();
                            pan_x.set(ox + coords.x - sx);
                            pan_y.set(oy + coords.y - sy);
                        }
                    },

                    onmouseup:    move |_| { is_painting.set(false); is_panning.set(false); },
                    onmouseleave: move |_| { is_painting.set(false); is_panning.set(false); },

                    // Checkerboard background
                    div { style: "position: absolute; inset: 0; background: #0a0d10;" }

                    // Transformed canvas stage
                    div {
                        style: "position: absolute; top: 0; left: 0; transform: translate({px}px, {py}px) scale({z}); transform-origin: 0 0; box-shadow: 0 0 0 1px #2a343a, 0 20px 60px rgba(0,0,0,0.8);",
                        canvas {
                            id: CANVAS_ID,
                            style: "display: block; image-rendering: pixelated; width: {css_w}px; height: {css_h}px;",
                        }
                    }
                }
            }
        }
    }
}
