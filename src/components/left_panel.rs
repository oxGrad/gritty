use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::state::{AppState, Frame};

fn draw_thumbnail(idx: usize, frame: &Frame, w: u32, h: u32) {
    let id = format!("frame-thumb-{idx}");
    let Some(canvas) = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.get_element_by_id(&id))
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    else { return };

    let Some(ctx) = canvas
        .get_context("2d").ok().flatten()
        .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
    else { return };

    let tw = w * 2;
    let th = h * 2;
    canvas.set_width(tw);
    canvas.set_height(th);

    ctx.set_fill_style_str("#000000");
    ctx.fill_rect(0.0, 0.0, tw as f64, th as f64);

    for row in 0..h {
        for col in 0..w {
            let cell = &frame.cells[(row * w + col) as usize];
            if cell.ch != ' ' {
                ctx.set_fill_style_str(&format!(
                    "rgb({},{},{})", cell.fg[0], cell.fg[1], cell.fg[2]
                ));
                ctx.fill_rect((col * 2) as f64, (row * 2) as f64, 2.0, 2.0);
            }
        }
    }
}

#[component]
pub fn LeftPanel() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    // Redraw all thumbnails whenever state changes
    use_effect(move || {
        let s = app_state.read();
        let w = s.project.width;
        let h = s.project.height;
        for (i, frame) in s.project.frames.iter().enumerate() {
            draw_thumbnail(i, frame, w, h);
        }
    });

    let (frame_count, active_idx, onion_skin) = {
        let s = app_state.read();
        (s.project.frames.len(), s.project.active_frame, s.onion_skin)
    };

    rsx! {
        aside { class: "left-panel",
            // Header
            div { class: "panel-header",
                span { class: "eyebrow", "Frames" }
                span { class: "badge-mini", "{frame_count}" }
            }

            // Frame list
            div { class: "frame-list",
                for i in 0..frame_count {
                    div {
                        class: if i == active_idx { "frame-card active" } else { "frame-card" },
                        onclick: move |_| app_state.with_mut(|s| s.project.active_frame = i),
                        div { class: "frame-num", "{i + 1:02}" }
                        canvas {
                            id: "frame-thumb-{i}",
                            class: "frame-thumb",
                        }
                        div { class: "frame-meta",
                            span { class: "mono", "F{i + 1}" }
                            span { class: "mute mono",
                                { let s = app_state.read(); format!("{}ms", (1000.0 / s.fps as f64).round() as u32) }
                            }
                        }
                    }
                }

                button {
                    class: "frame-add",
                    onclick: move |_| app_state.with_mut(|s| {
                        s.project.add_frame();
                        s.project.active_frame = s.project.frames.len() - 1;
                    }),
                    span { class: "plus", "+" }
                    span { "Add frame" }
                }
            }

            // Footer actions
            div { class: "panel-footer",
                button {
                    class: "ico-btn",
                    title: "Add frame (N)",
                    onclick: move |_| app_state.with_mut(|s| {
                        s.project.add_frame();
                        s.project.active_frame = s.project.frames.len() - 1;
                    }),
                    "+"
                }
                button {
                    class: "ico-btn",
                    title: "Duplicate frame (Ctrl+D)",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.duplicate_frame(idx);
                        s.project.active_frame = idx + 1;
                    }),
                    "❏"
                }
                button {
                    class: "ico-btn",
                    title: "Delete frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.delete_frame(idx);
                    }),
                    "−"
                }
                span { class: "divider" }
                button {
                    class: "ico-btn",
                    title: "Move frame up",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_up(idx);
                    }),
                    "↑"
                }
                button {
                    class: "ico-btn",
                    title: "Move frame down",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_down(idx);
                    }),
                    "↓"
                }
                span { class: "divider" }
                button {
                    class: if onion_skin { "ico-btn on" } else { "ico-btn" },
                    title: "Onion skin",
                    onclick: move |_| app_state.with_mut(|s| s.onion_skin = !s.onion_skin),
                    span { style: "font-size: 10px;", "◐" }
                }
            }
        }
    }
}
