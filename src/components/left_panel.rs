use dioxus::prelude::*;
use crate::state::{AppState, Tool, GLYPH_GROUPS};

#[component]
pub fn LeftPanel() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    rsx! {
        div {
            class: "flex flex-col gap-2 p-2 bg-[#221f22] border-r border-[#403e41] w-[72px] overflow-y-auto",

            div { class: "flex flex-col gap-1",
                span { class: "text-[11px] text-[#9ca0a4] tracking-widest uppercase", "Tool" }
                button {
                    class: {
                        let is_brush = app_state.read().tool == Tool::Brush;
                        if is_brush {
                            "w-full h-8 rounded text-sm font-bold bg-[#ff6188] text-[#2d2a2e]"
                        } else {
                            "w-full h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]"
                        }
                    },
                    aria_label: "Brush",
                    aria_pressed: if app_state.read().tool == Tool::Brush { "true" } else { "false" },
                    onclick: move |_| app_state.with_mut(|s| s.tool = Tool::Brush),
                    "✏"
                }
                button {
                    class: {
                        let is_eraser = app_state.read().tool == Tool::Eraser;
                        if is_eraser {
                            "w-full h-8 rounded text-sm font-bold bg-[#78dce8] text-[#2d2a2e]"
                        } else {
                            "w-full h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]"
                        }
                    },
                    aria_label: "Eraser",
                    aria_pressed: if app_state.read().tool == Tool::Eraser { "true" } else { "false" },
                    onclick: move |_| app_state.with_mut(|s| s.tool = Tool::Eraser),
                    "◻"
                }
            }

            div { class: "border-t border-[#403e41]" }

            div { class: "flex flex-col gap-1",
                span { class: "text-[11px] text-[#9ca0a4] tracking-widest uppercase", "Grid" }
                button {
                    class: {
                        let show = app_state.read().show_grid;
                        if show {
                            "w-full h-8 rounded text-sm font-bold bg-[#ffd866] text-[#2d2a2e]"
                        } else {
                            "w-full h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]"
                        }
                    },
                    title: "Toggle grid",
                    aria_label: if app_state.read().show_grid { "Hide grid" } else { "Show grid" },
                    aria_pressed: if app_state.read().show_grid { "true" } else { "false" },
                    onclick: move |_| app_state.with_mut(|s| s.show_grid = !s.show_grid),
                    "⊞"
                }
            }

            div { class: "border-t border-[#403e41]" }

            div { class: "flex flex-col gap-1",
                span { class: "text-[11px] text-[#9ca0a4] tracking-widest uppercase", "Shift" }

                span { class: "text-[10px] text-[#5b595c]", "Frame" }
                button {
                    class: "w-full h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                    title: "Shift frame up",
                    aria_label: "Shift frame up",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_up()),
                    "↑"
                }
                div { class: "flex gap-0.5",
                    button {
                        class: "flex-1 h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                        title: "Shift frame left",
                        aria_label: "Shift frame left",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_left()),
                        "←"
                    }
                    button {
                        class: "flex-1 h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                        title: "Shift frame right",
                        aria_label: "Shift frame right",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_right()),
                        "→"
                    }
                }
                button {
                    class: "w-full h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                    title: "Shift frame down",
                    aria_label: "Shift frame down",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_down()),
                    "↓"
                }

                span { class: "text-[10px] text-[#5b595c]", "All" }
                button {
                    class: "w-full h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                    title: "Shift all frames up",
                    aria_label: "Shift all frames up",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_all_up()),
                    "↑"
                }
                div { class: "flex gap-0.5",
                    button {
                        class: "flex-1 h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                        title: "Shift all frames left",
                        aria_label: "Shift all frames left",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_all_left()),
                        "←"
                    }
                    button {
                        class: "flex-1 h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                        title: "Shift all frames right",
                        aria_label: "Shift all frames right",
                        onclick: move |_| app_state.with_mut(|s| s.project.shift_all_right()),
                        "→"
                    }
                }
                button {
                    class: "w-full h-8 rounded text-sm bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]",
                    title: "Shift all frames down",
                    aria_label: "Shift all frames down",
                    onclick: move |_| app_state.with_mut(|s| s.project.shift_all_down()),
                    "↓"
                }
            }

            div { class: "border-t border-[#403e41]" }

            div { class: "flex flex-col gap-1",
                span { class: "text-[11px] text-[#9ca0a4] tracking-widest uppercase", "Glyph" }
                for (label, glyphs) in GLYPH_GROUPS.iter() {
                    div { class: "flex flex-col gap-0.5",
                        span { class: "text-[10px] text-[#5b595c]", "{label}" }
                        div { class: "flex flex-wrap gap-0.5",
                            for ch in glyphs.iter() {
                                {
                                    let ch = *ch;
                                    let is_active = app_state.read().active_glyph == ch;
                                    rsx! {
                                        button {
                                            class: if is_active {
                                                "w-7 h-7 text-base rounded bg-[#ffd866] text-[#2d2a2e] font-bold"
                                            } else {
                                                "w-7 h-7 text-base rounded bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c]"
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
        }
    }
}
