use crate::router::Route;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{bs_icons::*, fa_brands_icons::FaRust, fa_solid_icons::FaCubes},
    Icon,
};
use dioxus_router::prelude::*;

pub fn Wrapper(cx: Scope) -> Element {
    let nav_item_classes = use_state(cx, || ["is-active", ""]);
    render! {
        nav {
            class: "navbar is-link",
            div {
                class: "navbar-menu is-active",
                div {
                    class: "navbar-start",
                    Link {
                        class: "navbar-item {nav_item_classes[0]}",
                        to: Route::Overview {},
                        onclick: move |_| {
                            nav_item_classes.set(["is-active", ""]);
                        },
                        Icon {
                            width: 16,
                            height: 16,
                            icon: BsSpeedometer2,
                        }
                        span {
                            class: "ml-1",
                            "Overview"
                        }
                    }
                    Link {
                        class: "navbar-item {nav_item_classes[1]}",
                        to: Route::StargazerList {},
                        onclick: move |_| {
                            nav_item_classes.set(["", "is-active"]);
                        },
                        Icon {
                            width: 16,
                            height: 16,
                            icon: BsStars,
                        }
                        span {
                            class: "ml-1",
                            "Stargazers"
                        }
                    }
                }
                div {
                    class: "navbar-end",
                    a {
                        class: "navbar-item",
                        href: "https://github.com/photino/zino",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: BsGithub,
                        }
                        span {
                            class: "ml-1",
                            "github"
                        }
                    }
                    a {
                        class: "navbar-item",
                        href: "https://crates.io/crates/zino",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: FaRust,
                        }
                        span {
                            class: "ml-1",
                            "crates.io"
                        }
                    }
                    a {
                        class: "navbar-item",
                        href: "https://docs.rs/zino",
                        Icon {
                            width: 16,
                            height: 16,
                            icon: FaCubes,
                        }
                        span {
                            class: "ml-1",
                            "docs.rs"
                        }
                    }
                }
            }
        }
        main {
            class: "my-4 px-4",
            Outlet::<Route> {}
        }
        footer {}
    }
}
