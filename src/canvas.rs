use dioxus::prelude::*;
use dioxus_elements::geometry::WheelDelta;
use dioxus_elements::input_data::MouseButton;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

use crate::state::{AppState, Tool, CELL_H, CELL_W};

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

fn draw_project(state: &AppState) {
    let project = &state.project;
    let Some((canvas, ctx)) = get_canvas_ctx() else { return };

    let px_w = (project.width as f64 * CELL_W) as u32;
    let px_h = (project.height as f64 * CELL_H) as u32;
    let dpr = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0);
    canvas.set_width((px_w as f64 * dpr) as u32);
    canvas.set_height((px_h as f64 * dpr) as u32);
    let _ = ctx.scale(dpr, dpr);

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

            ctx.set_fill_style_str(&format!(
                "rgb({},{},{})", cell.bg[0], cell.bg[1], cell.bg[2]
            ));
            ctx.fill_rect(x, y, CELL_W, CELL_H);

            if cell.ch != ' ' {
                ctx.set_fill_style_str(&format!(
                    "rgb({},{},{})", cell.fg[0], cell.fg[1], cell.fg[2]
                ));
                let _ = ctx.fill_text(&cell.ch.to_string(), x, y);
            }
        }
    }

    if state.show_grid {
        ctx.begin_path();
        ctx.set_stroke_style_str("rgba(255,255,255,0.12)");
        ctx.set_line_width(0.5);
        for col in 0..=project.width {
            let x = col as f64 * CELL_W;
            ctx.move_to(x, 0.0);
            ctx.line_to(x, px_h as f64);
        }
        for row in 0..=project.height {
            let y = row as f64 * CELL_H;
            ctx.move_to(0.0, y);
            ctx.line_to(px_w as f64, y);
        }
        ctx.stroke();
    }
}

// Convert a viewport-space point to a canvas cell coordinate.
fn viewport_to_cell(vx: f64, vy: f64, pan_x: f64, pan_y: f64, zoom: f64) -> (u32, u32) {
    let cx = (vx - pan_x) / zoom;
    let cy = (vy - pan_y) / zoom;
    ((cx / CELL_W) as u32, (cy / CELL_H) as u32)
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

fn window_size() -> (f64, f64) {
    let win = match web_sys::window() {
        Some(w) => w,
        None => return (1280.0, 800.0),
    };
    let w = win.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1280.0);
    let h = win.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(800.0);
    (w, h)
}

