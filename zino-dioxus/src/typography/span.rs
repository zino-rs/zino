use crate::class::Class;
use dioxus::prelude::*;
use zino_core::{JsonValue, SharedString};

/// A fixed-width span with the custom alignment.
pub fn FixedWidthSpan(props: FixedWidthSpanProps) -> Element {
    let mut style = match props.width {
        JsonValue::Number(value) => format!("width:{value}px;"),
        JsonValue::String(value) => format!("width:{value};"),
        _ => String::new(),
    };
    style.push_str("text-align:");
    style.push_str(props.align.as_ref());
    rsx! {
        span {
            class: props.class,
            style: style,
            { props.children }
        }
    }
}

/// The [`FixedWidthSpan`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FixedWidthSpanProps {
    /// The class attribute for the component.
    #[props(into, default = "is-inline-block")]
    pub class: Class,
    /// The width of the span.
    #[props(into)]
    pub width: JsonValue,
    /// The `text-align` CSS property: `left` | `right` | `center` | `justify`.
    #[props(into, default = "left")]
    pub align: SharedString,
    /// The children to render within the component.
    children: Element,
}
