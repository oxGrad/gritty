use dioxus::prelude::*;
use js_sys::Array;
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

    let (w, h) = {
        let s = app_state.read();
        (s.project.width, s.project.height)
    };

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
        div {
            class: "h-10 bg-[#221f22] border-b border-[#403e41] flex items-center px-3 gap-4 shrink-0",

            span { class: "text-sm font-bold tracking-widest text-[#fcfcfa]", "GRITTY" }
            span { class: "text-xs text-[#9ca0a4] font-mono", "{w}×{h}" }
            div { class: "flex-1" }

            label {
                class: "text-xs bg-[#403e41] hover:bg-[#5b595c] text-[#a9dc76] px-2 py-1 rounded cursor-pointer",
                r#for: "file-import",
                "Import"
            }
            input {
                id: "file-import",
                r#type: "file",
                accept: ".json,.ans,.txt",
                class: "hidden",
                onchange: move |_| {
                    let document = web_sys::window().unwrap().document().unwrap();
                    let input = document.get_element_by_id("file-import").unwrap()
                        .dyn_into::<HtmlInputElement>().unwrap();
                    if let Some(files) = input.files() {
                        if let Some(file) = files.item(0) {
                            let name = file.name();
                            let is_json = name.ends_with(".json");
                            let w = app_state.read().project.width;
                            let h = app_state.read().project.height;
                            let text_promise = file.text();
                            let mut state_ref = app_state;
                            wasm_bindgen_futures::spawn_local(async move {
                                if let Ok(text_js) = wasm_bindgen_futures::JsFuture::from(text_promise).await {
                                    if let Some(text) = text_js.as_string() {
                                        if is_json {
                                            if let Ok(project) = import_json(&text) {
                                                state_ref.with_mut(|s| s.project = project);
                                            }
                                        } else {
                                            let project = import_ansi(&text, w, h);
                                            state_ref.with_mut(|s| s.project = project);
                                        }
                                    }
                                }
                            });
                        }
                    }
                },
            }

            button {
                class: "text-xs bg-[#ff6188] hover:bg-[#e05078] text-[#2d2a2e] font-bold px-2 py-1 rounded",
                onclick: move |_| export_open.set(true),
                "Export"
            }

            if *export_open.read() {
                div {
                    class: "fixed inset-0 flex items-center justify-center z-50",
                    style: "background: rgba(0,0,0,0.6);",
                    onclick: move |_| export_open.set(false),
                    div {
                        class: "bg-[#2d2a2e] border border-[#403e41] rounded-lg w-[560px] max-h-[80vh] flex flex-col overflow-hidden",
                        onclick: move |evt| evt.stop_propagation(),

                        div { class: "flex items-center border-b border-[#403e41]",
                            for (i, label) in ["ANSI", "Plain+ANSI", "JSON"].iter().enumerate() {
                                button {
                                    class: if *export_tab.read() == i {
                                        "px-4 py-2 text-xs font-bold text-[#ff6188] border-b-2 border-[#ff6188]"
                                    } else {
                                        "px-4 py-2 text-xs text-[#9ca0a4] hover:text-[#fcfcfa]"
                                    },
                                    onclick: move |_| export_tab.set(i),
                                    "{label}"
                                }
                            }
                            div { class: "flex-1" }
                            button {
                                class: "px-3 py-2 text-[#9ca0a4] hover:text-[#fcfcfa] text-sm",
                                onclick: move |_| export_open.set(false),
                                "✕"
                            }
                        }

                        if *export_tab.read() < 2 {
                            div { class: "flex items-center gap-2 px-4 py-2 border-b border-[#403e41]",
                                span { class: "text-xs text-[#9ca0a4]", "Scope:" }
                                button {
                                    class: if !*all_frames.read() {
                                        "text-xs px-2 py-0.5 rounded bg-[#ff6188] text-[#2d2a2e] font-bold"
                                    } else {
                                        "text-xs px-2 py-0.5 rounded bg-[#403e41] text-[#fcfcfa]"
                                    },
                                    onclick: move |_| all_frames.set(false),
                                    "Current frame"
                                }
                                button {
                                    class: if *all_frames.read() {
                                        "text-xs px-2 py-0.5 rounded bg-[#ff6188] text-[#2d2a2e] font-bold"
                                    } else {
                                        "text-xs px-2 py-0.5 rounded bg-[#403e41] text-[#fcfcfa]"
                                    },
                                    onclick: move |_| all_frames.set(true),
                                    "All frames"
                                }
                            }
                        }

                        if *export_tab.read() < 2 {
                            div { class: "flex flex-col flex-1 overflow-hidden p-3 gap-2",
                                pre {
                                    class: "flex-1 overflow-auto text-xs font-mono text-[#fcfcfa] bg-[#19181a] p-3 rounded border border-[#403e41] whitespace-pre",
                                    "{export_text}"
                                }
                                button {
                                    class: "self-end text-xs bg-[#403e41] hover:bg-[#5b595c] text-[#78dce8] px-3 py-1 rounded",
                                    onclick: {
                                        let text = export_text.clone();
                                        move |_| {
                                            if let Some(window) = web_sys::window() {
                                            let _ = window.navigator().clipboard().write_text(&text);
                                        }
                                        }
                                    },
                                    "Copy"
                                }
                            }
                        } else {
                            div { class: "flex flex-col items-center justify-center flex-1 gap-4 p-6",
                                p { class: "text-sm text-[#9ca0a4]", "Downloads the full project (all frames) as .json" }
                                button {
                                    class: "bg-[#ff6188] text-[#2d2a2e] font-bold px-6 py-2 rounded hover:bg-[#e05078]",
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
}

fn download_string(content: &str, filename: &str, mime: &str) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let array = Array::new();
    array.push(&wasm_bindgen::JsValue::from_str(content));
    let mut opts = BlobPropertyBag::new();
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
