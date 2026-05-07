use std::cell::RefCell;

use dioxus::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::state::AppState;

thread_local! {
    static INTERVAL_CB: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
}

#[component]
pub fn Timeline() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut interval_id: Signal<Option<i32>> = use_signal(|| None);

    let (frame_count, active_frame, delay_ms, playing) = {
        let s = app_state.read();
        (s.project.frames.len(), s.project.active_frame, s.playback.delay_ms, s.playback.playing)
    };

    rsx! {
        div {
            class: "fixed bottom-0 left-0 right-0 h-14 bg-[#252525]/95 border-t border-[#333333] flex items-center px-3 gap-2 z-20",

            span { class: "text-[10px] text-[#444444] tracking-widest uppercase shrink-0", "FRAMES" }

            for i in 0..frame_count {
                button {
                    class: if i == active_frame {
                        "w-9 h-9 rounded-lg text-sm font-bold bg-[#ff6188] text-[#1a1a1a] shrink-0"
                    } else {
                        "w-9 h-9 rounded-lg text-sm bg-[#2e2e2e] text-[#9ca0a4] hover:bg-[#383838] shrink-0 border border-[#3c3c3c]"
                    },
                    aria_label: "Frame {i + 1}",
                    aria_pressed: if i == active_frame { "true" } else { "false" },
                    onclick: move |_| app_state.with_mut(|s| s.project.active_frame = i),
                    "{i + 1}"
                }
            }

            div { class: "flex gap-1 shrink-0",
                button {
                    class: "w-8 h-8 rounded-lg bg-[#2e2e2e] text-[#a9dc76] hover:bg-[#383838] text-base font-bold border border-[#3c3c3c]",
                    title: "Add frame",
                    aria_label: "Add frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        s.project.add_frame();
                        s.project.active_frame = s.project.frames.len() - 1;
                    }),
                    "+"
                }
                button {
                    class: "w-8 h-8 rounded-lg bg-[#2e2e2e] text-[#78dce8] hover:bg-[#383838] text-sm border border-[#3c3c3c]",
                    title: "Duplicate frame",
                    aria_label: "Duplicate frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.duplicate_frame(idx);
                        s.project.active_frame = idx + 1;
                    }),
                    "⧉"
                }
                button {
                    class: "w-8 h-8 rounded-lg bg-[#2e2e2e] text-[#9ca0a4] hover:bg-[#383838] text-sm border border-[#3c3c3c]",
                    title: "Move frame left",
                    aria_label: "Move frame left",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_up(idx);
                    }),
                    "←"
                }
                button {
                    class: "w-8 h-8 rounded-lg bg-[#2e2e2e] text-[#9ca0a4] hover:bg-[#383838] text-sm border border-[#3c3c3c]",
                    title: "Move frame right",
                    aria_label: "Move frame right",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_down(idx);
                    }),
                    "→"
                }
                button {
                    class: "w-8 h-8 rounded-lg bg-[#2e2e2e] text-[#ff6188] hover:bg-[#383838] text-sm font-bold border border-[#3c3c3c]",
                    title: "Delete frame",
                    aria_label: "Delete current frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.delete_frame(idx);
                    }),
                    "✕"
                }
            }

            div { class: "flex-1" }

            div { class: "flex items-center gap-2 shrink-0",
                input {
                    r#type: "number",
                    value: "{delay_ms}",
                    min: "50",
                    max: "5000",
                    step: "50",
                    class: "w-16 bg-[#2e2e2e] text-[#fcfcfa] text-xs text-center rounded-lg px-1 py-1 font-mono border border-[#3c3c3c] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[#78dce8]",
                    title: "Frame delay (ms)",
                    aria_label: "Frame delay in milliseconds",
                    onchange: move |evt| {
                        if let Ok(ms) = evt.value().parse::<u32>() {
                            app_state.with_mut(|s| s.playback.delay_ms = ms.max(50));
                        }
                    },
                }
                span { class: "text-[10px] text-[#444444]", "ms" }

                button {
                    class: if playing {
                        "px-3 h-9 rounded-lg bg-[#fc9867] text-[#1a1a1a] text-sm font-bold hover:bg-[#e08856] shrink-0"
                    } else {
                        "px-3 h-9 rounded-lg bg-[#2e2e2e] text-[#78dce8] text-sm font-bold hover:bg-[#383838] shrink-0 border border-[#3c3c3c]"
                    },
                    aria_label: if playing { "Stop playback" } else { "Play animation" },
                    onclick: move |_| {
                        let currently_playing = app_state.read().playback.playing;
                        if currently_playing {
                            app_state.with_mut(|s| s.playback.playing = false);
                            if let Some(id) = *interval_id.read() {
                                if let Some(window) = web_sys::window() {
                                    window.clear_interval_with_handle(id);
                                }
                            }
                            interval_id.set(None);
                            INTERVAL_CB.with(|c| *c.borrow_mut() = None);
                        } else {
                            app_state.with_mut(|s| s.playback.playing = true);
                            let delay = app_state.read().playback.delay_ms as i32;
                            let mut state_ref = app_state;
                            let cb = Closure::wrap(Box::new(move || {
                                let mut s = state_ref.write();
                                if !s.playback.playing { return; }
                                let nframes = s.project.frames.len();
                                s.project.active_frame = (s.project.active_frame + 1) % nframes;
                            }) as Box<dyn FnMut()>);
                            if let Some(window) = web_sys::window() {
                                if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                                    cb.as_ref().unchecked_ref(),
                                    delay,
                                ) {
                                    interval_id.set(Some(id));
                                }
                            }
                            INTERVAL_CB.with(|c| *c.borrow_mut() = Some(cb));
                        }
                    },
                    if playing { "■ Stop" } else { "▶ Play" }
                }
            }
        }
    }
}
