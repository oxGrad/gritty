use dioxus::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
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
    let sq_half = inner_r * 0.68;

    let mut pixels = vec![0u8; (w * h * 4) as usize];
    for row in 0..h {
        for col in 0..w {
            let x = col as f64 - cx;
            let y = row as f64 - cy;
            let dist = (x * x + y * y).sqrt();
            let idx = ((row * w + col) * 4) as usize;

            if dist >= inner_r && dist <= outer_r {
                let angle = y.atan2(x).to_degrees();
                let h_deg = (angle + 360.0) % 360.0;
                let [r, g, b] = hsv_to_rgb(h_deg, 1.0, 1.0);
                pixels[idx]     = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            } else if x.abs() <= sq_half && y.abs() <= sq_half {
                let s = ((x + sq_half) / (2.0 * sq_half)).clamp(0.0, 1.0);
                let v = ((sq_half - y) / (2.0 * sq_half)).clamp(0.0, 1.0);
                let [r, g, b] = hsv_to_rgb(hue, s, v);
                pixels[idx]     = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            }
        }
    }

    if let Ok(image_data) = ImageData::new_with_u8_clamped_array_and_sh(Clamped(pixels.as_slice()), w, h) {
        let _ = ctx.put_image_data(&image_data, 0.0, 0.0);
    }

    let hue_rad = hue.to_radians();
    let ind_r = (inner_r + outer_r) / 2.0;
    let ix = cx + ind_r * hue_rad.cos();
    let iy = cy + ind_r * hue_rad.sin();
    ctx.begin_path();
    let _ = ctx.arc(ix, iy, 4.0, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style_str("#fcfcfa");
    ctx.set_line_width(1.5);
    ctx.stroke();

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
                            let angle = dy.atan2(dx).to_degrees();
                            let new_h = (angle + 360.0) % 360.0;
                            hsv_to_rgb(new_h, cur_s.max(0.1), cur_v.max(0.1))
                        } else if dx.abs() <= sq_half && dy.abs() <= sq_half {
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
