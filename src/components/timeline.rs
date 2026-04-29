use dioxus::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::state::AppState;

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
            class: "h-14 bg-[#221f22] border-t border-[#403e41] flex items-center px-3 gap-2 shrink-0 overflow-x-auto",

            span { class: "text-[11px] text-[#9ca0a4] tracking-widest uppercase shrink-0", "FRAMES" }

            for i in 0..frame_count {
                button {
                    class: if i == active_frame {
                        "w-9 h-9 rounded text-sm font-bold bg-[#ff6188] text-[#2d2a2e] shrink-0"
                    } else {
                        "w-9 h-9 rounded text-sm bg-[#403e41] text-[#9ca0a4] hover:bg-[#5b595c] shrink-0"
                    },
                    onclick: move |_| app_state.with_mut(|s| s.project.active_frame = i),
                    "{i + 1}"
                }
            }

            div { class: "flex gap-1 shrink-0",
                button {
                    class: "w-8 h-8 rounded bg-[#403e41] text-[#a9dc76] hover:bg-[#5b595c] text-base font-bold",
                    title: "Add frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        s.project.add_frame();
                        s.project.active_frame = s.project.frames.len() - 1;
                    }),
                    "+"
                }
                button {
                    class: "w-8 h-8 rounded bg-[#403e41] text-[#78dce8] hover:bg-[#5b595c] text-sm",
                    title: "Duplicate frame",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.duplicate_frame(idx);
                        s.project.active_frame = idx + 1;
                    }),
                    "⧉"
                }
                button {
                    class: "w-8 h-8 rounded bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c] text-sm",
                    title: "Move frame left",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_up(idx);
                    }),
                    "←"
                }
                button {
                    class: "w-8 h-8 rounded bg-[#403e41] text-[#fcfcfa] hover:bg-[#5b595c] text-sm",
                    title: "Move frame right",
                    onclick: move |_| app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.move_frame_down(idx);
                    }),
                    "→"
                }
                button {
                    class: "w-8 h-8 rounded bg-[#403e41] text-[#ff6188] hover:bg-[#5b595c] text-sm font-bold",
                    title: "Delete frame",
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
                    class: "w-16 bg-[#403e41] text-[#fcfcfa] text-sm text-center rounded px-1 py-1 font-mono border border-[#5b595c] focus:outline-none focus:border-[#78dce8]",
                    title: "Frame delay (ms)",
                    onchange: move |evt| {
                        if let Ok(ms) = evt.value().parse::<u32>() {
                            app_state.with_mut(|s| s.playback.delay_ms = ms.max(50));
                        }
                    },
                }
                span { class: "text-[11px] text-[#9ca0a4]", "ms" }

                button {
                    class: if playing {
                        "px-3 h-9 rounded bg-[#fc9867] text-[#2d2a2e] text-sm font-bold hover:bg-[#e08856] shrink-0"
                    } else {
                        "px-3 h-9 rounded bg-[#403e41] text-[#78dce8] text-sm font-bold hover:bg-[#5b595c] shrink-0"
                    },
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
                            cb.forget();
                        }
                    },
                    if playing { "■ Stop" } else { "▶ Play" }
                }
            }
        }
    }
}
