#![allow(non_snake_case)]
mod canvas;
mod color;
mod components;
mod export;
mod import;
mod state;

use canvas::Canvas;
use components::{
    left_panel::LeftPanel,
    right_panel::RightPanel,
    timeline::Timeline,
    top_bar::TopBar,
};
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
            class: "w-screen h-screen overflow-hidden relative select-none",
            Canvas {}
            TopBar {}
            LeftPanel {}
            RightPanel {}
            Timeline {}
        }
    }
}
