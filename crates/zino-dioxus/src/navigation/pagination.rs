use crate::{class::Class, icon::SvgIcon};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaArrowLeft, FaArrowRight};
use zino_core::SharedString;

/// A vertical menu used in the navigation aside.
pub fn Pagination(props: PaginationProps) -> Element {
    let total = props.total;
    let page_size = props.page_size.max(1);
    let current_page = props.current_page.max(1);
    let page_count = total.div_ceil(page_size);
    if total == 0 || page_count <= 1 {
        return rsx! {};
    }
    rsx! {
        nav {
            class: "{props.class} is-centered",
            a {
                class: "pagination-previous",
                class: if current_page == 1 || page_count <= 5 { "is-invisible" },
                onclick: move |_| {
                    if let Some(handler) = props.on_change.as_ref() {
                        handler.call(current_page - 1);
                    }
                },
                if let Some(prev) = props.prev {
                    { prev }
                } else {
                    SvgIcon {
                        shape: FaArrowLeft,
                        width: 16,
                    }
                    span {
                        class: "ml-1",
                        { props.prev_text }
                    }
                }
            }
            ul {
                class: "pagination-list",
                if current_page > 2 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(1);
                                }
                            },
                            "1"
                        }
                    }
                }
                if current_page > 3 && page_count > 5 {
                    li {
                        span {
                            class: "pagination-ellipsis",
                            "…"
                        }
                    }
                }
                if current_page > 4 && page_count < current_page + 1 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page - 3);
                                }
                            },
                            "{current_page - 3}"
                        }
                    }
                }
                if current_page > 3 && page_count < current_page + 2 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page - 2);
                                }
                            },
                            "{current_page - 2}"
                        }
                    }
                }
                if current_page > 1 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page - 1);
                                }
                            },
                            "{current_page - 1}"
                        }
                    }
                }
                li {
                    a {
                        class: "pagination-link is-current",
                        onclick: move |_| {
                            if let Some(handler) = props.on_change.as_ref() {
                                handler.call(current_page);
                            }
                        },
                        "{current_page}"
                    }
                }
                if current_page < page_count {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page + 1);
                                }
                            },
                            "{current_page + 1}"
                        }
                    }
                }
                if current_page < 3 && page_count > current_page + 2 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page + 2);
                                }
                            },
                            "{current_page + 2}"
                        }
                    }
                }
                if current_page < 2 && page_count > current_page + 3 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(current_page + 3);
                                }
                            },
                            "{current_page + 3}"
                        }
                    }
                }
                if page_count > current_page + 2 && page_count > 5 {
                    li {
                        span {
                            class: "pagination-ellipsis",
                            "…"
                        }
                    }
                }
                if page_count > current_page + 1 {
                    li {
                        a {
                            class: "pagination-link",
                            onclick: move |_| {
                                if let Some(handler) = props.on_change.as_ref() {
                                    handler.call(page_count);
                                }
                            },
                            "{page_count}"
                        }
                    }
                }
            }
            a {
                class: "pagination-next",
                class: if total <= current_page * page_size || page_count <= 5 { "is-invisible" },
                onclick: move |_| {
                    if let Some(handler) = props.on_change.as_ref() {
                        handler.call(current_page + 1);
                    }
                },
                if let Some(next) = props.next {
                    { next }
                } else {
                    span {
                        class: "mr-1",
                        { props.next_text }
                    }
                    SvgIcon {
                        shape: FaArrowRight,
                        width: 16,
                    }
                }
            }
        }
    }
}

/// The [`Pagination`] properties struct for the configuration of the component.
#[derive(Clone, PartialEq, Props)]
pub struct PaginationProps {
    /// The class attribute for the component.
    #[props(into, default = "pagination")]
    pub class: Class,
    /// Total number of data items.
    pub total: usize,
    /// Number of data items per page.
    #[props(default = 10)]
    pub page_size: usize,
    /// The current page number.
    pub current_page: usize,
    /// The element for the previous button.
    #[props(into, default = "Previous")]
    pub prev_text: SharedString,
    /// The text for the next button.
    #[props(into, default = "Next")]
    pub next_text: SharedString,
    /// The element for the previous button.
    pub prev: Option<Element>,
    /// The element for the next button.
    pub next: Option<Element>,
    /// An event handler to be called when the page number is changed.
    pub on_change: Option<EventHandler<usize>>,
}
