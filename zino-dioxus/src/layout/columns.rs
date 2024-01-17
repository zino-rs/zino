use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// Responsive columns powered by flexbox.
pub fn Columns<'a>(cx: Scope<'a, ColumnsProps<'a>>) -> Element {
    let class = format_class!(cx, "columns");
    let column_class = format_class!(cx, column_class, "column");
    let size_class = if let Some(size) = cx.props.size {
        format!("is-{size}")
    } else {
        String::new()
    };
    let offset_class = if let Some(offset) = cx.props.offset {
        format!("is-offset-{offset}")
    } else {
        String::new()
    };
    let gap_class = if let Some(gap) = cx.props.gap {
        format!("is-variable is-{gap}")
    } else {
        String::new()
    };
    render! {
        div {
            class: "{class} {gap_class}",
            for column in cx.props.columns.iter() {
                div {
                    class: "{column_class} {size_class} {offset_class}",
                    column
                }
            }
        }
    }
}

/// The [`Columns`] properties struct for the configuration of the component.
#[derive(Props)]
pub struct ColumnsProps<'a> {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class<'a>>,
    /// A class to apply to a single column.
    #[props(into)]
    pub column_class: Option<Class<'a>>,
    /// A custom size in the 12 columns system.
    #[props(into)]
    pub size: Option<u8>,
    /// A custom column offset.
    #[props(into)]
    pub offset: Option<u8>,
    /// A custom column gap.
    #[props(into)]
    pub gap: Option<u8>,
    /// The columns to be rendered.
    #[props(into)]
    pub columns: Vec<Element<'a>>,
}
