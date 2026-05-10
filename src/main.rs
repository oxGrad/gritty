#![allow(non_snake_case)]
mod canvas;
mod color;
mod components;
mod export;
mod import;
mod state;

use std::cell::RefCell;

use canvas::Canvas;
use components::{
    left_panel::LeftPanel,
    right_panel::RightPanel,
    status_bar::StatusBar,
    top_bar::TopBar,
};
use dioxus::prelude::*;
use state::AppState;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

thread_local! {
    static PLAY_CB: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut app_state = use_context_provider(|| Signal::new(AppState::default()));
    let mut interval_id: Signal<Option<i32>> = use_signal(|| None);

    // Playback interval — restarts whenever fps or playing changes
    use_effect(move || {
        let s = app_state.read();
        let playing = s.playing;
        let fps = s.fps;
        drop(s);

        // Clear existing interval
        let existing_id = *interval_id.peek();
        if let Some(id) = existing_id {
            if let Some(win) = web_sys::window() { win.clear_interval_with_handle(id); }
            interval_id.set(None);
            PLAY_CB.with(|c| *c.borrow_mut() = None);
        }

        if playing {
            let delay = (1000.0 / fps as f64) as i32;
            let mut sr = app_state;
            let cb = Closure::wrap(Box::new(move || {
                let mut s = sr.write();
                if !s.playing { return; }
                let n = s.project.frames.len();
                s.project.active_frame = (s.project.active_frame + 1) % n;
            }) as Box<dyn FnMut()>);
            if let Some(win) = web_sys::window() {
                if let Ok(id) = win.set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(), delay,
                ) {
                    interval_id.set(Some(id));
                }
            }
            PLAY_CB.with(|c| *c.borrow_mut() = Some(cb));
        }
    });

    // Global keyboard shortcuts
    use_effect(move || {
        let cb = Closure::<dyn FnMut(_)>::wrap(Box::new(move |evt: web_sys::KeyboardEvent| {
            if let Some(target) = evt.target() {
                if let Ok(el) = target.dyn_into::<web_sys::HtmlElement>() {
                    let tag = el.tag_name().to_lowercase();
                    if tag == "input" || tag == "textarea" || tag == "select" { return; }
                }
            }
            let k = evt.key().to_lowercase();
            match k.as_str() {
                " " => { evt.prevent_default(); app_state.with_mut(|s| s.playing = !s.playing); }
                "b" => app_state.with_mut(|s| s.tool = state::Tool::Brush),
                "e" => app_state.with_mut(|s| s.tool = state::Tool::Eraser),
                "g" => app_state.with_mut(|s| s.tool = state::Tool::Fill),
                "i" => app_state.with_mut(|s| s.tool = state::Tool::Eyedrop),
                "r" => app_state.with_mut(|s| s.tool = state::Tool::Rect),
                "l" => app_state.with_mut(|s| s.tool = state::Tool::Line),
                "x" => app_state.with_mut(|s| { let fg = s.fg_color; s.fg_color = s.bg_color; s.bg_color = fg; }),
                "n" if !evt.ctrl_key() && !evt.meta_key() => app_state.with_mut(|s| {
                    s.project.add_frame();
                    s.project.active_frame = s.project.frames.len() - 1;
                }),
                "d" if evt.ctrl_key() || evt.meta_key() => {
                    evt.prevent_default();
                    app_state.with_mut(|s| {
                        let idx = s.project.active_frame;
                        s.project.duplicate_frame(idx);
                        s.project.active_frame = idx + 1;
                    });
                }
                "arrowup" if evt.shift_key() => app_state.with_mut(|s| s.project.shift_up()),
                "arrowdown" if evt.shift_key() => app_state.with_mut(|s| s.project.shift_down()),
                "arrowleft" if evt.shift_key() => app_state.with_mut(|s| s.project.shift_left()),
                "arrowright" if evt.shift_key() => app_state.with_mut(|s| s.project.shift_right()),
                "[" => app_state.with_mut(|s| {
                    if s.project.active_frame > 0 { s.project.active_frame -= 1; }
                }),
                "]" => app_state.with_mut(|s| {
                    let n = s.project.frames.len();
                    if s.project.active_frame + 1 < n { s.project.active_frame += 1; }
                }),
                _ => {}
            }
        }));
        web_sys::window().unwrap().document().unwrap()
            .add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget();
    });

    let scanlines = app_state.read().show_scanlines;

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: "app",
            "data-scanlines": if scanlines { "1" } else { "0" },
            TopBar {}
            div { class: "workspace",
                LeftPanel {}
                Canvas {}
                RightPanel {}
            }
            StatusBar {}
        }
    }
}
