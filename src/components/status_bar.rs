use dioxus::prelude::*;

use crate::color::to_hex;
use crate::state::AppState;

#[component]
pub fn StatusBar() -> Element {
    let app_state = use_context::<Signal<AppState>>();

    let (tool, glyph, fg, bg, active_idx, frame_count, w, h, fps) = {
        let s = app_state.read();
        (
            s.tool.label(),
            s.active_glyph,
            to_hex(s.fg_color),
            to_hex(s.bg_color),
            s.project.active_frame,
            s.project.frames.len(),
            s.project.width,
            s.project.height,
            s.fps,
        )
    };

    rsx! {
        footer { class: "statusbar",
            span { class: "sb-grp",
                span { class: "sb-dot", style: "background: #39ff14; color: #39ff14;" }
                "READY"
            }
            span { class: "sb-sep", "│" }
            span { class: "sb-grp", "tool: " b { "{tool}" } }
            span { class: "sb-sep", "│" }
            span { class: "sb-grp", "glyph: " b { style: "color: #39ff14;", "{glyph}" } }
            span { class: "sb-sep", "│" }
            span { class: "sb-grp",
                "fg: "
                span { style: "color: {fg};", "●" }
                " {fg}"
            }
            span { class: "sb-sep", "│" }
            span { class: "sb-grp",
                "bg: "
                span { style: "color: {bg};", "●" }
                " {bg}"
            }
            span { class: "sb-grp grow" }
            span { class: "sb-grp",
                "frame "
                b { "{active_idx + 1:02}/{frame_count:02}" }
            }
            span { class: "sb-sep", "│" }
            span { class: "sb-grp", "{w}×{h}" }
            span { class: "sb-sep", "│" }
            span { class: "sb-grp", "{fps} fps" }
            span { class: "sb-sep", "│" }
            span { class: "sb-grp", "utf-8 / cp437" }
        }
    }
}
