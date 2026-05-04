use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

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

    use_effect(move || {
        let state = app_state.read();
        draw_project(&state);
    });

    let (css_w, css_h) = {
        let s = app_state.read();
        let w = (s.project.width as f64 * CELL_W) as u32;
        let h = (s.project.height as f64 * CELL_H) as u32;
        (w, h)
    };

    rsx! {
        div {
            class: "flex-1 overflow-auto bg-[#19181a] flex items-start justify-start p-2",
            canvas {
                id: CANVAS_ID,
                style: "cursor: crosshair; image-rendering: pixelated; width: {css_w}px; height: {css_h}px;",
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
