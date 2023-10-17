use crate::service;
use dioxus::prelude::*;
use dioxus_free_icons::{icons::bs_icons::*, Icon};
use zino::prelude::*;

pub fn StargazerList(cx: Scope) -> Element {
    let mut page = use_state(cx, || 1);
    let stargazers = use_future(cx, (page,), |(page,)| service::stargazer::list_stargazers(10, *page));
    match stargazers.value() {
        Some(Ok(items)) => {
            let prev_invisible = if **page == 1 { "is-invisible" } else { "" };
            let next_invisible = if items.len() < 10 { "is-invisible" } else { "" };
            render! {
                div {
                    class: "columns is-6",
                    div {
                        class: "column",
                        img {
                            width: 800,
                            height: 533,
                            src: "https://api.star-history.com/svg?repos=photino/zino&type=Timeline",
                        }
                    }
                    div {
                        class: "column",
                        table {
                            class: "table is-fullwidth",
                            thead {
                                tr {
                                    th {}
                                    th { "Account" }
                                    th { "Avatar" }
                                    th { "Starred at" }
                                }
                            }
                            tbody {
                                for (index, item) in items.iter().enumerate() {
                                    StargazerListing {
                                        index: (**page - 1) * 10 + index + 1,
                                        stargazer: item,
                                    }
                                }
                            }
                        }
                        nav {
                            class: "pagination",
                            a {
                                class: "pagination-previous {prev_invisible}",
                                onclick: move |_| {
                                    page -= 1;
                                },
                                Icon {
                                    width: 16,
                                    height: 16,
                                    icon: BsArrowLeftCircleFill,
                                }
                                span {
                                    class: "ml-1",
                                    "Previous"
                                }
                            }
                            a {
                                class: "pagination-next {next_invisible}",
                                onclick: move |_| {
                                    page += 1;
                                },
                                span {
                                    class: "mr-1",
                                    "Next"
                                }
                                Icon {
                                    width: 16,
                                    height: 16,
                                    icon: BsArrowRightCircleFill,
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Err(err)) => {
            render! {
                div {
                    class: "notification is-danger is-light",
                    "An error occurred while fetching stargazers: {err}"
                }
            }
        }
        None => {
            render! {
                progress {
                    class: "progress is-small is-primary",
                    max: 100,
                }
            }
        }
    }
}

#[inline_props]
fn StargazerListing<'a>(cx: Scope<'a>, index: usize, stargazer: &'a Map) -> Element {
    let name = stargazer.get_str("login").unwrap_or_default();
    let avatar_url = stargazer.get_str("avatar_url").unwrap_or_default();
    let starred_at = stargazer.get_str("starred_at").unwrap_or_default();
    render! {
        tr {
            th { "{index}" }
            td {
                a {
                    href: "https://github.com/{name}",
                    "{name}"
                }
            }
            td {
                figure {
                   class: "image is-24x24",
                   img { src: avatar_url },
                }
            }
            td {
                span { "{starred_at}" }
            }
        }
    }
}
