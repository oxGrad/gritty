use dioxus::prelude::*;
use crate::state::{AppState, Tool, GLYPH_GROUPS};

#[component]
pub fn LeftPanel() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let show = app_state.read().show_left_panel;

    if !show {
        return rsx! {
            button {
                class: "fixed left-0 top-1/2 -translate-y-1/2 w-5 h-14 bg-[#252525] border border-l-0 border-[#3c3c3c] rounded-r-lg text-[#9ca0a4] hover:text-[#fcfcfa] hover:bg-[#303030] text-xs z-20 flex items-center justify-center",
                title: "Show tools panel",
                onclick: move |_| app_state.with_mut(|s| s.show_left_panel = true),
                "▸"
            }
        };
    }

    rsx! {
        div {
            class: "fixed left-3 top-14 bottom-16 w-[80px] bg-[#252525] border border-[#3c3c3c] rounded-xl shadow-2xl flex flex-col gap-2 p-2 overflow-y-auto z-10",

            // Tools
            div { class: "flex flex-col gap-1",
                span { class: "text-[10px] text-[#5b595c] tracking-widest uppercase", "Tool" }
                button {
                    class: if app_state.read().tool == Tool::Brush {
                        "w-full h-8 rounded-lg text-sm font-bold bg-[#ff6188] text-[#1a1a1a]"
                    } else {
                        "w-full h-8 rounded-lg text-sm bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]"
                    },
                    aria_label: "Brush",
                    aria_pressed: if app_state.read().tool == Tool::Brush { "true" } else { "false" },
                    onclick: move |_| app_state.with_mut(|s| s.tool = Tool::Brush),
                    "✏"
                }
                button {
                    class: if app_state.read().tool == Tool::Eraser {
                        "w-full h-8 rounded-lg text-sm font-bold bg-[#78dce8] text-[#1a1a1a]"
                    } else {
                        "w-full h-8 rounded-lg text-sm bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]"
                    },
                    aria_label: "Eraser",
                    aria_pressed: if app_state.read().tool == Tool::Eraser { "true" } else { "false" },
                    onclick: move |_| app_state.with_mut(|s| s.tool = Tool::Eraser),
                    "◻"
                }
            }

            div { class: "border-t border-[#333333]" }

            // Grid
            div { class: "flex flex-col gap-1",
                span { class: "text-[10px] text-[#5b595c] tracking-widest uppercase", "Grid" }
                button {
                    class: if app_state.read().show_grid {
                        "w-full h-8 rounded-lg text-sm font-bold bg-[#ffd866] text-[#1a1a1a]"
                    } else {
                        "w-full h-8 rounded-lg text-sm bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]"
                    },
                    title: "Toggle grid",
                    aria_label: if app_state.read().show_grid { "Hide grid" } else { "Show grid" },
                    aria_pressed: if app_state.read().show_grid { "true" } else { "false" },
                    onclick: move |_| app_state.with_mut(|s| s.show_grid = !s.show_grid),
                    "⊞"
                }
            }

            div { class: "border-t border-[#333333]" }

            // Shift
            div { class: "flex flex-col gap-1",
                span { class: "text-[10px] text-[#5b595c] tracking-widest uppercase", "Shift" }

                span { class: "text-[9px] text-[#444444]", "Frame" }
                button {
                    class: "w-full h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                    title: "Shift frame up",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_up()),
                    "↑"
                }
                div { class: "flex gap-0.5",
                    button {
                        class: "flex-1 h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                        title: "Shift frame left",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_left()),
                        "←"
                    }
                    button {
                        class: "flex-1 h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                        title: "Shift frame right",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_right()),
                        "→"
                    }
                }
                button {
                    class: "w-full h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                    title: "Shift frame down",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_down()),
                    "↓"
                }

                span { class: "text-[9px] text-[#444444]", "All" }
                button {
                    class: "w-full h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                    title: "Shift all frames up",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_all_up()),
                    "↑"
                }
                div { class: "flex gap-0.5",
                    button {
                        class: "flex-1 h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                        title: "Shift all frames left",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_all_left()),
                        "←"
                    }
                    button {
                        class: "flex-1 h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                        title: "Shift all frames right",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_all_right()),
                        "→"
                    }
                }
                button {
                    class: "w-full h-7 rounded-lg text-xs bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]",
                    title: "Shift all frames down",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_all_down()),
                    "↓"
                }
            }

            div { class: "border-t border-[#333333]" }

            // Glyphs
            div { class: "flex flex-col gap-1",
                span { class: "text-[10px] text-[#5b595c] tracking-widest uppercase", "Glyph" }
                for (label, glyphs) in GLYPH_GROUPS.iter() {
                    div { class: "flex flex-col gap-0.5",
                        span { class: "text-[9px] text-[#444444]", "{label}" }
                        div { class: "flex flex-wrap gap-0.5",
                            for ch in glyphs.iter() {
                                {
                                    let ch = *ch;
                                    let is_active = app_state.read().active_glyph == ch;
                                    rsx! {
                                        button {
                                            class: if is_active {
                                                "w-7 h-7 text-base rounded-lg bg-[#ffd866] text-[#1a1a1a] font-bold"
                                            } else {
                                                "w-7 h-7 text-base rounded-lg bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]"
                                            },
                                            aria_label: "Glyph {ch}",
                                            aria_pressed: if is_active { "true" } else { "false" },
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

            // Collapse button pinned to bottom
            div { class: "mt-auto pt-2 border-t border-[#333333]",
                button {
                    class: "w-full h-7 rounded-lg text-[#5b595c] hover:text-[#9ca0a4] hover:bg-[#2e2e2e] text-xs",
                    title: "Hide tools panel",
                    onclick: move |_| app_state.with_mut(|s| s.show_left_panel = false),
                    "◂ hide"
                }
            }
        }
    }
}
