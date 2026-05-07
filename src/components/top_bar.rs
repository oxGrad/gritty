use dioxus::prelude::*;
use js_sys::Array;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, HtmlInputElement, Url};

use crate::export::{export_ansi_all, export_ansi_frame, export_json};
use crate::import::{import_ansi, import_json};
use crate::state::AppState;

#[component]
pub fn TopBar() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut export_open = use_signal(|| false);
    let mut export_tab = use_signal(|| 0usize);
    let mut all_frames = use_signal(|| false);

    use_effect(move || {
        if *export_open.read() {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("export-dialog") {
                    if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                        let _ = html_el.focus();
                    }
                }
            }
        }
    });

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
            class: "fixed top-0 left-0 right-0 h-12 bg-[#252525]/95 border-b border-[#333333] flex items-center px-4 gap-3 z-20",

            span { class: "text-sm font-bold tracking-widest text-[#ff6188]", "GRITTY" }
            span { class: "text-xs text-[#444444] font-mono", "{w}×{h}" }

            div { class: "flex-1" }

            label {
                class: "text-xs bg-[#2e2e2e] hover:bg-[#383838] text-[#a9dc76] px-3 py-1.5 rounded-lg cursor-pointer border border-[#3c3c3c]",
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
                class: "text-xs bg-[#ff6188] hover:bg-[#e05078] text-[#1a1a1a] font-bold px-3 py-1.5 rounded-lg",
                onclick: move |_| export_open.set(true),
                "Export"
            }

            if *export_open.read() {
                div {
                    class: "fixed inset-0 flex items-center justify-center z-50",
                    style: "background: rgba(0,0,0,0.7);",
                    onclick: move |_| export_open.set(false),
                    div {
                        id: "export-dialog",
                        role: "dialog",
                        aria_modal: "true",
                        aria_label: "Export",
                        tabindex: "-1",
                        class: "bg-[#252525] border border-[#3c3c3c] rounded-2xl w-[560px] max-h-[80vh] flex flex-col overflow-hidden focus:outline-none shadow-2xl",
                        onclick: move |evt| evt.stop_propagation(),
                        onkeydown: move |evt| {
                            if evt.key() == Key::Escape {
                                export_open.set(false);
                            }
                        },

                        div { class: "flex items-center border-b border-[#333333]",
                            for (i, label) in ["ANSI", "Plain+ANSI", "JSON"].iter().enumerate() {
                                button {
                                    class: if *export_tab.read() == i {
                                        "px-4 py-2.5 text-sm font-bold text-[#ff6188] border-b-2 border-[#ff6188]"
                                    } else {
                                        "px-4 py-2.5 text-sm text-[#5b595c] hover:text-[#fcfcfa]"
                                    },
                                    onclick: move |_| export_tab.set(i),
                                    "{label}"
                                }
                            }
                            div { class: "flex-1" }
                            button {
                                class: "px-3 py-2 text-[#5b595c] hover:text-[#fcfcfa] text-base",
                                aria_label: "Close export dialog",
                                onclick: move |_| export_open.set(false),
                                "✕"
                            }
                        }

                        if *export_tab.read() < 2 {
                            div { class: "flex items-center gap-2 px-4 py-2 border-b border-[#333333]",
                                span { class: "text-xs text-[#5b595c]", "Scope:" }
                                button {
                                    class: if !*all_frames.read() {
                                        "text-xs px-2 py-1 rounded-lg bg-[#ff6188] text-[#1a1a1a] font-bold"
                                    } else {
                                        "text-xs px-2 py-1 rounded-lg bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]"
                                    },
                                    onclick: move |_| all_frames.set(false),
                                    "Current frame"
                                }
                                button {
                                    class: if *all_frames.read() {
                                        "text-xs px-2 py-1 rounded-lg bg-[#ff6188] text-[#1a1a1a] font-bold"
                                    } else {
                                        "text-xs px-2 py-1 rounded-lg bg-[#2e2e2e] text-[#fcfcfa] hover:bg-[#383838]"
                                    },
                                    onclick: move |_| all_frames.set(true),
                                    "All frames"
                                }
                            }
                        }

                        if *export_tab.read() < 2 {
                            div { class: "flex flex-col flex-1 overflow-hidden p-3 gap-2",
                                pre {
                                    class: "flex-1 overflow-auto text-sm font-mono text-[#fcfcfa] bg-[#1a1a1a] p-3 rounded-xl border border-[#333333] whitespace-pre",
                                    "{export_text}"
                                }
                                button {
                                    class: "self-end text-xs bg-[#2e2e2e] hover:bg-[#383838] text-[#78dce8] px-3 py-1.5 rounded-lg border border-[#3c3c3c]",
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
                                p { class: "text-sm text-[#5b595c]", "Downloads the full project (all frames) as .json" }
                                button {
                                    class: "bg-[#ff6188] text-[#1a1a1a] font-bold text-sm px-6 py-2 rounded-lg hover:bg-[#e05078]",
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
