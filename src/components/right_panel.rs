use std::cell::RefCell;

use dioxus::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use crate::color::{hsv_to_rgb, parse_hex, rgb_to_hsv, to_hex};
use crate::state::{AppState, ColorTarget};

const WHEEL_ID: &str = "color-wheel";
const WHEEL_SIZE: u32 = 120;
const RING_W: f64 = 16.0;

thread_local! {
    static RING_CACHE: RefCell<Option<(f64, f64, Vec<u8>)>> = const { RefCell::new(None) };
}

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

    let dpr = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0);
    let phys = (WHEEL_SIZE as f64 * dpr) as u32;
    canvas.set_width(phys);
    canvas.set_height(phys);

    let cx = phys as f64 / 2.0;
    let cy = phys as f64 / 2.0;
    let outer_r = cx - 2.0 * dpr;
    let inner_r = outer_r - RING_W * dpr;
    let sq_half = inner_r * 0.68;

    let mut pixels = RING_CACHE.with(|cache| {
        let mut c = cache.borrow_mut();
        if let Some((cached_hue, cached_dpr, ref px)) = *c {
            if (cached_hue - hue).abs() < 0.001 && (cached_dpr - dpr).abs() < 0.001 {
                return px.clone();
            }
        }
        let mut ring = vec![0u8; (phys * phys * 4) as usize];
        for row in 0..phys {
            for col in 0..phys {
                let x = col as f64 - cx;
                let y = row as f64 - cy;
                let dist = (x * x + y * y).sqrt();
                if dist >= inner_r && dist <= outer_r {
                    let angle = y.atan2(x).to_degrees();
                    let h_deg = (angle + 360.0) % 360.0;
                    let [r, g, b] = hsv_to_rgb(h_deg, 1.0, 1.0);
                    let idx = ((row * phys + col) * 4) as usize;
                    ring[idx]     = r;
                    ring[idx + 1] = g;
                    ring[idx + 2] = b;
                    ring[idx + 3] = 255;
                }
            }
        }
        let result = ring.clone();
        *c = Some((hue, dpr, ring));
        result
    });

    for row in 0..phys {
        for col in 0..phys {
            let x = col as f64 - cx;
            let y = row as f64 - cy;
            if x.abs() <= sq_half && y.abs() <= sq_half {
                let s = ((x + sq_half) / (2.0 * sq_half)).clamp(0.0, 1.0);
                let v = ((sq_half - y) / (2.0 * sq_half)).clamp(0.0, 1.0);
                let [r, g, b] = hsv_to_rgb(hue, s, v);
                let idx = ((row * phys + col) * 4) as usize;
                pixels[idx]     = r;
                pixels[idx + 1] = g;
                pixels[idx + 2] = b;
                pixels[idx + 3] = 255;
            }
        }
    }

    if let Ok(image_data) = ImageData::new_with_u8_clamped_array_and_sh(Clamped(pixels.as_slice()), phys, phys) {
        let _ = ctx.put_image_data(&image_data, 0.0, 0.0);
    }

    let hue_rad = hue.to_radians();
    let ind_r = (inner_r + outer_r) / 2.0;
    let ix = cx + ind_r * hue_rad.cos();
    let iy = cy + ind_r * hue_rad.sin();
    ctx.begin_path();
    let _ = ctx.arc(ix, iy, 4.0 * dpr, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style_str("#fcfcfa");
    ctx.set_line_width(1.5 * dpr);
    ctx.stroke();

    let sx = cx - sq_half + sat * 2.0 * sq_half;
    let sy = cy + sq_half - val * 2.0 * sq_half;
    ctx.begin_path();
    let _ = ctx.arc(sx, sy, 4.0 * dpr, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style_str("#fcfcfa");
    ctx.stroke();
}

#[component]
pub fn RightPanel() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let show = app_state.read().show_right_panel;

    if !show {
        return rsx! {
            button {
                class: "fixed right-0 top-1/2 -translate-y-1/2 w-5 h-14 bg-[#252525] border border-r-0 border-[#3c3c3c] rounded-l-lg text-[#9ca0a4] hover:text-[#fcfcfa] hover:bg-[#303030] text-xs z-20 flex items-center justify-center",
                title: "Show color panel",
                onclick: move |_| app_state.with_mut(|s| s.show_right_panel = true),
                "◂"
            }
        };
    }

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
            class: "fixed right-3 top-14 bottom-16 w-[152px] bg-[#252525] border border-[#3c3c3c] rounded-xl shadow-2xl flex flex-col items-center gap-2 p-2 overflow-y-auto z-10",

            span { class: "text-[10px] text-[#5b595c] tracking-widest uppercase self-start", "Color" }

            canvas {
                id: WHEEL_ID,
                style: "cursor: crosshair; border-radius: 50%; width: {WHEEL_SIZE}px; height: {WHEEL_SIZE}px;",
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
                aria_label: "Color hex value",
                class: "w-full bg-[#2e2e2e] text-[#fcfcfa] text-sm text-center rounded-lg px-1 py-1 font-mono border border-[#3c3c3c] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[#ff6188]",
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
                span { class: "text-[10px] text-[#5b595c] tracking-widest uppercase", "FG / BG" }
                div { class: "flex gap-1",
                    button {
                        class: if matches!(color_target, ColorTarget::Fg) {
                            "flex-1 h-8 rounded-lg border-2 border-[#ffd866]"
                        } else {
                            "flex-1 h-8 rounded-lg border-2 border-transparent hover:border-[#3c3c3c]"
                        },
                        style: "background-color: {fg_hex};",
                        onclick: move |_| app_state.with_mut(|s| s.color_target = ColorTarget::Fg),
                        aria_label: "Foreground color",
                        aria_pressed: if matches!(color_target, ColorTarget::Fg) { "true" } else { "false" },
                    }
                    button {
                        class: if matches!(color_target, ColorTarget::Bg) {
                            "flex-1 h-8 rounded-lg border-2 border-[#ffd866]"
                        } else {
                            "flex-1 h-8 rounded-lg border-2 border-transparent hover:border-[#3c3c3c]"
                        },
                        style: "background-color: {bg_hex};",
                        onclick: move |_| app_state.with_mut(|s| s.color_target = ColorTarget::Bg),
                        aria_label: "Background color",
                        aria_pressed: if matches!(color_target, ColorTarget::Bg) { "true" } else { "false" },
                    }
                }
            }

            // Collapse button
            div { class: "mt-auto pt-2 border-t border-[#333333] w-full",
                button {
                    class: "w-full h-7 rounded-lg text-[#5b595c] hover:text-[#9ca0a4] hover:bg-[#2e2e2e] text-xs",
                    title: "Hide color panel",
                    onclick: move |_| app_state.with_mut(|s| s.show_right_panel = false),
                    "hide ▸"
                }
            }
        }
    }
}
