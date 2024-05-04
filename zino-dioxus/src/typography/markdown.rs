use crate::class::Class;
use dioxus::prelude::*;
use markdown::{to_html_with_options, Options};
use zino_core::SharedString;

/// A markdown-to-html converter.
pub fn Markdown(props: MarkdownProps) -> Element {
    let class = props.class;
    let text = props.content.as_ref();
    let html = to_html_with_options(text, &Options::gfm()).unwrap_or_else(|_| text.to_owned());
    rsx! {
        div {
            class: "{class}",
            dangerous_inner_html: "{html}",
        }
    }
}

/// The [`Markdown`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct MarkdownProps {
    /// The class attribute for the component.
    #[props(into, default = "markdown content".into())]
    pub class: Class,
    /// The children to render within the component.
    #[props(into)]
    pub content: SharedString,
}
