use crate::{class::Class, format_class};
use dioxus::prelude::*;
use std::borrow::Cow;
use zino_core::JsonValue;

/// A fixed-width span with the custom alignment.
pub fn FixedWidthSpan<'a>(cx: Scope<'a, FixedWidthSpanProps<'a>>) -> Element {
    let class = format_class!(cx, "is-inline-block");
    let mut style = match &cx.props.width {
        JsonValue::Number(value) => format!("width:{value}px;"),
        JsonValue::String(value) => format!("width:{value};"),
        _ => String::new(),
    };
    if let Some(alignment) = cx.props.align.as_deref() {
        style.push_str("text-align:");
        style.push_str(alignment);
    }
    render! {
        span {
            class: "{class}",
            style: "{style}",
            &cx.props.children
        }
    }
}

/// The [`FixedWidthSpan`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct FixedWidthSpanProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// The width of the span.
    #[props(into)]
    pub width: JsonValue,
    /// The text alignment in one of `left`, `right`, `center` and `justify`.
    #[props(into)]
    pub align: Option<Cow<'a, str>>,
    /// The children to render within the component.
    children: Element<'a>,
}
