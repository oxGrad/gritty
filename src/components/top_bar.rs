use js_sys::Array;
use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlInputElement, Url};

use crate::export::{export_ansi_all, export_ansi_frame, export_json};
use crate::import::{import_ansi, import_json};
use crate::state::AppState;

#[component]
pub fn TopBar() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut export_open = use_signal(|| false);
    let mut export_tab = use_signal(|| 0usize);
    let mut all_frames = use_signal(|| false);

    let (w, h, frame_count, active_idx, fps, playing, zoom) = {
        let s = app_state.read();
        (
            s.project.width, s.project.height,
            s.project.frames.len(), s.project.active_frame,
            s.fps, s.playing, s.zoom,
        )
    };

    let accent_css = "#39ff14";

    let zoom_pct = (zoom * 100.0).round() as i32;

    let export_text = {
        let s = app_state.read();
        let tab = *export_tab.read();
        let all = *all_frames.read();
        match tab {
            0 | 1 => {
                if all { export_ansi_all(&s.project) }
                else { export_ansi_frame(&s.project.frames[s.project.active_frame], s.project.width, s.project.height) }
            }
            _ => String::new()
        }
    };

    rsx! {
        header { class: "toolbar",
            // Left: wordmark + file info
            div { class: "toolbar-left",
                div { class: "wordmark",
                    span { class: "wordmark-bracket", style: "color: {accent_css}", "▮" }
                    span { class: "wordmark-name", "Gritty" }
                    span { class: "wordmark-cursor", style: "background: {accent_css}" }
                }
                span { class: "dot" }
                button { class: "tb-btn ghost", "⊞ Untitled.ans" }
                span { class: "dot" }
                span { class: "tb-meta mono",
                    "{w}"
                    span { class: "mute", "×" }
                    "{h} cells"
                }
            }

            // Center: playback
            div { class: "toolbar-center",
                div { class: "playback",
                    button {
                        class: "tb-btn icon",
                        title: "First frame",
                        onclick: move |_| app_state.with_mut(|s| s.project.active_frame = 0),
                        "⏮"
                    }
                    button {
                        class: "tb-btn icon",
                        title: "Previous frame ([)",
                        onclick: move |_| app_state.with_mut(|s| {
                            if s.project.active_frame > 0 { s.project.active_frame -= 1; }
                        }),
                        "◀"
                    }
                    button {
                        class: "tb-btn icon play",
                        title: "Play / Pause (Space)",
                        "data-active": if playing { "1" } else { "0" },
                        onclick: move |_| app_state.with_mut(|s| s.playing = !s.playing),
                        if playing { "⏸" } else { "▶" }
                    }
                    button {
                        class: "tb-btn icon",
                        title: "Next frame (])",
                        onclick: move |_| app_state.with_mut(|s| {
                            let n = s.project.frames.len();
                            if s.project.active_frame + 1 < n { s.project.active_frame += 1; }
                        }),
                        "▶"
                    }
                    button {
                        class: "tb-btn icon",
                        title: "Last frame",
                        onclick: move |_| app_state.with_mut(|s| {
                            s.project.active_frame = s.project.frames.len() - 1;
                        }),
                        "⏭"
                    }
                }

                div { class: "fps-input",
                    input {
                        r#type: "number",
                        min: "1",
                        max: "60",
                        value: "{fps}",
                        onchange: move |evt| {
                            if let Ok(v) = evt.value().parse::<u32>() {
                                app_state.with_mut(|s| s.fps = v.clamp(1, 60));
                            }
                        },
                    }
                    span { class: "fps-label", "fps" }
                }

                span { class: "tb-meta mono",
                    "F{active_idx + 1}/{frame_count}"
                }
            }

            // Right: zoom + import/export
            div { class: "toolbar-right",
                div { class: "zoom-grp",
                    button {
                        class: "tb-btn icon",
                        title: "Zoom out",
                        onclick: move |_| app_state.with_mut(|s| s.zoom = (s.zoom / 1.25).clamp(0.25, 8.0)),
                        "−"
                    }
                    span { class: "zoom-val mono", "{zoom_pct}%" }
                    button {
                        class: "tb-btn icon",
                        title: "Zoom in",
                        onclick: move |_| app_state.with_mut(|s| s.zoom = (s.zoom * 1.25).clamp(0.25, 8.0)),
                        "+"
                    }
                }
                span { class: "dot" }

                // Import
                label {
                    class: "tb-btn outlined",
                    style: "cursor: pointer;",
                    r#for: "file-import-tb",
                    "↧ Import"
                }
                input {
                    id: "file-import-tb",
                    r#type: "file",
                    accept: ".json,.ans,.txt",
                    style: "display: none;",
                    onchange: move |_| {
                        let document = web_sys::window().unwrap().document().unwrap();
                        let input = document.get_element_by_id("file-import-tb").unwrap()
                            .dyn_into::<HtmlInputElement>().unwrap();
                        if let Some(files) = input.files() {
                            if let Some(file) = files.item(0) {
                                let name = file.name();
                                let is_json = name.ends_with(".json");
                                let w = app_state.read().project.width;
                                let h = app_state.read().project.height;
                                let text_promise = file.text();
                                let mut sr = app_state;
                                wasm_bindgen_futures::spawn_local(async move {
                                    if let Ok(txt_js) = wasm_bindgen_futures::JsFuture::from(text_promise).await {
                                        if let Some(text) = txt_js.as_string() {
                                            if is_json {
                                                if let Ok(project) = import_json(&text) {
                                                    sr.with_mut(|s| s.project = project);
                                                }
                                            } else {
                                                let project = import_ansi(&text, w, h);
                                                sr.with_mut(|s| s.project = project);
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    },
                }

                button {
                    class: "tb-btn primary",
                    style: "background: {accent_css}; color: #000;",
                    onclick: move |_| export_open.set(true),
                    "Export"
                }
            }
        }

        // Export modal
        if *export_open.read() {
            div {
                style: "position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 100;",
                onclick: move |_| export_open.set(false),
                div {
                    style: "background: var(--bg-2); border: 1px solid var(--line); border-radius: 12px; width: 560px; max-height: 80vh; display: flex; flex-direction: column; overflow: hidden;",
                    onclick: move |evt| evt.stop_propagation(),
                    onkeydown: move |evt| { if evt.key() == Key::Escape { export_open.set(false); } },

                    // Tab bar
                    div { style: "display: flex; align-items: center; border-bottom: 1px solid var(--line);",
                        for (i, label) in ["ANSI", "JSON"].iter().enumerate() {
                            button {
                                style: if *export_tab.read() == i {
                                    "padding: 10px 16px; font-size: 12px; font-weight: 700; color: var(--accent); border-bottom: 2px solid var(--accent); background: none; border-top: none; border-left: none; border-right: none; cursor: pointer;"
                                } else {
                                    "padding: 10px 16px; font-size: 12px; color: var(--fg-mute); background: none; border: none; cursor: pointer;"
                                },
                                onclick: move |_| export_tab.set(i),
                                "{label}"
                            }
                        }
                        div { style: "flex: 1;" }
                        if *export_tab.read() == 0 {
                            div { style: "display: flex; gap: 6px; padding: 0 12px; align-items: center;",
                                span { style: "font-size: 11px; color: var(--fg-mute);", "Scope:" }
                                button {
                                    style: if !*all_frames.read() {
                                        "font-size: 11px; padding: 4px 8px; border-radius: 4px; background: var(--accent); color: #000; font-weight: 700; border: none; cursor: pointer;"
                                    } else {
                                        "font-size: 11px; padding: 4px 8px; border-radius: 4px; background: var(--bg-3); color: var(--fg); border: none; cursor: pointer;"
                                    },
                                    onclick: move |_| all_frames.set(false),
                                    "Current"
                                }
                                button {
                                    style: if *all_frames.read() {
                                        "font-size: 11px; padding: 4px 8px; border-radius: 4px; background: var(--accent); color: #000; font-weight: 700; border: none; cursor: pointer;"
                                    } else {
                                        "font-size: 11px; padding: 4px 8px; border-radius: 4px; background: var(--bg-3); color: var(--fg); border: none; cursor: pointer;"
                                    },
                                    onclick: move |_| all_frames.set(true),
                                    "All"
                                }
                            }
                        }
                        button {
                            style: "padding: 8px 12px; color: var(--fg-mute); background: none; border: none; font-size: 16px; cursor: pointer;",
                            onclick: move |_| export_open.set(false),
                            "✕"
                        }
                    }

                    // Content
                    if *export_tab.read() == 0 {
                        div { style: "display: flex; flex-direction: column; flex: 1; overflow: hidden; padding: 12px; gap: 8px;",
                            pre {
                                style: "flex: 1; overflow: auto; font-size: 12px; font-family: var(--font-mono); color: var(--fg); background: var(--bg-1); padding: 12px; border-radius: 8px; border: 1px solid var(--line); white-space: pre; min-height: 200px;",
                                "{export_text}"
                            }
                            button {
                                style: "align-self: flex-end; font-size: 12px; padding: 6px 12px; border-radius: 6px; background: var(--bg-3); color: var(--fg); border: 1px solid var(--line); cursor: pointer;",
                                onclick: {
                                    let text = export_text.clone();
                                    move |_| {
                                        if let Some(win) = web_sys::window() {
                                            let _ = win.navigator().clipboard().write_text(&text);
                                        }
                                    }
                                },
                                "Copy"
                            }
                        }
                    } else {
                        div { style: "display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1; gap: 16px; padding: 24px;",
                            p { style: "font-size: 13px; color: var(--fg-mute);", "Downloads full project (all frames) as .json" }
                            button {
                                style: "background: var(--accent); color: #000; font-weight: 700; font-size: 13px; padding: 8px 24px; border-radius: 8px; border: none; cursor: pointer;",
                                onclick: move |_| {
                                    let json = export_json(&app_state.read().project);
                                    download_string(&json, "gritty-project.json", "application/json");
                                },
                                "Download .json"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn download_string(content: &str, filename: &str, mime: &str) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let array = Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(content));
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let Ok(blob) = Blob::new_with_str_sequence_and_options(&array, &opts) else { return };
    let Ok(url) = Url::create_object_url_with_blob(&blob) else { return };
    if let Ok(el) = document.create_element("a") {
        if let Ok(a) = el.dyn_into::<HtmlAnchorElement>() {
            a.set_href(&url);
            a.set_download(filename);
            if let Some(body) = document.body() {
                let _ = body.append_child(&a);
                a.click();
                let _ = body.remove_child(&a);
            }
        }
    }
    let _ = Url::revoke_object_url(&url);
}