#[component]
pub fn Canvas() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut is_painting = use_signal(|| false);
    let mut is_panning = use_signal(|| false);
    let mut space_held = use_signal(|| false);
    let mut pan_start = use_signal(|| (0.0f64, 0.0f64));
    let mut pan_start_offset = use_signal(|| (0.0f64, 0.0f64));
    let mut last_mouse = use_signal(|| (0.0f64, 0.0f64));

    // Centre canvas in viewport at 2× zoom on first render.
    let (init_px, init_py) = {
        let (vw, vh) = window_size();
        let s = app_state.read();
        let z = 2.0f64;
        let cw = s.project.width as f64 * CELL_W * z;
        let ch = s.project.height as f64 * CELL_H * z;
        ((vw - cw) / 2.0, (vh - ch) / 2.0)
    };
    let mut zoom = use_signal(|| 2.0f64);
    let mut pan_x = use_signal(|| init_px);
    let mut pan_y = use_signal(|| init_py);

    // Global spacebar listeners — space held turns left-drag into pan.
    use_effect(move || {
        let document = web_sys::window().unwrap().document().unwrap();

        let down_cb = Closure::<dyn FnMut(_)>::wrap(Box::new(move |evt: KeyboardEvent| {
            if evt.code() == "Space" {
                evt.prevent_default(); // stop browser from scrolling
                space_held.set(true);
            }
        }));
        let up_cb = Closure::<dyn FnMut(_)>::wrap(Box::new(move |evt: KeyboardEvent| {
            if evt.code() == "Space" {
                space_held.set(false);
            }
        }));

        document.add_event_listener_with_callback("keydown", down_cb.as_ref().unchecked_ref()).unwrap();
        document.add_event_listener_with_callback("keyup",   up_cb.as_ref().unchecked_ref()).unwrap();

        // Leak the closures — they live for the app's lifetime.
        down_cb.forget();
        up_cb.forget();
    });

    use_effect(move || {
        draw_project(&app_state.read());
    });

    let (css_w, css_h) = {
        let s = app_state.read();
        (
            (s.project.width as f64 * CELL_W) as u32,
            (s.project.height as f64 * CELL_H) as u32,
        )
    };

    let z = *zoom.read();
    let px = *pan_x.read();
    let py = *pan_y.read();
    let panning = *is_panning.read();
    let space = *space_held.read();
    let zoom_pct = (z * 100.0).round() as i32;

    let cursor = if panning { "cursor: grabbing;" }
                 else if space { "cursor: grab;" }
                 else { "cursor: crosshair;" };

    rsx! {
        // Workspace — fills full viewport, sits behind all floating panels.
        div {
            class: "absolute inset-0",
            style: cursor,

            onwheel: move |evt| {
                let delta_x = match evt.delta() {
                    WheelDelta::Pixels(v) => (v.x, v.y),
                    WheelDelta::Lines(v)  => (v.x * 20.0, v.y * 20.0),
                    WheelDelta::Pages(v)  => (v.x * 300.0, v.y * 300.0),
                };
                let (dx, dy) = delta_x;
                if evt.modifiers().ctrl() {
                    // Ctrl+scroll → zoom to cursor
                    if dy == 0.0 { return; }
                    let factor = if dy > 0.0 { 1.0 / 1.1 } else { 1.1 };
                    let old_z = *zoom.read();
                    let new_z = (old_z * factor).clamp(0.1, 16.0);
                    let (mx, my) = *last_mouse.read();
                    let ratio = new_z / old_z;
                    let old_px = *pan_x.read();
                    let old_py = *pan_y.read();
                    pan_x.set(mx * (1.0 - ratio) + old_px * ratio);
                    pan_y.set(my * (1.0 - ratio) + old_py * ratio);
                    zoom.set(new_z);
                } else if evt.modifiers().shift() {
                    // Shift+scroll → pan horizontally (use dy when device only sends dy)
                    let scroll = if dx != 0.0 { dx } else { dy };
                    let cur = *pan_x.read();
                    pan_x.set(cur - scroll);
                } else {
                    // Plain scroll → pan (vertical + horizontal for trackpads)
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
                    // Middle-click always pans
                    start_pan();
                } else if btn == Some(MouseButton::Primary) {
                    if *space_held.read() {
                        // Space + left-click = pan (Figma hand tool)
                        start_pan();
                    } else {
                        is_painting.set(true);
                        let (col, row) = viewport_to_cell(
                            coords.x, coords.y, *pan_x.read(), *pan_y.read(), *zoom.read(),
                        );
                        app_state.with_mut(|s| apply_tool(s, col, row));
                    }
                }
            },

            onmousemove: move |evt| {
                let coords = evt.element_coordinates();
                last_mouse.set((coords.x, coords.y));
                if *is_painting.read() {
                    let (col, row) = viewport_to_cell(
                        coords.x, coords.y, *pan_x.read(), *pan_y.read(), *zoom.read(),
                    );
                    app_state.with_mut(|s| apply_tool(s, col, row));
                } else if *is_panning.read() {
                    let (sx, sy) = *pan_start.read();
                    let (ox, oy) = *pan_start_offset.read();
                    pan_x.set(ox + coords.x - sx);
                    pan_y.set(oy + coords.y - sy);
                }
            },

            onmouseup: move |_| {
                is_painting.set(false);
                is_panning.set(false);
            },
            onmouseleave: move |_| {
                is_painting.set(false);
                is_panning.set(false);
            },

            // Checkerboard workspace background
            div { class: "absolute inset-0", style: "background: #1a1a1a;" }

            // Stage — the transformed canvas container
            div {
                style: "position: absolute; top: 0; left: 0; transform: translate({px}px, {py}px) scale({z}); transform-origin: 0 0; box-shadow: 0 0 0 1px #3c3c3c, 0 12px 48px rgba(0,0,0,0.7);",
                canvas {
                    id: CANVAS_ID,
                    style: "display: block; image-rendering: pixelated; width: {css_w}px; height: {css_h}px;",
                }
            }

            // Zoom HUD — centred above the timeline
            div {
                class: "fixed bottom-16 left-1/2 -translate-x-1/2 flex items-center gap-0.5 z-30",
                button {
                    class: "w-7 h-7 rounded-lg bg-[#252525] border border-[#3c3c3c] text-[#9ca0a4] text-base leading-none hover:bg-[#303030] hover:text-[#fcfcfa]",
                    title: "Zoom out",
                    onclick: move |_| {
                        let new_z = (*zoom.read() / 1.25).clamp(0.1, 16.0);
                        zoom.set(new_z);
                    },
                    "−"
                }
                button {
                    class: "min-w-[52px] h-7 rounded-lg bg-[#252525] border border-[#3c3c3c] text-[#9ca0a4] text-xs font-mono hover:bg-[#303030] hover:text-[#fcfcfa]",
                    title: "Reset zoom to 100%",
                    onclick: move |_| {
                        let (vw, vh) = window_size();
                        let s = app_state.read();
                        let cw = s.project.width as f64 * CELL_W;
                        let ch = s.project.height as f64 * CELL_H;
                        zoom.set(1.0);
                        pan_x.set((vw - cw) / 2.0);
                        pan_y.set((vh - ch) / 2.0);
                    },
                    "{zoom_pct}%"
                }
                button {
                    class: "w-7 h-7 rounded-lg bg-[#252525] border border-[#3c3c3c] text-[#9ca0a4] text-base leading-none hover:bg-[#303030] hover:text-[#fcfcfa]",
                    title: "Zoom in",
                    onclick: move |_| {
                        let new_z = (*zoom.read() * 1.25).clamp(0.1, 16.0);
                        zoom.set(new_z);
                    },
                    "+"
                }
            }
        }
    }
}
