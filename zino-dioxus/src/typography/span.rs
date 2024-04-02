use crate::{class::Class, format_class};
use dioxus::prelude::*;
use zino_core::{JsonValue, SharedString};

/// A fixed-width span with the custom alignment.
pub fn FixedWidthSpan(props: FixedWidthSpanProps) -> Element {
    let class = format_class!(props, "is-inline-block");
    let mut style = match props.width {
        JsonValue::Number(value) => format!("width:{value}px;"),
        JsonValue::String(value) => format!("width:{value};"),
        _ => String::new(),
    };
    if let Some(alignment) = props.align.as_deref() {
        style.push_str("text-align:");
        style.push_str(alignment);
    }
    rsx! {
        span {
            class: "{class}",
            style: "{style}",
            { props.children }
        }
    }
}

/// The [`FixedWidthSpan`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct FixedWidthSpanProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// The width of the span.
    #[props(into)]
    pub width: JsonValue,
    /// The `text-align` CSS property: `left` | `right` | `center` | `justify`.
    #[props(into)]
    pub align: Option<SharedString>,
    /// The children to render within the component.
    children: Element,
}
