#![allow(non_snake_case)]
mod canvas;
mod color;
mod components;
mod export;
mod import;
mod state;

use canvas::Canvas;
use components::left_panel::LeftPanel;
use dioxus::prelude::*;
use state::AppState;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(AppState::default()));

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div {
            class: "flex flex-col h-screen bg-[#2d2a2e] text-[#fcfcfa]",
            div { class: "h-10 bg-[#221f22] border-b border-[#403e41] flex items-center px-3",
                span { class: "text-sm font-bold tracking-widest", "GRITTY" }
            }
            div { class: "flex flex-1 overflow-hidden",
                LeftPanel {}
                Canvas {}
                div { class: "w-16 bg-[#221f22] border-l border-[#403e41]" }
            }
            div { class: "h-12 bg-[#221f22] border-t border-[#403e41]" }
        }
    }
}
