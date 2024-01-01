use crate::service;
use dioxus::prelude::*;
use dioxus_free_icons::{icons::bs_icons::*, Icon};
use zino::prelude::*;

pub fn StargazerList(cx: Scope) -> Element {
    let stargazers_count = use_future(cx, (), |_| service::stargazer::get_stargazers_count());
    match stargazers_count.value() {
        Some(Ok(count)) => {
            render! {
                div {
                    class: "columns is-desktop is-6",
                    div {
                        class: "column",
                        StargazerHistory {}
                    }
                    div {
                        class: "column",
                        StargazerPaginate { num_stargazers: *count }
                    }
                }
            }
        }
        Some(Err(err)) => {
            render! {
                div {
                    class: "notification is-danger is-light",
                    "An error occurred while fetching stargazers count: {err}"
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

fn StargazerHistory(cx: Scope) -> Element {
    let chart_type = use_state(cx, || "Date");
    render! {
        label {
            class: "checkbox is-pulled-right",
            input {
                r#type: "checkbox",
                onchange: |event| {
                    let value = if event.value == "true" { "Timeline" } else { "Date" };
                    chart_type.set(value);
                },
            }
            span {
                class: "ml-1",
                "Align timeline"
            }
        }
        img {
            margin_top: "-30px",
            width: 800,
            height: 533,
            src: "https://api.star-history.com/svg?repos=photino/zino&type={chart_type}",
        }
    }
}

#[component]
fn StargazerPaginate(cx: Scope, num_stargazers: usize) -> Element {
    let mut page = use_state(cx, || 1);
    let stargazers = use_future(cx, (page,), |(page,)| {
        service::stargazer::list_stargazers(10, *page)
    });
    match stargazers.value() {
        Some(Ok(stargazers)) => {
            let total_pages = num_stargazers.div_ceil(10);
            let prev_invisible = if **page == 1 { "is-invisible" } else { "" };
            let next_invisible = if stargazers.len() < 10 {
                "is-invisible"
            } else {
                ""
            };
            render! {
                table {
                    class: "table is-fullwidth",
                    thead {
                        tr {
                            th {}
                            th { "Username" }
                            th { "Avatar" }
                            th { "Followers" }
                            th { "Starred at" }
                        }
                    }
                    tbody {
                        for (index, stargazer) in stargazers.iter().enumerate() {
                            StargazerListing {
                                index: (**page - 1) * 10 + index + 1,
                                stargazer: stargazer,
                            }
                        }
                    }
                }
                nav {
                    class: "pagination is-centered",
                    a {
                        class: "pagination-previous {prev_invisible}",
                        onclick: move |_| {
                            page -= 1;
                        },
                        Icon {
                            width: 16,
                            height: 16,
                            icon: BsArrowLeft,
                        }
                        span {
                            class: "ml-1",
                            "Previous"
                        }
                    }
                    ul {
                        class: "pagination-list",
                        if **page > 2 {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page.set(1);
                                        },
                                        "1"
                                    }
                                }
                            )
                        }
                        if **page > 3 {
                            rsx!(
                                li {
                                    span {
                                        class: "pagination-ellipsis",
                                        "…"
                                    }
                                }
                            )
                        }
                        if total_pages < page + 1 {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page -= 3;
                                        },
                                        "{page - 3}"
                                    }
                                }
                            )
                        }
                        if total_pages < page + 2 {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page -= 2;
                                        },
                                        "{page - 2}"
                                    }
                                }
                            )
                        }
                        if **page > 1 {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page -= 1;
                                        },
                                        "{page - 1}"
                                    }
                                }
                            )
                        }
                        li {
                            a {
                                class: "pagination-link is-current",
                                "{page}"
                            }
                        }
                        if **page < total_pages {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page += 1;
                                        },
                                        "{page + 1}"
                                    }
                                }
                            )
                        }
                        if **page < 3 {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page += 2;
                                        },
                                        "{page + 2}"
                                    }
                                }
                            )
                        }
                        if **page < 2 {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page += 3;
                                        },
                                        "{page + 3}"
                                    }
                                }
                            )
                        }
                        if total_pages > page + 2 {
                            rsx!(
                                li {
                                    span {
                                        class: "pagination-ellipsis",
                                        "…"
                                    }
                                }
                            )
                        }
                        if total_pages > page + 1 {
                            rsx!(
                                li {
                                    a {
                                        class: "pagination-link",
                                        onclick: move |_| {
                                            page.set(total_pages);
                                        },
                                        "{total_pages}"
                                    }
                                }
                            )
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
                            icon: BsArrowRight,
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

#[component]
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
                a {
                    href: "https://github.com/{name}?tab=followers",
                    img {
                        src: "https://img.shields.io/github/followers/{name}?label=&style=social"
                    }
                }
            }
            td {
                span { "{starred_at}" }
            }
        }
    }
}
