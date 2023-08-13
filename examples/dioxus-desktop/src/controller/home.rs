use dioxus::prelude::*;

pub fn Homepage(cx: Scope) -> Element {
    render! {
        h3 { "Congratulations, you have booted your application successfully!" }
    }
}
