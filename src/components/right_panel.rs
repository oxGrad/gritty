use dioxus::prelude::*;

use crate::color::{parse_hex, to_hex};
use crate::state::{AppState, ANSI_16, ColorTarget, GlyphSet, Tool};

const TOOLS: &[(&str, &str, &str)] = &[
    ("Brush",   "✎", "B"),
    ("Eraser",  "⌫", "E"),
    ("Fill",    "⬛", "G"),
    ("Eyedrop", "◉", "I"),
    ("Rect",    "▭", "R"),
    ("Line",    "╱", "L"),
];

#[component]
pub fn RightPanel() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut custom_char = use_signal(|| String::new());
    let mut pending_w = use_signal(|| 0u32);
    let mut pending_h = use_signal(|| 0u32);
    let mut hex_input = use_signal(|| "#39ff14".to_string());

    // Sync pending dims from state on mount
    use_effect(move || {
        let s = app_state.read();
        pending_w.set(s.project.width);
        pending_h.set(s.project.height);
    });

    let (tool, glyph_set, active_glyph, fg, bg, color_target) = {
        let s = app_state.read();
        (
            s.tool.clone(), s.glyph_set.clone(), s.active_glyph,
            to_hex(s.fg_color), to_hex(s.bg_color),
            s.color_target.clone(),
        )
    };

    let tool_label = tool.label();
    let active_hex = if matches!(color_target, ColorTarget::Fg) { fg.clone() } else { bg.clone() };

    let glyphs: Vec<char> = glyph_set.glyphs().to_vec();

    rsx! {
        aside { class: "right-panel",

            // ── Tools ───────────────────────────────────────────
            section { class: "rp-section",
                div { class: "panel-header", span { class: "eyebrow", "Tools" } }
                div { class: "tool-grid",
                    for (label, glyph, key) in TOOLS.iter() {
                        {
                            let is_active = tool_label == label.to_lowercase();
                            let label = *label;
                            rsx! {
                                button {
                                    class: if is_active { "tool-btn active" } else { "tool-btn" },
                                    title: "{label} ({key})",
                                    onclick: move |_| app_state.with_mut(|s| {
                                        s.tool = match label {
                                            "Brush"   => Tool::Brush,
                                            "Eraser"  => Tool::Eraser,
                                            "Fill"    => Tool::Fill,
                                            "Eyedrop" => Tool::Eyedrop,
                                            "Rect"    => Tool::Rect,
                                            "Line"    => Tool::Line,
                                            _         => Tool::Brush,
                                        };
                                    }),
                                    span { class: "tool-glyph", "{glyph}" }
                                    span { class: "tool-key", "{key}" }
                                }
                            }
                        }
                    }
                }
            }

            // ── Canvas size + nudge ──────────────────────────────
            section { class: "rp-section",
                div { class: "panel-header", span { class: "eyebrow", "Canvas" } }
                div { class: "size-grp",
                    label { class: "lab",
                        span { class: "mute mono", "W" }
                        input {
                            r#type: "number",
                            min: "1", max: "200",
                            value: "{pending_w}",
                            oninput: move |evt| {
                                if let Ok(v) = evt.value().parse::<u32>() { pending_w.set(v.max(1)); }
                            },
                            onblur: move |_| {
                                let w = (*pending_w.peek()).max(1);
                                let h = app_state.read().project.height;
                                app_state.with_mut(|s| s.project.resize(w, h));
                            },
                        }
                        span { class: "mute mono", "cols" }
                    }
                    button { class: "chain", "⊠" }
                    label { class: "lab",
                        span { class: "mute mono", "H" }
                        input {
                            r#type: "number",
                            min: "1", max: "200",
                            value: "{pending_h}",
                            oninput: move |evt| {
                                if let Ok(v) = evt.value().parse::<u32>() { pending_h.set(v.max(1)); }
                            },
                            onblur: move |_| {
                                let h = (*pending_h.peek()).max(1);
                                let w = app_state.read().project.width;
                                app_state.with_mut(|s| s.project.resize(w, h));
                            },
                        }
                        span { class: "mute mono", "rows" }
                    }
                }

                div { class: "panel-subhead",
                    span { class: "eyebrow-sm", "Nudge / wrap" }
                    span { class: "mute mono", "⇧+arrow" }
                }
                div { class: "nudge-pad",
                    button { class: "nudge n", onclick: move |_| app_state.with_mut(|s| s.project.shift_up()), "↑" }
                    button { class: "nudge w", onclick: move |_| app_state.with_mut(|s| s.project.shift_left()), "←" }
                    div { class: "nudge-center", "SHIFT" }
                    button { class: "nudge e", onclick: move |_| app_state.with_mut(|s| s.project.shift_right()), "→" }
                    button { class: "nudge s", onclick: move |_| app_state.with_mut(|s| s.project.shift_down()), "↓" }
                }
            }

            // ── Glyphs ───────────────────────────────────────────
            section { class: "rp-section",
                div { class: "panel-header",
                    span { class: "eyebrow", "Glyphs" }
                    select {
                        class: "set-select",
                        value: glyph_set.label(),
                        onchange: move |evt| {
                            app_state.with_mut(|s| {
                                s.glyph_set = match evt.value().as_str() {
                                    "Block Art"   => GlyphSet::BlockArt,
                                    "Box Drawing" => GlyphSet::BoxDrawing,
                                    "Shading"     => GlyphSet::Shading,
                                    _             => GlyphSet::BlockArt,
                                };
                            });
                        },
                        option { value: "Block Art",   "Block Art" }
                        option { value: "Box Drawing", "Box Drawing" }
                        option { value: "Shading",     "Shading" }
                    }
                }
                div { class: "glyph-grid",
                    for ch in glyphs.iter() {
                        {
                            let ch = *ch;
                            let is_active = ch == active_glyph;
                            rsx! {
                                button {
                                    class: if is_active { "glyph-btn active" } else { "glyph-btn" },
                                    onclick: move |_| app_state.with_mut(|s| s.active_glyph = ch),
                                    "{ch}"
                                }
                            }
                        }
                    }
                }
                div { class: "custom-char-row",
                    span { class: "lab-inline", "Custom" }
                    input {
                        class: "char-input mono",
                        maxlength: "1",
                        placeholder: "A",
                        value: "{custom_char}",
                        oninput: move |evt| custom_char.set(evt.value()),
                    }
                    button {
                        class: "btn-tiny",
                        disabled: custom_char.read().is_empty(),
                        onclick: move |_| {
                            let s = custom_char.read();
                            if let Some(ch) = s.chars().next() {
                                app_state.with_mut(|a| a.active_glyph = ch);
                            }
                        },
                        "Use →"
                    }
                }
            }

            // ── Colors ───────────────────────────────────────────
            section { class: "rp-section",
                div { class: "panel-header", span { class: "eyebrow", "Colors" } }

                div { class: "fg-bg-row",
                    // Stacked FG / BG swatches
                    div { class: "fgbg-stack",
                        button {
                            class: if matches!(color_target, ColorTarget::Bg) { "swatch bg active" } else { "swatch bg" },
                            style: "background: {bg};",
                            onclick: move |_| app_state.with_mut(|s| {
                                s.color_target = ColorTarget::Bg;
                            }),
                        }
                        button {
                            class: if matches!(color_target, ColorTarget::Fg) { "swatch fg active" } else { "swatch fg" },
                            style: "background: {fg};",
                            onclick: move |_| app_state.with_mut(|s| {
                                s.color_target = ColorTarget::Fg;
                            }),
                        }
                    }
                    div { class: "fgbg-meta",
                        button {
                            class: "btn-tiny",
                            title: "Swap FG/BG (X)",
                            onclick: move |_| app_state.with_mut(|s| {
                                let a = s.fg_color;
                                s.fg_color = s.bg_color;
                                s.bg_color = a;
                            }),
                            "⇄ Swap"
                        }
                        span { class: "mute mono", "FG " span { style: "color: {fg};", "●" } " {fg}" }
                        span { class: "mute mono", "BG " span { style: "color: {bg};", "●" } " {bg}" }
                    }
                }

                div { class: "hex-row",
                    span { class: "hash", "#" }
                    input {
                        class: "hex-input",
                        value: "{active_hex.trim_start_matches('#')}",
                        maxlength: "6",
                        oninput: move |evt| {
                            let raw = evt.value().to_uppercase();
                            let hex = format!("#{raw}");
                            hex_input.set(hex.clone());
                            if let Some(rgb) = parse_hex(&hex) {
                                app_state.with_mut(|s| match s.color_target {
                                    ColorTarget::Fg => s.fg_color = rgb,
                                    ColorTarget::Bg => s.bg_color = rgb,
                                });
                            }
                        },
                    }
                }

                div { class: "panel-subhead", span { class: "eyebrow-sm", "ANSI 16" } }
                div { class: "palette-grid",
                    for (_name, hex) in ANSI_16.iter() {
                        {
                            let hex = *hex;
                            rsx! {
                                button {
                                    class: "swatch-ansi",
                                    style: "background: {hex};",
                                    title: "{hex}",
                                    onclick: move |_| {
                                        if let Some(rgb) = parse_hex(hex) {
                                            app_state.with_mut(|s| match s.color_target {
                                                ColorTarget::Fg => s.fg_color = rgb,
                                                ColorTarget::Bg => s.bg_color = rgb,
                                            });
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
