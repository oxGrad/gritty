#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div { class: "flex flex-col h-screen",
            p { "Gritty loading..." }
        }
    }
}
