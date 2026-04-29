#![allow(non_snake_case)]
mod canvas;
mod color;
mod components;
mod export;
mod import;
mod state;

use canvas::Canvas;
use components::left_panel::LeftPanel;
use components::right_panel::RightPanel;
use components::top_bar::TopBar;
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
            TopBar {}
            div { class: "flex flex-1 overflow-hidden",
                LeftPanel {}
                Canvas {}
                RightPanel {}
            }
            div { class: "h-12 bg-[#221f22] border-t border-[#403e41]" }
        }
    }
}
