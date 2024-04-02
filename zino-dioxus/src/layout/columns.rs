use crate::{class::Class, format_class};
use dioxus::prelude::*;

/// Responsive columns powered by flexbox.
pub fn Columns(props: ColumnsProps) -> Element {
    let class = format_class!(props, "columns");
    let column_class = format_class!(props, column_class, "column");
    let size_class = if let Some(size) = props.size {
        format!("is-{size}")
    } else {
        String::new()
    };
    let offset_class = if let Some(offset) = props.offset {
        format!("is-offset-{offset}")
    } else {
        String::new()
    };
    let gap_class = if let Some(gap) = props.gap {
        format!("is-variable is-{gap}")
    } else {
        String::new()
    };
    let multiline_class = Class::check("is-multiline", props.multiline);
    rsx! {
        div {
            class: "{class} {gap_class} {multiline_class}",
            for column in props.columns.iter() {
                div {
                    class: "{column_class} {size_class} {offset_class}",
                    { column }
                }
            }
        }
    }
}

/// The [`Columns`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct ColumnsProps {
    /// The class attribute for the component.
    #[props(into)]
    pub class: Option<Class>,
    /// A class to apply to a single column.
    #[props(into)]
    pub column_class: Option<Class>,
    /// A custom size in the 12 columns system.
    #[props(into)]
    pub size: Option<u8>,
    /// A custom column offset.
    #[props(into)]
    pub offset: Option<u8>,
    /// A custom column gap.
    #[props(into)]
    pub gap: Option<u8>,
    /// A flag to add more column elements than would fit in a single row.
    #[props(default = false)]
    pub multiline: bool,
    /// The columns to be rendered.
    #[props(into)]
    pub columns: Vec<Element>,
}
